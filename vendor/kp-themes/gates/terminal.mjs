// A theme read the way a terminal and an editor need it, and its 16 ANSI
// colours. Shared by kp-themes-vscode (the workbench's terminal.ansi* keys)
// and the desktop repos (Windows Terminal, Konsole, the fish colours), which
// used to read the ANSI back out of the generated VS Code themes. It ships in
// the release's tokens.tar so both derive the same colours from one copy.

import { readFileSync } from 'node:fs';
import { contrast, hslToRgb, parseHsl, rgbToHsl } from './colour.mjs';

const ROOT = new URL('../', import.meta.url);
const THEMES_CSS = readFileSync(new URL('css/themes.css', ROOT), 'utf8');

// ── Reading a theme ─────────────────────────────────────────────────────

/** @param {string} name */
export function readTheme(name) {
    const raw = JSON.parse(readFileSync(new URL(`themes/${name}/tokens.json`, ROOT), 'utf8'));
    /** @type {Record<string, string>} */
    const tokens = Object.fromEntries(
        raw.entries
            .filter((/** @type {{token?: string}} */ e) => e.token !== undefined)
            .map((/** @type {{token: string, value: string}} */ e) => [e.token, e.value]),
    );
    // The derived states exist only in the generated stylesheet.
    const block = new RegExp(`^\\[data-theme='${name}'\\] \\{\\n([\\s\\S]*?)^\\}`, 'm').exec(THEMES_CSS);
    if (!block) throw new Error(`css/themes.css has no block for ${name}`);
    /** @type {Record<string, string>} */
    const derived = {};
    const after = block[1].split('Derived interaction states')[1] ?? '';
    for (const m of after.matchAll(/--([\w-]+):\s*([^;]+);/g)) derived[m[1]] = m[2].trim();
    return { name, label: raw.label, tokens, derived, dark: tokens['color-scheme'] === 'dark' };
}

// ── Colour arithmetic ───────────────────────────────────────────────────

/** @param {[number, number, number]} rgb @param {number} [alpha] */
export function hex(rgb, alpha = 1) {
    const h = (/** @type {number} */ u) =>
        Math.round(Math.min(1, Math.max(0, u)) * 255)
            .toString(16)
            .padStart(2, '0');
    return `#${rgb.map(h).join('')}${alpha < 1 ? h(alpha) : ''}`.toUpperCase();
}

/** @param {string} value #RRGGBB or #RRGGBBAA @returns {{rgb: [number, number, number], alpha: number}} */
export function parseHex(value) {
    const n = (/** @type {number} */ i) => parseInt(value.slice(i, i + 2), 16) / 255;
    return { rgb: [n(1), n(3), n(5)], alpha: value.length === 9 ? n(7) : 1 };
}

/** What the eye sees: `top` (maybe translucent) over an opaque `ground`. */
export function over(/** @type {string} */ top, /** @type {string} */ ground) {
    const t = parseHex(top);
    const g = parseHex(ground);
    return /** @type {[number, number, number]} */ (t.rgb.map((u, i) => t.alpha * u + (1 - t.alpha) * g.rgb[i]));
}

/**
 * Resolve one expression against a theme.
 * @param {ReturnType<typeof readTheme>} theme
 * @param {string} expr
 */
export function resolve(theme, expr) {
    const m = /^([\w-]+)(?:\/([\d.]+))?(?:~(-?\d+))?$/.exec(expr);
    if (!m) throw new Error(`bad expression ${expr}`);
    const [, name, alpha, shift] = m;
    const value = theme.tokens[name] ?? theme.derived[name];
    if (value === undefined || !value.startsWith('hsl')) return null;
    const hsl = parseHsl(value);
    if (shift) {
        // Towards the foreground: lighter on a dark theme, darker on a light one.
        const d = Number(shift) * (theme.dark ? 1 : -1);
        hsl.l = Math.min(96, Math.max(4, hsl.l + d));
    }
    return hex(hslToRgb(hsl), alpha ? Number(alpha) : 1);
}

/**
 * A token that must be there. Every theme declares these, so a missing one
 * is a broken theme rather than a colour to fall back from.
 * @param {ReturnType<typeof readTheme>} theme @param {string} expr
 */
export function must(theme, expr) {
    const hex = resolve(theme, expr);
    if (hex === null) throw new Error(`${theme.name}: --${expr} resolves to nothing`);
    return hex;
}

// ── ANSI: nearest token by hue, each used once where the theme allows ──

export const ANSI_HUES = { Red: 0, Green: 120, Yellow: 55, Blue: 225, Magenta: 300, Cyan: 185 };
export const ANSI_POOL = [
    'primary',
    'accent',
    'destructive',
    'chart-1',
    'chart-2',
    'chart-3',
    'chart-4',
    'chart-5',
    'success-foreground',
    'warning-foreground',
    'info-foreground',
    'link',
    'status-sent',
    'status-screening',
    'status-interview',
    'status-offer',
    'status-rejected',
    'sidebar-accent-foreground',
];

/** @param {ReturnType<typeof readTheme>} theme */
export function ansi(theme) {
    const bg = parseHex(must(theme, 'background')).rgb;
    const hueGap = (/** @type {number} */ a, /** @type {number} */ b) => Math.min(Math.abs(a - b), 360 - Math.abs(a - b));
    /** @type {{expr: string, rgb: import('./colour.mjs').Rgb, hue: number, sat: number, ratio: number}[]} */
    const pool = [];
    for (const expr of ANSI_POOL) {
        const h = resolve(theme, expr);
        if (!h) continue;
        const rgb = parseHex(h).rgb;
        const hsl = rgbToHsl(rgb);
        const candidate = { expr, rgb, hue: hsl.h, sat: hsl.s, ratio: contrast(rgb, bg) };
        if (candidate.sat >= 30 && candidate.ratio >= 4.5) pool.push(candidate);
    }
    /** @type {Record<string, {expr: string, note: string}>} */
    const chosen = {};
    const pairs = Object.entries(ANSI_HUES)
        .flatMap(([slot, hue]) => pool.map((c) => ({ slot, c, gap: hueGap(hue, c.hue) })))
        // Within 5° of each other, the more saturated token is the truer
        // ANSI colour (cyberpunk's accent, not its pale sidebar cyan).
        .sort((a, b) => Math.floor(a.gap / 5) - Math.floor(b.gap / 5) || b.c.sat - a.c.sat || a.gap - b.gap);
    for (const { slot, c, gap } of pairs) {
        if (chosen[slot] || gap > 60) continue;
        const taken = Object.values(chosen).some((x) => {
            const already = pool.find((p) => p.expr === x.expr);
            return already !== undefined && hueGap(already.hue, c.hue) < 20;
        });
        if (taken) continue;
        chosen[slot] = { expr: c.expr, note: `hue ${Math.round(c.hue)}, ${gap.toFixed(0)}° from ${slot.toLowerCase()}` };
    }
    for (const [slot, hue] of Object.entries(ANSI_HUES)) {
        if (chosen[slot]) continue;
        // Fewer hues in the theme than ANSI has slots: share the nearest.
        const near = [...pool].sort((a, b) => hueGap(hue, a.hue) - hueGap(hue, b.hue))[0];
        chosen[slot] = near
            ? { expr: near.expr, note: `shared: no free token within 60° (nearest hue ${Math.round(near.hue)})` }
            : { expr: 'foreground', note: 'shared: no saturated token reads on the ground' };
    }
    /** @type {Record<string, string>} */
    const colors = {};
    /** @type {Record<string, string>} */
    const sources = {};
    const set = (/** @type {string} */ key, /** @type {string} */ expr, note = '') => {
        colors[`terminal.${key}`] = must(theme, expr);
        sources[`terminal.${key}`] = expr + (note ? ` (${note})` : '');
    };
    if (theme.dark) {
        set('ansiBlack', 'muted');
        set('ansiBrightBlack', 'muted-foreground');
        set('ansiWhite', 'foreground');
        set('ansiBrightWhite', 'foreground~10');
    } else {
        // As VS Code's own Light Modern does: "white" text must still read
        // on a light ground, so it is a grey, and only bright white is pale.
        set('ansiBlack', 'foreground');
        set('ansiBrightBlack', 'muted-foreground');
        set('ansiWhite', 'muted-foreground');
        set('ansiBrightWhite', 'border-strong');
    }
    for (const [slot, { expr, note }] of Object.entries(chosen)) {
        set(`ansi${slot}`, expr, note);
        set(`ansiBright${slot}`, `${expr}~10`);
    }
    return { colors, sources };
}

/**
 * What a terminal emulator needs: its ground, ink, selection and cursor,
 * with the same expressions as the VS Code workbench's terminal.* keys, and
 * the sixteen ANSI colours. Keyed as VS Code keys them, so a consumer that
 * used to read a generated VS Code theme reads this unchanged.
 * @param {string} name
 * @returns {Record<string, string>}
 */
export function terminalColors(name) {
    const theme = readTheme(name);
    return {
        'terminal.background': must(theme, 'background'),
        'terminal.foreground': must(theme, 'foreground'),
        'terminal.selectionBackground': must(theme, 'primary/0.3'),
        'terminalCursor.foreground': must(theme, 'primary'),
        ...ansi(theme).colors,
    };
}
