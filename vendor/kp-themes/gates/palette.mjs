// Reading a theme's resolved values out of the generated stylesheet, and a
// value as the three bytes a terminal or a desktop colour file receives.
// Shared by the TUI palette and the desktop generators; it ships in the
// release's tokens.tar.

import { readFileSync } from 'node:fs';
import { hsl } from './colour.mjs';

const CSS = readFileSync(new URL('../css/themes.css', import.meta.url), 'utf8');

/**
 * The derived states live only in the generated stylesheet: they are
 * computed in OKLCh there, and reading that output keeps one implementation
 * of the derivation instead of a second one here.
 * @param {string} name @param {string} [css]
 * @returns {Record<string, string>}
 */
export function derivedBlock(name, css = CSS) {
    const block = new RegExp(`^\\[data-theme='${name}'\\] \\{\\n([\\s\\S]*?)^\\}`, 'm').exec(css);
    if (block === null) throw new Error(`css/themes.css has no block for ${name}`);
    /** @type {Record<string, string>} */
    const values = {};
    for (const m of block[1].matchAll(/^\s*--([a-z0-9-]+):\s*([^;]+);/gm)) values[m[1]] = m[2].trim();
    return values;
}

/**
 * `hsl(...)` as the three bytes a terminal receives.
 * @param {string} value
 * @returns {[number, number, number]}
 */
export function rgbOf(value) {
    const [r, g, b] = hsl(value);
    return [Math.round(r * 255), Math.round(g * 255), Math.round(b * 255)];
}
