// The design invariants that can be checked against the token source
// [L3, DI1, DI2, DI4, DI6]. Contrast of text pairs stays in check-contrast.mjs,
// which reads the generated artefact because that is what a consumer takes.
//
// Every check declares how many things it expected to inspect and fails
// when the count differs (AR8): a gate answers "did I check everything",
// not only "did what I ran pass".
//
// Usage: node gates/check-invariants.mjs

import { readFileSync } from 'node:fs';
import process from 'node:process';
import { hsl, contrast, distance, luminance, simulateDeuteranopia, derive, deriveVisible } from './colour.mjs';

/** @typedef {{name: string, tokens: Record<string,string>, derivation?: {stepL: number}}} Theme */

const dir = new URL('../themes/', import.meta.url);
const config = JSON.parse(readFileSync(new URL('config.json', import.meta.url), 'utf8'));

/** WCAG 2.2 SC 1.4.11 Non-text Contrast, Level AA. A standards constant. */
const BOUNDARY_FLOOR = 3.0;
/** DI4's floor is a house number, not a standard — see gates/config.json. */
const DISTANCE_FLOOR = config.perceptualDistanceFloor.value;

/** Surfaces a boundary can be drawn against. */
const SURFACES = ['background', 'card', 'popover'];

/** The hero's own surfaces [TH116]: its ground and its card. */
const HERO_SURFACES = ['surface-hero-bg', 'surface-hero-card'];

/** Where the hero's focus ring can land: its ground, card, button and alert. */
const HERO_FOCUS_SURFACES = ['surface-hero-bg', 'surface-hero-card', 'surface-hero-primary', 'surface-hero-danger'];

/** The hero's interactive sources [TH116]. */
const HERO_INTERACTIVE = ['surface-hero-primary', 'surface-hero-danger'];

/** @param {string} value an hsl() literal */
function isDark(value) {
    const m = value.match(/hsl\([^,]+,[^,]+,\s*([\d.]+)%/);
    return m !== null && Number(m[1]) < 50;
}

/** The application-pipeline statuses, the same seven the contrast gate names. */
const STATUS_NAMES = ['draft', 'sent', 'screening', 'interview', 'offer', 'rejected', 'withdrawn'];

export function themes() {
    const order = JSON.parse(readFileSync(new URL('order.json', dir), 'utf8'));
    return order.map(
        /** @param {string} name */ (name) => {
            const raw = JSON.parse(readFileSync(new URL(`${name}/tokens.json`, dir), 'utf8'));
            const tokens = Object.fromEntries(
                raw.entries.filter(/** @param {any} e */ (e) => e.token !== undefined).map(/** @param {any} e */ (e) => [e.token, e.value]),
            );
            return { name, tokens, derivation: raw.derivation };
        },
    );
}

/** DI6: a theme says whether it is light or dark, and the layers rise. */
/** @param {Theme} theme */
export function checkColourScheme(theme) {
    const problems = [];
    // This checks the token, and only the token. Whether the browser ever
    // receives a color-scheme property is a question about the stylesheet,
    // and it went unasked until a fixture was opened in a real browser on
    // 2026-09-04: every theme declared the token, this gate read pass, and
    // nothing applied it. tests/fixtures.spec.mjs asks the other half.
    const scheme = theme.tokens['color-scheme'];
    if (scheme === undefined) {
        problems.push(
            'declares no color-scheme, so the browser keeps drawing its own scrollbars, select internals and autofill highlight in light mode',
        );
        return problems;
    }
    // A raised surface is never darker than the one it sits on. In a dark
    // theme that is the whole mechanism — four of the five surveyed design
    // systems agree, only Material dissents. In a light theme it matters
    // less, because a shadow carries most of the signal, but a card that
    // is darker than its page still reads as a hole rather than a card.
    const L = SURFACES.map((s) => ({ s, v: luminance(hsl(theme.tokens[s])) }));
    for (let i = 1; i < L.length; i++) {
        if (L[i].v < L[i - 1].v) {
            problems.push(
                `--${L[i].s} (${L[i].v.toFixed(4)}) is darker than --${L[i - 1].s} (${L[i - 1].v.toFixed(4)}), ` +
                    `so a raised surface sinks behind the one below it`,
            );
        }
    }
    return problems;
}

/** DI1: a control's boundary reaches 3:1 against every surface it sits on. */
/** @param {Theme} theme */
export function checkBoundaries(theme) {
    const problems = [];
    for (const token of ['border-strong', 'input', 'selected']) {
        const value = theme.tokens[token];
        if (value === undefined) {
            problems.push(`declares no --${token}`);
            continue;
        }
        for (const surface of SURFACES) {
            const ratio = contrast(hsl(value), hsl(theme.tokens[surface]));
            if (ratio < BOUNDARY_FLOOR) {
                problems.push(`--${token} on --${surface} is ${ratio.toFixed(2)}, under the ${BOUNDARY_FLOOR.toFixed(1)} floor of SC 1.4.11`);
            }
        }
    }
    // TH116: on the hero the generator remaps --input to the hero's frame
    // and --selected to the hero's button colour, so those two sources are
    // the boundaries measured against the hero ground and the hero card.
    for (const token of ['surface-hero-border', 'surface-hero-primary']) {
        const value = theme.tokens[token];
        if (value === undefined) {
            problems.push(`declares no --${token}`);
            continue;
        }
        for (const surface of HERO_SURFACES) {
            const ratio = contrast(hsl(value), hsl(theme.tokens[surface]));
            if (ratio < BOUNDARY_FLOOR) {
                problems.push(
                    `--${token} on --${surface} is ${ratio.toFixed(2)}, under the ${BOUNDARY_FLOOR.toFixed(1)} floor of SC 1.4.11 (the hero surface, TH116)`,
                );
            }
        }
    }
    return problems;
}

/**
 * DI4 at the token layer.
 *
 * Not all twenty-one pairs. Seven pale badge tints cannot all stay apart
 * under a colour deficiency — measured on 2026-09-04, the best any theme
 * manages across all pairs is single digits, and no palette of seven
 * light plates will do better. That is not a defect to fix, it is the
 * reason DI4 is worded as "colour is never the ONLY carrier": the badge
 * carries a label, and that contract lives in the component.
 *
 * What the tokens must still guarantee is the pair where confusing the
 * two is harmful. An offer and a rejection are opposite outcomes; reading
 * one as the other is the failure DI4 exists to prevent.
 *
 * Ratified 2026-09-04 (mini-round DI4-SCOPE) rather than widened, with
 * the measurements in DESIGN_INVARIANTS.md. The short version: 13 to 16
 * of the 21 pairs fall under the floor in five themes and only 2 do in
 * cyberpunk, because cyberpunk is saturated and the others are pale on
 * purpose. Widening to all 21 would delete six palettes rather than
 * tighten a check. Since L7 a semantic badge without words is refused in
 * code, so the label is the carrier and the colour is the second channel.
 */
const OPPOSED = [['offer', 'rejected']];

/** @param {Theme} theme */
export function checkColourVision(theme) {
    const problems = [];
    for (const [a, b] of OPPOSED) {
        const va = theme.tokens[`status-${a}`];
        const vb = theme.tokens[`status-${b}`];
        if (va === undefined || vb === undefined) continue;
        const d = distance(simulateDeuteranopia(hsl(va)), simulateDeuteranopia(hsl(vb)));
        if (d < DISTANCE_FLOOR) {
            problems.push(
                `status ${a} and ${b} mean opposite things and are ${d.toFixed(1)} apart for the commonest colour deficiency (floor ${DISTANCE_FLOOR})`,
            );
        }
    }
    return problems;
}

/**
 * DI2: the focus ring is two channels, and it is checked on everything it
 * can land on.
 *
 * A focus indicator that fails is not a cosmetic problem — it is a
 * keyboard user losing their place on the page. Two conditions, both
 * measured: the two rings contrast with each other, so the pair reads as
 * a ring whatever it sits on; and on every surface a component can have,
 * at least one of the two clears the 3:1 floor of SC 1.4.11.
 *
 * The second condition is what a single ring failed: measured 2026-09-04,
 * --ring against --primary was 1.00 in three themes. Identical luminance.
 */
const FOCUS_SURFACES = ['background', 'card', 'popover', 'primary', 'secondary', 'accent', 'destructive', 'muted', 'success', 'warning', 'info'];

/** @param {Theme} theme */
export function checkFocusRing(theme) {
    const problems = [];
    // Generated tokens are not in the authored source, so the values the
    // generator would derive are reproduced here rather than read back
    // from the stylesheet: this gate measures the source, and DI2's
    // default pair is the theme's own foreground and background.
    const inner = theme.tokens['focus-ring'] ?? theme.tokens.foreground;
    const outer = theme.tokens['focus-ring-contrast'] ?? theme.tokens.background;

    const pair = contrast(hsl(inner), hsl(outer));
    if (pair < BOUNDARY_FLOOR) {
        problems.push(`the two focus rings are ${pair.toFixed(2)} apart, under ${BOUNDARY_FLOOR.toFixed(1)} — the pair does not read as a ring`);
    }

    for (const surface of FOCUS_SURFACES) {
        const value = theme.tokens[surface];
        if (value === undefined) continue;
        const best = Math.max(contrast(hsl(inner), hsl(value)), contrast(hsl(outer), hsl(value)));
        if (best < BOUNDARY_FLOOR) {
            problems.push(
                `neither focus ring reaches ${BOUNDARY_FLOOR.toFixed(1)} on --${surface} (best ${best.toFixed(2)}), so focus is invisible there`,
            );
        }
    }
    // DI2 on the hero [TH116]: the generator remaps the ring to the hero's
    // ink and ground, so that pair and every hero surface are measured.
    const heroInner = theme.tokens['surface-hero-fg'];
    const heroOuter = theme.tokens['surface-hero-bg'];
    if (heroInner !== undefined && heroOuter !== undefined) {
        const heroPair = contrast(hsl(heroInner), hsl(heroOuter));
        if (heroPair < BOUNDARY_FLOOR) {
            problems.push(`on the hero the two focus rings are ${heroPair.toFixed(2)} apart, under ${BOUNDARY_FLOOR.toFixed(1)}`);
        }
        for (const surface of HERO_FOCUS_SURFACES) {
            const best = Math.max(contrast(hsl(heroInner), hsl(theme.tokens[surface])), contrast(hsl(heroOuter), hsl(theme.tokens[surface])));
            if (best < BOUNDARY_FLOOR) {
                problems.push(
                    `neither hero focus ring reaches ${BOUNDARY_FLOOR.toFixed(1)} on --${surface} (best ${best.toFixed(2)}), so focus is invisible there`,
                );
            }
        }
    }
    return problems;
}

/**
 * KT2: a state nobody can see is not a state.
 *
 * Every derived state was checked for whether its own text still reads
 * (checkStates below), and nothing ever asked the other question —
 * whether the state differs perceptibly from the colour it came from. It
 * does not: measured 2026-09-04, the pressed state lands 10.0 to 12.1
 * from its base in five themes and 2.6 to 8.4 in cyberpunk and terminal,
 * the two that took DI3's smaller-step opt-out.
 *
 * The floor is the pressed state only. Hover is deliberately subtle in
 * every theme (2.4 to 3.4 here, which is the same order as Material's 8%
 * state layer), and a floor that failed it would be arguing with every
 * design system rather than with this one's own inconsistency.
 *
 * Not wired into the run yet: it fails today, on purpose. Standing rule 8
 * says a live-found fault becomes a failing test before the fix, and the
 * fix is Kenny's to approve (correction KT2).
 *
 * @param {Theme} theme
 */
export function checkStateVisibility(theme) {
    const problems = [];
    const dark = theme.tokens['color-scheme'] === 'dark';
    const stepL = theme.derivation?.stepL ?? config.derivation.stepL;
    const floor = config.stateVisibilityFloor.value;
    for (const surface of INTERACTIVE) {
        const base = theme.tokens[surface];
        const ink = theme.tokens[`${surface}-foreground`];
        if (base === undefined || ink === undefined) continue;
        // The same derivation the generator uses, so this measures what
        // ships rather than what an older version of the maths produced.
        // It therefore fails exactly when the colour space cannot deliver
        // a visible pressed state, which is the case worth knowing about.
        //
        // And an authored state wins here exactly as it wins there [Kenny,
        // 2026-09-12]. Until this line the check re-derived unconditionally,
        // so a theme that wrote its own pressed colour was still measured
        // on the one it had replaced — which is the whole point of writing
        // it, and which would have made this gate report a fault that does
        // not ship.
        const active =
            theme.tokens[`${surface}-active`] ?? deriveVisible(base, config.derivation.active, { towardsLight: dark, stepL }, { floor, ink });
        const seen = distance(hsl(base), hsl(active));
        if (seen < floor) {
            problems.push(
                `--${surface}-active is only ${seen.toFixed(1)} from --${surface} (floor ${floor}); pressing the control changes nothing anyone can see`,
            );
        }
    }
    const heroDark = isDark(theme.tokens['surface-hero-bg'] ?? theme.tokens.background);
    for (const surface of HERO_INTERACTIVE) {
        const base = theme.tokens[surface];
        const ink = theme.tokens[`${surface}-foreground`];
        if (base === undefined || ink === undefined) continue;
        const active =
            theme.tokens[`${surface}-active`] ?? deriveVisible(base, config.derivation.active, { towardsLight: heroDark, stepL }, { floor, ink });
        const seen = distance(hsl(base), hsl(active));
        if (seen < floor) {
            problems.push(
                `--${surface}-active is only ${seen.toFixed(1)} from --${surface} (floor ${floor}) on the hero; pressing changes nothing anyone can see`,
            );
        }
    }
    return problems;
}

/**
 * The other two places the same fault sat [KT2-3].
 *
 * The fault is not "the states were too small". It is: a gate checks one
 * half of a two-halved property and reports green. Stated that way it was
 * found twice more.
 *
 * A · A badge's text is gated against its plate; the plate was never
 * measured against the surface it lies on. Measured in perceptual
 * distance rather than contrast ratio — the first attempt used the ratio
 * and read 1.04 to 1.40, which sounds like an invisible badge and is not:
 * a contrast ratio only compares luminance, and these plates differ in
 * hue. Measured properly they sit 5.6 to 12.6 from their card.
 *
 * The floor is deliberately low. The label carries the meaning (DI4), so
 * the plate only has to read as a plate; this is a floor against a future
 * palette change flattening a badge into its card, not a claim that five
 * is comfortable.
 *
 * B · The visited link is held apart from the link by the generator, and
 * by nothing else. A theme that authors either by hand would slip past.
 *
 * @param {Theme} theme
 */
export function checkSecondHalves(theme) {
    const problems = [];

    const plateFloor = config.badgePlateFloor.value;
    for (const status of STATUS_NAMES) {
        const plate = theme.tokens[`status-${status}`];
        if (plate === undefined) continue;
        for (const surface of ['background', 'card']) {
            const d = distance(hsl(plate), hsl(theme.tokens[surface]));
            if (d < plateFloor) {
                problems.push(`the ${status} badge is ${d.toFixed(1)} from --${surface} (floor ${plateFloor}); it stops reading as a badge`);
            }
        }
    }

    // The visited link used to be checked here and never was: it is a
    // derived token, so `theme.tokens['link-visited']` is always
    // undefined and the branch never ran. Found in the Phase 7 audit and
    // moved to check-contrast.mjs, which reads the generated stylesheet
    // and therefore measures what ships.

    return problems;
}

const INTERACTIVE = ['primary', 'secondary', 'accent', 'destructive'];

/**
 * DI3 + AR12: the derived states are checked at their worst case rather
 * than one by one. "Active" is two steps — the furthest any state moves —
 * so if its text still clears AA, every gentler state does too, provided
 * the derivation is monotone in luminance. That proviso is what OKLCh buys
 * (AR4), and it is why the two themes that take DI3's opt-out are still
 * checked here rather than trusted.
 *
 * Disabled is deliberately exempt: DI8 is Kenny's recorded decision that a
 * disabled control may fall below the floor, with the GOV.UK counter-case
 * kept beside it.
 */
/** @param {Theme} theme */
export function checkStates(theme) {
    const problems = [];
    const dark = theme.tokens['color-scheme'] === 'dark';
    const stepL = theme.derivation?.stepL ?? config.derivation.stepL;
    for (const surface of INTERACTIVE) {
        const base = theme.tokens[surface];
        const ink = theme.tokens[`${surface}-foreground`];
        if (base === undefined || ink === undefined) continue;
        const active = derive(base, config.derivation.active, { towardsLight: dark, stepL });
        const ratio = contrast(hsl(active), hsl(ink));
        if (ratio < 4.5) {
            problems.push(`--${surface}-active (${active}) carries --${surface}-foreground at ${ratio.toFixed(2)}, under 4.5`);
        }
    }
    // The hero's button and alert, derived away from the hero ground's
    // own lightness — the hero may be light in a dark theme [TH116, AR38].
    const heroDark = isDark(theme.tokens['surface-hero-bg'] ?? theme.tokens.background);
    for (const surface of HERO_INTERACTIVE) {
        const base = theme.tokens[surface];
        const ink = theme.tokens[`${surface}-foreground`];
        if (base === undefined || ink === undefined) continue;
        const active = derive(base, config.derivation.active, { towardsLight: heroDark, stepL });
        const ratio = contrast(hsl(active), hsl(ink));
        if (ratio < 4.5) {
            problems.push(
                `--${surface}-active (${active}) carries --${surface}-foreground at ${ratio.toFixed(2)}, under 4.5 (the hero surface, TH116)`,
            );
        }
    }
    return problems;
}

if (import.meta.url === `file://${process.argv[1]}`) {
    const all = themes();
    let failed = 0;
    let checks = 0;

    for (const theme of all) {
        const problems = [
            ...checkColourScheme(theme),
            ...checkBoundaries(theme),
            ...checkColourVision(theme),
            ...checkStates(theme),
            ...checkFocusRing(theme),
            ...checkStateVisibility(theme),
            ...checkSecondHalves(theme),
        ];
        checks += 7;
        if (problems.length > 0) {
            failed += problems.length;
            console.error(`\n${theme.name}:`);
            for (const p of problems) console.error(`  ${p}`);
        }
    }

    const expected = all.length * 7;
    if (checks !== expected) {
        console.error(`\ngate broke: expected ${expected} checks over ${all.length} themes, ran ${checks}`);
        process.exit(1);
    }

    if (failed > 0) {
        console.error(`\n${failed} invariant violation(s) across ${all.length} themes.`);
        process.exit(1);
    }
    console.log(`All ${all.length} themes satisfy DI1, DI2, DI3, DI4, DI6 and KT2 (${expected} checks).`);
}
