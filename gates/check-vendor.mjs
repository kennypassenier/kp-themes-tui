// vendor/kp-themes/ is the unpacked tokens.tar of the kp-themes release
// that vendor/PIN names. This rebuilds the tar from the unpacked copy with
// the flags kp-themes builds it with (sorted names, mtime 0, owner 0), which
// makes it byte-identical to the release asset, and compares its sha256
// with the pin. A hand edit under vendor/ fails here; so does a pin moved
// without unpacking the matching tar.
//
// Usage: node gates/check-vendor.mjs

import { execFileSync } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createHash } from 'node:crypto';

const ROOT = new URL('../', import.meta.url);
const pin = Object.fromEntries(
    readFileSync(new URL('vendor/PIN', ROOT), 'utf8')
        .split('\n')
        .filter((l) => l && !l.startsWith('#'))
        .map((l) => l.split(/\s+/, 2)),
);
const dir = new URL('vendor/kp-themes/', ROOT).pathname;
const files = execFileSync('find', ['.', '-type', 'f'], { cwd: dir, encoding: 'utf8' })
    .split('\n')
    .filter(Boolean)
    .map((f) => f.replace(/^\.\//, ''))
    .sort();
const tmp = mkdtempSync(join(tmpdir(), 'kp-vendor-'));
try {
    const tar = join(tmp, 'tokens.tar');
    execFileSync('tar', ['--sort=name', '--mtime=@0', '--owner=0', '--group=0', '--numeric-owner', '--mode=0644', '-cf', tar, ...files], { cwd: dir });
    const sha = createHash('sha256').update(readFileSync(tar)).digest('hex');
    const version = readFileSync(join(dir, 'VERSION'), 'utf8').trim();
    if (version !== pin.version) {
        console.error(`vendor/kp-themes is ${version}, vendor/PIN says ${pin.version}`);
        process.exit(1);
    }
    if (sha !== pin.sha256) {
        console.error(`vendor/kp-themes rebuilds to ${sha}, vendor/PIN says ${pin.sha256}: the copy was edited, or the pin moved without it`);
        process.exit(1);
    }
    console.log(`vendor: kp-themes ${version}, ${files.length} files, sha256 matches the pin.`);
} finally {
    rmSync(tmp, { recursive: true, force: true });
}
