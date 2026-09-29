// Colour arithmetic shared by every gate and by the state derivation [AR4].
//
// Values are authored and emitted as `hsl(h, s%, l%)`, because that is what
// the stylesheet has always used and what the vendored copies contain. But
// HSL lightness is not perceptually uniform: one numeric step on terminal's
// saturated green and on formal's dark navy look nothing alike. So anything
// that reasons about how a colour *looks* — the derivation, the distance
// checks — converts to OKLCh first and comes back at the end.
//
// Nothing here is configuration. The WCAG numbers below are standards
// constants and stay pinned with the reason; the house numbers that get
// tuned live in gates/config.json.

/** @typedef {import('../js/contrast.js').Hsl} Hsl */
/** @typedef {import('../js/contrast.js').Rgb} Rgb linear-ish sRGB in 0..1 */

// The primitives moved to js/contrast.js when the colour picker needed
// them [TH57]: gates/ is the package's own tooling and not something a
// consumer imports, so the public half lives where public things live.
// Re-exported rather than reimplemented — one implementation is the whole
// point of half the gates in this directory.
export { contrast, formatHsl, hsl, hslToRgb, luminance, parseHsl, rgbToHsl } from '../js/contrast.js';

import { contrast, formatHsl, hsl, hslToRgb, luminance, parseHsl, rgbToHsl } from '../js/contrast.js';

/** @param {number} u */
const srgbToLinear = (u) => (u <= 0.04045 ? u / 12.92 : ((u + 0.055) / 1.055) ** 2.4);
/** @param {number} u */
const linearToSrgb = (u) => (u <= 0.0031308 ? u * 12.92 : 1.055 * u ** (1 / 2.4) - 0.055);

/**
 * Contrast as the BROWSER paints it: each channel rounded to 8 bits before
 * the luminance, because that is what lands on the screen. `contrast()`
 * from js/contrast.js works on unrounded channels and reads up to 0.04
 * higher — enough to call a pair readable that is not (titanium's code
 * string measured 4.51 unrounded and 4.47 painted, 2026-09-17, fix-59).
 * check-contrast.mjs has rounded since TH116, where light's warning pair
 * measured 4.50 in the gate and 4.48 on the rendered page; this is that
 * same function, in one place instead of two.
 * @param {Rgb} a @param {Rgb} b
 */
export function paintedContrast(a, b) {
    const paint = (/** @type {Rgb} */ rgb) => /** @type {Rgb} */ (rgb.map((v) => Math.round(v * 255) / 255));
    return contrast(paint(a), paint(b));
}

/** @param {Rgb} rgb @returns {{L: number, C: number, h: number}} OKLCh */
export function rgbToOklch(rgb) {
    const [r, g, b] = rgb.map(srgbToLinear);
    const l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b);
    const m = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b);
    const s = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);
    const L = 0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s;
    const a = 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s;
    const bb = 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s;
    return { L, C: Math.hypot(a, bb), h: ((Math.atan2(bb, a) * 180) / Math.PI + 360) % 360 };
}

/** @param {{L: number, C: number, h: number}} c @returns {Rgb} clamped to sRGB */
export function oklchToRgb({ L, C, h }) {
    const a = C * Math.cos((h * Math.PI) / 180);
    const b = C * Math.sin((h * Math.PI) / 180);
    const l_ = (L + 0.3963377774 * a + 0.2158037573 * b) ** 3;
    const m_ = (L - 0.1055613458 * a - 0.0638541728 * b) ** 3;
    const s_ = (L - 0.0894841775 * a - 1.291485548 * b) ** 3;
    const lin = [
        4.0767416621 * l_ - 3.3077115913 * m_ + 0.2309699292 * s_,
        -1.2684380046 * l_ + 2.6097574011 * m_ - 0.3413193965 * s_,
        -0.0041960863 * l_ - 0.7034186147 * m_ + 1.707614701 * s_,
    ];
    return /** @type {Rgb} */ (lin.map((u) => Math.min(1, Math.max(0, linearToSrgb(u)))));
}

/**
 * Machado, Oliveira & Fernandes (2009), severity 1.0. Deuteranomaly is the
 * commonest form: about 8% of men and 0.4% of women of European ancestry
 * (Birch, JOSA A 29(3), 2012).
 * @param {Rgb} rgb @returns {Rgb}
 */
export function simulateDeuteranopia(rgb) {
    const M = [
        [0.367322, 0.860646, -0.227968],
        [0.280085, 0.672501, 0.047413],
        [-0.01182, 0.04294, 0.968881],
    ];
    return /** @type {Rgb} */ (M.map((row) => Math.min(1, Math.max(0, row[0] * rgb[0] + row[1] * rgb[1] + row[2] * rgb[2]))));
}

/**
 * Perceptual distance in OKLab, scaled so the numbers read on the familiar
 * CIE76 scale where roughly 10 is "tells them apart".
 * @param {Rgb} a @param {Rgb} b
 */
export function distance(a, b) {
    const A = rgbToOklch(a);
    const B = rgbToOklch(b);
    const [ax, ay] = [A.C * Math.cos((A.h * Math.PI) / 180), A.C * Math.sin((A.h * Math.PI) / 180)];
    const [bx, by] = [B.C * Math.cos((B.h * Math.PI) / 180), B.C * Math.sin((B.h * Math.PI) / 180)];
    return Math.hypot(A.L - B.L, ax - bx, ay - by) * 100;
}

/**
 * Move a colour a given number of derivation steps away from a reference —
 * lighter on a dark ground, darker on a light one. Done in OKLCh so a step
 * is the same perceived size in every theme (AR4), which is the property
 * AR12's worst-case shortcut depends on.
 * @param {string} value the authored hsl() text
 * @param {number} steps
 * @param {{towardsLight: boolean, stepL: number}} opts
 * @returns {string} an hsl() value
 */
export function derive(value, steps, { towardsLight, stepL }) {
    const c = rgbToOklch(hsl(value));
    const L = Math.min(1, Math.max(0, c.L + (towardsLight ? 1 : -1) * steps * stepL));
    return formatHsl(rgbToHsl(oklchToRgb({ ...c, L })));
}

/**
 * The visited-link colour, derived from the link colour [TH31].
 *
 * Browser-default blue scores 1.99, 2.09 and 2.06 against the three dark
 * themes' backgrounds — unreadable, and the default purple is worse. So
 * the link colour is the theme's own, and its visited state is derived
 * from it the way purple-after-blue has always worked: same family,
 * rotated round the hue wheel, slightly less saturated.
 *
 * Rotation first, lightness only if the rotation alone cannot clear the
 * floor, because a hue change is the signal — a link that merely dims
 * reads as a disabled link. Every candidate must clear AA text contrast
 * on both surfaces a link sits on and stay a visible distance from the
 * unvisited colour, or it is not a state anyone can perceive.
 *
 * @param {string} link the authored or derived link colour
 * @param {string[]} surfaces the hsl() values it must be readable on
 * @param {number} distanceFloor
 * @returns {string | null} an hsl() value, or null when nothing clears
 */
export function deriveVisited(link, surfaces, distanceFloor) {
    const base = hsl(link);
    const c = rgbToOklch(base);
    for (const dL of [0, -0.06, 0.06, -0.12, 0.12, -0.18, 0.18]) {
        for (const dh of [45, -45, 60, -60, 30]) {
            const candidate = oklchToRgb({
                L: Math.min(1, Math.max(0, c.L + dL)),
                C: c.C * 0.9,
                h: (c.h + dh + 360) % 360,
            });
            const readable = surfaces.every((s) => contrast(candidate, hsl(s)) >= 4.5);
            if (readable && distance(base, candidate) >= distanceFloor) return formatHsl(rgbToHsl(candidate));
        }
    }
    return null;
}

/**
 * The pressed state, far enough away to be seen [KT2].
 *
 * `derive` moves lightness only, which is right for five of the seven
 * themes and useless for the two whose colours already sit near the top
 * of the space: measured 2026-09-04, cyberpunk's accent reached 2.6 from
 * its base and could only get to 3.3 even at the full default step, where
 * ten is roughly "you can see that it changed". Neon and phosphor have
 * nowhere lighter to go.
 *
 * So when lightness alone falls short, chroma gives way. A neon sign that
 * is pressed loses some of its saturation; that reads as pressed rather
 * than as a different colour, and it keeps the hue — which is what those
 * two themes are made of. The ink is re-checked at every step, because a
 * state whose own text stops reading is not an improvement.
 *
 * A theme that already clears the floor on lightness alone comes back
 * byte-identical, so the five that were right stay untouched.
 *
 * @param {string} value the base colour
 * @param {number} steps
 * @param {{towardsLight: boolean, stepL: number}} opts
 * @param {{floor: number, ink: string}} bar  how far it must move, and the text that must survive
 * @returns {string} an hsl() value
 */
export function deriveVisible(value, steps, opts, { floor, ink }) {
    const base = hsl(value);
    const inkRgb = hsl(ink);
    // Every candidate is tested as it will be written, rounded to whole
    // hsl() numbers. Testing the unrounded colour and emitting the rounded
    // one is how a value lands four thousandths under its own floor.
    /** @param {string} text */
    const reached = (text) => distance(base, hsl(text)) >= floor;
    /** @param {string} text */
    const reads = (text) => contrast(hsl(text), inkRgb) >= 4.5;

    const plain = derive(value, steps, opts);
    if (reached(plain)) return plain;

    const start = rgbToOklch(hsl(plain));
    let best = plain;

    // First chroma: a pressed neon sign loses saturation, which reads as
    // pressed rather than as a different colour, and it keeps the hue —
    // which is what cyberpunk and terminal are made of.
    for (let pull = 0.01; pull <= start.C; pull += 0.01) {
        const candidate = formatHsl(rgbToHsl(oklchToRgb({ ...start, C: Math.max(0, start.C - pull) })));
        if (!reads(candidate)) break;
        best = candidate;
        if (reached(candidate)) return candidate;
    }

    // Then lightness, further than the theme's own step would go. This is
    // the case of a near-black secondary: it has almost no chroma to give
    // up, so the only room left is up or down.
    const direction = opts.towardsLight ? 1 : -1;
    for (let extra = 0.01; extra <= 0.4; extra += 0.01) {
        const L = Math.min(1, Math.max(0, start.L + direction * extra));
        const candidate = formatHsl(rgbToHsl(oklchToRgb({ ...start, L })));
        if (!reads(candidate)) break;
        best = candidate;
        if (reached(candidate)) return candidate;
    }

    // Nothing reached the floor. Returning the closest attempt rather than
    // throwing: the gate reports it, which is the honest place for it.
    return best;
}
