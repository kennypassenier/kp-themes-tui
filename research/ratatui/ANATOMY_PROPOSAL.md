# Anatomy proposal for the other nineteen themes

`research/ratatui/demo/src/anatomy.rs` fills in three of the twenty-two
themes (formal, cyberpunk, terminal). This is a proposal for the other
nineteen, read out of `themes/<name>/anatomy.md`,
`css/<name>-register.css`, `themes/<name>/tokens.json`, `css/components.css`
and `js/effects.js`. Nothing here was run, nothing was committed, nothing in
the repository was changed.

Every cell carries its source. A cell marked **°** is a guess: no file in the
repository states it, and the one-line reason why that default is the
least-wrong one is in the theme's own section below.

**Totals: 64 guesses out of 247 decided fields (19 themes × 13 fields), 26 %.**
Per theme: retro 1 · brutalism 2 · phantom 2 · light 3 · synthwave 3 ·
high-contrast 3 · shade-light 3 · shade-dark 3 · lapis 3 · nostromo 3 ·
titanium 3 · dark 4 · pastel 4 · sepia 4 · deco 4 · grotesk 4 · forest 5 ·
blueprint 5 · solstice 5.

`button_border` is not scored: it follows `border` in every theme, as it does
for formal and terminal, except where a section says otherwise (dark,
titanium, phantom).

## How each field was decided

These conventions were derived from the three rows that already exist, so the
nineteen stay consistent with them. Where a convention itself is a judgement
rather than a reading, the affected cells are marked °.

| #   | Field                   | Rule, and the shipped row it comes from                                                                                                                                                                                                                                                                                                                                                                                                           |
| --- | ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| C1  | `border`                | `--radius` ≥ 0.375 rem → `ROUNDED` (formal, 0.375 rem). `--radius` 0 or a register/anatomy that calls the corner square → `PLAIN` (cyberpunk "the radius is 0", terminal `--radius: 0`). A panel line of 2 px or more → `THICK`. A two-tone bevel → `DOUBLE`. A clip-path chamfer → a named `const`, the way `NOTCHED` is one.                                                                                                                    |
| C2  | `border_focus`          | Register writes its own focus with two lines of different colour (two rings, or outline + ring) → `DOUBLE` (formal's doubling rule). Register keeps the line and only changes its colour → same set as `border` (terminal). Nothing but the base ring → `THICK` °, cyberpunk's own guess.                                                                                                                                                         |
| C3  | `button_face`           | `Square` where the register sets `border-radius: 0` on `.kp-button`, or the anatomy calls the corner square/cut (cyberpunk). `Soft` where the button keeps a visible radius (formal). `Bracket` only where glyphs stand around the label **at rest** (terminal). A clip-path cut is never a glyph: `anatomy.rs:109-112` already threw that out.                                                                                                   |
| C4  | `button_spaced`         | `letter-spacing` on `.kp-button` ≥ 0.1 em → `true`. Cyberpunk 0.12 em is `true`, terminal 0.08 em is `false`, so the line sits between them.                                                                                                                                                                                                                                                                                                      |
| C5  | `uppercase_labels`      | `true` when the register uppercases `.kp-tab` and/or `.kp-button` (cyberpunk, terminal). A `.microlabel`-only uppercase is **not** enough — formal uppercases its microlabel (`formal-register.css:81`) and its row is `false`.                                                                                                                                                                                                                   |
| C6  | `label_prefix`          | The `content:` of `[data-theme=…] .microlabel::before`, verbatim; `""` when the register writes none. A glyph on `.kp-platforms span::before` is a platform list, not a panel title, and does not count.                                                                                                                                                                                                                                          |
| C7  | `tab_divider`           | The register draws the tab strip as plates (own frame, plate ground, folder shape) → `"  "`, terminal's answer. Otherwise `" │ "` °, formal's answer — no file in the package states a tab separator; `components.css:2920` only gives the breadcrumb one (`'/'`, overridden by cyberpunk alone).                                                                                                                                                 |
| C8  | `selected_tab_modifier` | Underline-shaped selection (the base `border-bottom-color` of `components.css:2886`, or an inset bottom shadow) → `UNDERLINED` (formal). A plate → `Modifier::empty()`, the colour carries it (terminal). Weight/position only → `BOLD`.                                                                                                                                                                                                          |
| C9  | `cursor`                | Guessed everywhere but retro, which draws one. `SteadyBar` for the quiet, editorial and light themes (formal's reasoning), `SteadyBlock` for the machine, mono and HUD themes (cyberpunk's). No theme gets a blinking caret unless its own anatomy asks for one; several forbid it outright.                                                                                                                                                      |
| C10 | `reveal`                | The theme's `--kp-reveal-headline`, resolved through `js/effects/headline.js` and the `TIMINGS` table of `js/effects.js`. Only cyberpunk and terminal declare `--kp-decipher-*`, and only they use `decipher`/`type`: **every one of the nineteen routines leaves the text untouched**, so all nineteen are `Arrive`, with the keyframe's own duration. Word-staggered routines carry their stagger in the notes — see "what I could not decide". |

## The table

° = guess. `sel tab` = `selected_tab_modifier`, `focus mod` = `focus_modifier`.

| theme         | border       | border_focus | button_face | spaced | upper | prefix | brackets     | title | focus mod | divider  | sel tab    | cursor       | reveal      |
| ------------- | ------------ | ------------ | ----------- | ------ | ----- | ------ | ------------ | ----- | --------- | -------- | ---------- | ------------ | ----------- |
| light         | ROUNDED      | THICK°       | Soft        | false  | false | `""`   | –            | BOLD  | empty     | `" │ "`° | UNDERLINED | SteadyBar°   | Arrive 620  |
| dark          | CHAMFER_FALL | THICK°       | Square      | false  | false | `"· "` | `("[", "]")` | BOLD  | BOLD°     | `" │ "`° | UNDERLINED | SteadyBar°   | Arrive 640  |
| synthwave     | PLAIN        | DOUBLE       | Square      | true   | true  | `"▶ "` | –            | BOLD  | empty°    | `" │ "`° | UNDERLINED | SteadyBlock° | Arrive 700  |
| pastel        | ROUNDED      | THICK°       | Soft        | false  | false | `""`   | –            | BOLD  | empty°    | `" │ "`° | UNDERLINED | SteadyBar°   | Arrive 650  |
| forest        | ROUNDED      | THICK°       | Soft        | false  | false | `"◦ "` | –            | BOLD° | empty°    | `" │ "`° | UNDERLINED | SteadyBar°   | Arrive 500  |
| high-contrast | THICK        | DOUBLE       | Square      | false  | false | `""`   | –            | BOLD  | empty°    | `" │ "`° | UNDERLINED | SteadyBlock° | Arrive 550  |
| sepia         | ROUNDED      | DOUBLE       | Soft        | false  | false | `""`   | –            | BOLD° | empty°    | `" │ "`° | UNDERLINED | SteadyBar°   | Arrive 1050 |
| blueprint     | PLAIN        | THICK°       | Square      | false  | false | `""`   | –            | BOLD° | empty°    | `" │ "`° | UNDERLINED | SteadyBar°   | Arrive 300  |
| solstice      | ROUNDED      | THICK°       | Soft        | false  | false | `""`   | –            | BOLD° | empty°    | `" │ "`° | UNDERLINED | SteadyBar°   | Arrive 740  |
| brutalism     | THICK        | DOUBLE       | Square      | true   | true  | `""`   | –            | BOLD  | empty°    | `"  "`   | empty      | SteadyBlock° | Arrive 260  |
| deco          | PLAIN        | PLAIN        | Square      | false  | true  | `""`   | –            | BOLD° | empty°    | `" │ "`° | UNDERLINED | SteadyBar°   | Arrive 520  |
| phantom       | PHANTOM_BAR  | DOUBLE       | Square      | true   | true  | `"▮ "` | –            | BOLD  | empty°    | `"  "`   | empty      | SteadyBlock° | Arrive 620  |
| shade-light   | ROUNDED      | DOUBLE       | Soft        | false  | false | `""`   | –            | BOLD  | empty°    | `" │ "`° | UNDERLINED | SteadyBar°   | Arrive 520  |
| shade-dark    | ROUNDED      | DOUBLE       | Soft        | false  | false | `""`   | –            | BOLD  | empty°    | `" │ "`° | UNDERLINED | SteadyBar°   | Arrive 600  |
| retro         | DOUBLE       | DOUBLE       | Square      | false  | false | `"> "` | –            | BOLD  | REVERSED° | `"  "`   | BOLD       | SteadyBlock  | Arrive 640  |
| grotesk       | THICK        | THICK°       | Square      | false  | false | `""`   | –            | BOLD  | REVERSED° | `" │ "`° | UNDERLINED | SteadyBar°   | Arrive 640  |
| lapis         | ROUNDED      | THICK°       | Soft        | false  | false | `""`   | –            | BOLD  | empty°    | `"  "`   | empty      | SteadyBar°   | Arrive 900  |
| nostromo      | ROUNDED      | DOUBLE       | Soft        | true   | true  | `""`   | –            | BOLD° | empty°    | `"  "`   | UNDERLINED | SteadyBlock° | Arrive 340  |
| titanium      | CHAMFER_RISE | DOUBLE       | Square      | false  | false | `"· "` | –            | BOLD  | empty°    | `" │ "`° | UNDERLINED | SteadyBar°   | Arrive 640  |

`–` in the brackets column is `("", "")`.

### The two new border sets

Following `NOTCHED`, which names cyberpunk's cut corner with a filled
triangle in that corner's cell:

```rust
/// dark's chamfer: the top-right and bottom-left corners cut on the
/// diagonal (`clip-path` on `.kp-card::after` and `.kp-button`,
/// `css/dark-register.css:117`, `:137`).
pub const CHAMFER_FALL: border::Set<'static> =
    border::Set { top_right: "◥", bottom_left: "◣", ..border::PLAIN };

/// titanium's chamfer: the same cut, mirrored — top-left and bottom-right
/// (`css/titanium-register.css:82`, `:96`).
pub const CHAMFER_RISE: border::Set<'static> =
    border::Set { top_left: "◤", bottom_right: "◢", ..border::PLAIN };

/// phantom's panel: no frame at all, one heavy bar down the left edge
/// (`.kp-card { border: 0; border-left: 5px solid var(--primary) }`,
/// `css/phantom-register.css:736`).
pub const PHANTOM_BAR: border::Set<'static> =
    border::Set { vertical_left: "┃", ..border::PLAIN };
```

## Per theme: the evidence

### light — 3 guesses

| field                 | value      | evidence                                                                                                                                                                |
| --------------------- | ---------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| border                | ROUNDED    | `themes/light/tokens.json` `--radius: 0.5rem`; `themes/light/anatomy.md:34` "a small surface radius (`--radius`, 0.5rem) for cards, inputs and the dialog"              |
| border_focus          | THICK°     | `css/light-register.css:975` is one ring (`0 0 0 2px var(--focus-ring)`) plus the base outline; nothing doubles a line. ° cyberpunk's own default for a plain ring      |
| button_face           | Soft       | `css/light-register.css:350` `border-radius: var(--kp-radius-pill, 999px)` on `.kp-button` — the maximum radius, so half-block caps                                     |
| button_spaced         | false      | no `letter-spacing` on `.kp-button` anywhere in the register                                                                                                            |
| uppercase_labels      | false      | the register uppercases neither `.kp-tab` nor `.kp-button`, and writes no `.microlabel` rule at all                                                                     |
| label_prefix          | `""`       | `css/light-register.css` contains no `.microlabel` selector                                                                                                             |
| button_brackets       | `("", "")` | no `content:` on `.kp-button::before/::after`                                                                                                                           |
| title_modifier        | BOLD       | `css/light-register.css:74` heading `font-weight: 600`                                                                                                                  |
| focus_modifier        | empty      | the focus is a ring around the plate (`:975`); the label is untouched                                                                                                   |
| tab_divider           | `" │ "`°   | ° no tab separator exists in the package; formal's row is the house answer                                                                                              |
| selected_tab_modifier | UNDERLINED | `css/light-register.css:618` `border-bottom: 2px solid var(--primary)`                                                                                                  |
| cursor                | SteadyBar° | ° a quiet light theme; `anatomy.md:187` has two one-shot keyframes and nothing that loops                                                                               |
| reveal                | Arrive 620 | `--kp-reveal-headline: clip` (`:64`); `TIMINGS['kp-clip-reveal']` 620 ms (`js/effects.js:564`); the clip window opens over text that is never touched (`anatomy.md:51`) |

### dark — 4 guesses

| field                 | value        | evidence                                                                                                                                                                                                                         |
| --------------------- | ------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| border                | CHAMFER_FALL | `css/dark-register.css:117-128` (controls) and `:137-143` (panels) cut the top-right and bottom-left at 45°; `themes/dark/anatomy.md:49` "Cut, never rounded. The token radius is 0"                                             |
| border_focus          | THICK°       | `css/dark-register.css:463` is the base ring only. ° as cyberpunk                                                                                                                                                                |
| button_face           | Square       | `--radius: 0`; the chamfer is a `clip-path`, and `anatomy.rs:109-112` already rejected a one-cell cut as a glyph                                                                                                                 |
| button_spaced         | false        | `.kp-button` sets none; the 0.22 em at `:305` is on `.kp-button__readout`, a readout above the control                                                                                                                           |
| uppercase_labels      | false        | uppercase reaches `.microlabel` (`:498`) and the readout only, not `.kp-tab` or `.kp-button` — formal's precedent                                                                                                                |
| label_prefix          | `"· "`       | `css/dark-register.css:503` `.microlabel::before { content: '· ' }`                                                                                                                                                              |
| button_brackets       | `("[", "]")` | `css/dark-register.css:236` and `:242`. **They are not a rest state**: `:222-231` sets `opacity: 0`, and `:267-272` brings them in on hover/focus only. So the face stays `Square` and the brackets belong to the focused button |
| title_modifier        | BOLD         | `css/dark-register.css:475` heading `font-weight: 700`                                                                                                                                                                           |
| focus_modifier        | BOLD°        | the label itself changes on focus — `.kp-button__label { scale: 0.94 }` (`:294`) and the brackets close. ° bold is the nearest a cell grid has to "the label draws itself in"                                                    |
| tab_divider           | `" │ "`°     | ° as light                                                                                                                                                                                                                       |
| selected_tab_modifier | UNDERLINED   | `css/dark-register.css:1326` `font-weight: 600` + `border-bottom: 2px solid var(--primary)`                                                                                                                                      |
| cursor                | SteadyBar°   | ° `themes/dark/anatomy.md:173-175` "nothing free-runs or loops", so not blinking; an instrument's caret is the quiet one                                                                                                         |
| reveal                | Arrive 640   | `--kp-reveal-headline: resolve` (`:47`), `--kp-word-stagger: 28ms` (`:38`), `TIMINGS['kp-resolve']` 640 ms. Word-staggered, not per-glyph                                                                                        |

`button_border`: `CHAMFER_FALL` too — the button takes the same cut at
`--kp-chamfer` (0.55 rem) where the panel takes `--kp-chamfer-panel` (1 rem).

### synthwave — 3 guesses

| field                 | value        | evidence                                                                                                                                                      |
| --------------------- | ------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| border                | PLAIN        | `themes/synthwave/anatomy.md:46` "The radius is 2px. Every cut is horizontal … Nothing is notched"; card is a 1 px hairline, `css/synthwave-register.css:795` |
| border_focus          | DOUBLE       | `css/synthwave-register.css:604`: two inset rings, `--focus-ring` then `--focus-ring-contrast` two pixels further out — two lines                             |
| button_face           | Square       | `:576` `border-radius: var(--radius)` = 2 px, under a cell; the anatomy forbids a notch                                                                       |
| button_spaced         | true         | `:571` `letter-spacing: 0.16em` on `.kp-button`                                                                                                               |
| uppercase_labels      | true         | `:572` on `.kp-button` and `:1241` on `.kp-tab`                                                                                                               |
| label_prefix          | `"▶ "`       | `css/synthwave-register.css:124` `.microlabel::before`                                                                                                        |
| button_brackets       | `("", "")`   | `:613-615` `.kp-button::before/::after` are `content: ''` decorations                                                                                         |
| title_modifier        | BOLD         | `:135` heading `font-weight: 900`                                                                                                                             |
| focus_modifier        | empty°       | ° the double ring sits inside the plate and nothing touches the label                                                                                         |
| tab_divider           | `" │ "`°     | ° as light                                                                                                                                                    |
| selected_tab_modifier | UNDERLINED   | `:1249` `box-shadow: inset 0 -4px 0 0 var(--primary)`                                                                                                         |
| cursor                | SteadyBlock° | ° a neon HUD; cyberpunk's reasoning                                                                                                                           |
| reveal                | Arrive 700   | `--kp-reveal-headline: tracking` (`:49`), `TIMINGS['kp-tracking']` 700 ms; the shine follows at 1400 ms. The text is never touched                            |

### pastel — 4 guesses

| field                 | value      | evidence                                                                                                                                                 |
| --------------------- | ---------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| border                | ROUNDED    | `--radius: 1rem`, "the largest token radius in the package" (`themes/pastel/anatomy.md:43`); card 1.2 rem (`css/pastel-register.css:544`)                |
| border_focus          | THICK°     | `:401` composes one ring with the sticker shadow. ° as cyberpunk                                                                                         |
| button_face           | Soft       | `css/pastel-register.css:373` `border-radius: 0.85rem`                                                                                                   |
| button_spaced         | false      | none on `.kp-button`                                                                                                                                     |
| uppercase_labels      | false      | uppercase reaches `.microlabel` only (`:109`)                                                                                                            |
| label_prefix          | `""`       | no `.microlabel::before`                                                                                                                                 |
| button_brackets       | `("", "")` | none                                                                                                                                                     |
| title_modifier        | BOLD       | `:96` heading `font-weight: 700`                                                                                                                         |
| focus_modifier        | empty°     | ° the ring is composed with the sticker; nothing reaches the label                                                                                       |
| tab_divider           | `" │ "`°   | ° as light                                                                                                                                               |
| selected_tab_modifier | UNDERLINED | `:972` sets colour only, so the base underline of `css/components.css:2886` stands                                                                       |
| cursor                | SteadyBar° | ° `anatomy.md:156` "Take a sharp corner, or a flicker" is on the may-not list                                                                            |
| reveal                | Arrive 650 | `--kp-reveal-headline: overprint` (`:73`), `TIMINGS['kp-registration']` 650 ms, stagger 150 ms; the overprint is a second layer, not a changed character |

### forest — 5 guesses

| field                 | value      | evidence                                                                                                                                                                        |
| --------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| border                | ROUNDED    | `--radius: 0.625rem`; card `border: 1px solid var(--border-strong)` (`css/forest-register.css:538`)                                                                             |
| border_focus          | THICK°     | `:417` is one ring plus the offset shadow. ° as cyberpunk                                                                                                                       |
| button_face           | Soft       | the button keeps the base `border-radius: var(--radius)` (`css/components.css:480`); `:401` overrides paint only                                                                |
| button_spaced         | false      | none on `.kp-button`                                                                                                                                                            |
| uppercase_labels      | false      | uppercase reaches `.microlabel` only (`:93`)                                                                                                                                    |
| label_prefix          | `"◦ "`     | `css/forest-register.css:97` `.microlabel::before { content: '◦ ' }`                                                                                                            |
| button_brackets       | `("", "")` | none                                                                                                                                                                            |
| title_modifier        | BOLD°      | ° the register sets no heading weight; formal's reasoning — bold is the only weight channel a terminal has                                                                      |
| focus_modifier        | empty°     | ° the ring does the work                                                                                                                                                        |
| tab_divider           | `" │ "`°   | ° as light                                                                                                                                                                      |
| selected_tab_modifier | UNDERLINED | `:891` `border-bottom-color: var(--primary)` over the `.kp-tab` transparent bottom                                                                                              |
| cursor                | SteadyBar° | ° `anatomy.md:180` two once-on-load changes and nothing else moves                                                                                                              |
| reveal                | Arrive 500 | no `--kp-reveal-headline` at all (`css/forest-register.css:60`, "js/effects.js treats the hook as quiet"); the register's own `kp-headline-in 0.5s` (`:1018`), `TIMINGS` 500 ms |

### high-contrast — 3 guesses

| field                 | value        | evidence                                                                                                                                                     |
| --------------------- | ------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| border                | THICK        | `css/high-contrast-register.css:512` card `border: 2px solid var(--border-strong)`, `box-shadow: none`                                                       |
| border_focus          | DOUBLE       | `:63` `:focus-visible { box-shadow: 0 0 0 2px var(--background), 0 0 0 4px var(--foreground) }` — a line, a gap, a second line                               |
| button_face           | Square       | `themes/high-contrast/anatomy.md:37` "`--radius: 0.25rem`. Nearly square. A soft corner blurs the edge"                                                      |
| button_spaced         | false        | none on `.kp-button`                                                                                                                                         |
| uppercase_labels      | false        | uppercase reaches `.microlabel` only (`:93`)                                                                                                                 |
| label_prefix          | `""`         | no `.microlabel::before`                                                                                                                                     |
| button_brackets       | `("", "")`   | `:384` and `:411` are `content: ''` bars, not glyphs                                                                                                         |
| title_modifier        | BOLD         | weight is this theme's own channel: `.kp-button` 700 (`:334`), `.kp-tab` 700 (`:859`), `.microlabel` 700 (`:96`)                                             |
| focus_modifier        | empty°       | ° the four-pixel double ring is already the loudest focus in the package                                                                                     |
| tab_divider           | `" │ "`°     | ° as light                                                                                                                                                   |
| selected_tab_modifier | UNDERLINED   | `:862` `border-bottom-color: var(--foreground)` over the 2 px bottom of `:857`                                                                               |
| cursor                | SteadyBlock° | ° the largest caret is the legible one for this theme's purpose                                                                                              |
| reveal                | Arrive 550   | no `--kp-reveal-headline` (`anatomy.md:105-113`, "the text intact throughout"); `--kp-headline-wipe: 550ms` (`:39`), `TIMINGS['kp-hc-headline-wipe']` 550 ms |

### sepia — 4 guesses

| field                 | value       | evidence                                                                                                                                                                                                                              |
| --------------------- | ----------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| border                | ROUNDED     | `--radius: 0.375rem`, formal's own value; card 1 px, radius 0.5 rem (`css/sepia-register.css:624`)                                                                                                                                    |
| border_focus          | DOUBLE      | `:1366-1376` gives the button a second, three-sided frame in `--primary` on its `::before`, and `:1383` brings it to full opacity on focus: a line inside the line                                                                    |
| button_face           | Soft        | `:497` `border-radius: 0.375rem`; `themes/sepia/anatomy.md:108` "every button is a plain plate with no bevel, the theme's own 0.375rem radius"                                                                                        |
| button_spaced         | false       | none on `.kp-button`                                                                                                                                                                                                                  |
| uppercase_labels      | false       | `:149` `.microlabel { text-transform: lowercase }` — this theme lowercases                                                                                                                                                            |
| label_prefix          | `""`        | no `.microlabel::before`                                                                                                                                                                                                              |
| button_brackets       | `("", "")`  | `:1366` and `:1429` are `content: ''`                                                                                                                                                                                                 |
| title_modifier        | BOLD°       | ° `:136` heading `font-weight: 400` and `:157-159` "the theme's whole character is the face" — the register argues against bold; formal's own guess (a serif cannot exist in a terminal, bold is the only channel left) argues for it |
| focus_modifier        | empty°      | ° the second frame does the work                                                                                                                                                                                                      |
| tab_divider           | `" │ "`°    | ° as light                                                                                                                                                                                                                            |
| selected_tab_modifier | UNDERLINED  | `:1075` `border-bottom-color: var(--primary)` + `font-weight: 600`                                                                                                                                                                    |
| cursor                | SteadyBar°  | ° `anatomy.md` "unhurried on purpose"; nothing here blinks                                                                                                                                                                            |
| reveal                | Arrive 1050 | `--kp-reveal-headline: ink` (`:108`), `--kp-ink-duration: 1050ms` (`:82`), `js/effects/headline.js:191` `later(finish, 1050 + 50)`. One transition over the whole line                                                                |

### blueprint — 5 guesses

| field                 | value      | evidence                                                                                                                               |
| --------------------- | ---------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| border                | PLAIN      | `themes/blueprint/anatomy.md:53` "`--radius: 0.125rem`. Nearly square, because a drawn line does not"; card 1 px, radius 2 px (`:674`) |
| border_focus          | THICK°     | `:1389` lights the four corner construction marks on focus, which is not a second frame. ° as cyberpunk                                |
| button_face           | Square     | `:559` `border-radius: 2px`; the one mitred corner is on `.kp-button--mirror` alone (`:614`)                                           |
| button_spaced         | false      | none on `.kp-button`                                                                                                                   |
| uppercase_labels      | false      | uppercase reaches `.microlabel` only (`:113`)                                                                                          |
| label_prefix          | `""`       | `.microlabel` (`:109`) has no `::before`; the `'▪ '` at `:407` is `.kp-platforms span::before`                                         |
| button_brackets       | `("", "")` | `:1349` is `content: ''`                                                                                                               |
| title_modifier        | BOLD°      | ° the register sets the display family and no weight; formal's reasoning                                                               |
| focus_modifier        | empty°     | ° the corner marks do it                                                                                                               |
| tab_divider           | `" │ "`°   | ° as light                                                                                                                             |
| selected_tab_modifier | UNDERLINED | `:1007` `border-block-end-color: var(--primary)` + `font-weight: 600`                                                                  |
| cursor                | SteadyBar° | ° a drafting sheet has no caret; the drawn line is thin                                                                                |
| reveal                | Arrive 300 | `--kp-reveal-headline` deliberately undeclared (`:34`, `:62-67`); `kp-headline-fade 300ms ease-out 600ms` (`:1134`), `TIMINGS` 300 ms  |

### solstice — 5 guesses

| field                 | value      | evidence                                                                                                                                                             |
| --------------------- | ---------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| border                | ROUNDED    | `--radius: 0.625rem`; `themes/solstice/anatomy.md:32` "The softest in the set"                                                                                       |
| border_focus          | THICK°     | the register writes a focus rule for `.kp-button--mirror` only (`:308`); everything else takes the base ring. ° as cyberpunk                                         |
| button_face           | Soft       | `:257` `border-radius: var(--radius)`                                                                                                                                |
| button_spaced         | false      | none on `.kp-button`                                                                                                                                                 |
| uppercase_labels      | false      | uppercase reaches `.microlabel` only (`:74`)                                                                                                                         |
| label_prefix          | `""`       | no `.microlabel::before`                                                                                                                                             |
| button_brackets       | `("", "")` | `:1020` is a gradient sweep, `content: ''`                                                                                                                           |
| title_modifier        | BOLD°      | ° `:65` heading `font-weight: 400` on a display serif; formal's reasoning                                                                                            |
| focus_modifier        | empty°     | ° nothing reaches the label                                                                                                                                          |
| tab_divider           | `" │ "`°   | ° as light                                                                                                                                                           |
| selected_tab_modifier | UNDERLINED | `:689` sets colour only; the base underline stands                                                                                                                   |
| cursor                | SteadyBar° | ° "warmth is unhurried" (`anatomy.md:37`); nothing loops                                                                                                             |
| reveal                | Arrive 740 | `--kp-reveal-headline: calibrate` (`:38`), `TIMINGS['kp-cal-slide']` 740 ms; `js/effects/headline.js:194-196` "the text is whole and solid" under three wiping bands |

### brutalism — 2 guesses

| field                 | value        | evidence                                                                                                                                    |
| --------------------- | ------------ | ------------------------------------------------------------------------------------------------------------------------------------------- |
| border                | THICK        | `--kp-line: 3px` (`css/brutalism-register.css:38`); card `border: var(--kp-line) solid var(--border-strong)`, `border-radius: 0` (`:583`)   |
| border_focus          | DOUBLE       | `:396` puts a ring in `--focus-ring-contrast` outside the 3 px frame, and `:69` gives the theme its own `:focus-visible` outline: two lines |
| button_face           | Square       | `:354` `border-radius: 0`; `themes/brutalism/anatomy.md:40` "Radius 0"                                                                      |
| button_spaced         | true         | `:358` `letter-spacing: 0.1em` — exactly on the line between cyberpunk's 0.12 em and terminal's 0.08 em                                     |
| uppercase_labels      | true         | `:359` on `.kp-button`, `:1069` on `.kp-tab`                                                                                                |
| label_prefix          | `""`         | `.microlabel` is a plate (`:108-118`) with no `::before`                                                                                    |
| button_brackets       | `("", "")`   | `:1548` `.kp-button::after { content: var(--kp-label, 'BUTTON') }` is the catalogue stamp, not a bracket                                    |
| title_modifier        | BOLD         | Archivo Black on the headings (`:85` display family, `anatomy.md:40` "Archivo Black for headings")                                          |
| focus_modifier        | empty°       | ° focus moves the whole plate (`translate: -2px -2px`, `:385`) and never the label                                                          |
| tab_divider           | `"  "`       | the tabs are plates with their own 3 px frame (`:1064`) and the selected one takes a plate plus an offset shadow (`:1072`)                  |
| selected_tab_modifier | empty        | `:1072` `background: var(--secondary)` — a plate; `.kp-tab`'s own border override (`:1064`) removes the base underline                      |
| cursor                | SteadyBlock° | ° the heaviest caret for the heaviest theme                                                                                                 |
| reveal                | Arrive 260   | `--kp-reveal-headline: slam` (`:42`), `TIMINGS['kp-slam']` 260 ms, `--kp-word-stagger: 60ms` (`:34`), `--kp-reveal-stagger: 320ms` (`:31`)  |

### deco — 4 guesses

| field                 | value      | evidence                                                                                                                                                                                                      |
| --------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| border                | PLAIN      | `themes/deco/anatomy.md:110` "No radius. `0rem`. The style is corners"; card 1 px, radius 0 (`:516`)                                                                                                          |
| border_focus          | PLAIN      | `:359` `.kp-button:hover, .kp-button:focus-visible { border-color: var(--primary); color: var(--primary) }` — the register keeps the line and moves its colour, exactly terminal's case                       |
| button_face           | Square     | `:326` `border-radius: 0`                                                                                                                                                                                     |
| button_spaced         | false      | `:329` `letter-spacing: 0.08em` — terminal's value, and terminal's row is `false`                                                                                                                             |
| uppercase_labels      | true       | `:330` on `.kp-button`, `:934` on `.kp-tab`                                                                                                                                                                   |
| label_prefix          | `""`       | `.microlabel` (`:90`) has no `::before`; the `'◆ '` at `:1094` is `.kp-platforms span::before`                                                                                                                |
| button_brackets       | `("", "")` | `:1322` is a conic gradient, `content: ''`                                                                                                                                                                    |
| title_modifier        | BOLD°      | ° `:76` heading `font-weight: 400` and `anatomy.md:35` "One is hairline-thin and lives only on h1–h3" — the face is the opposite of bold. `Modifier::empty()` is the honest alternative; see the last section |
| focus_modifier        | empty°     | ° the border colour is the whole focus answer                                                                                                                                                                 |
| tab_divider           | `" │ "`°   | ° as light                                                                                                                                                                                                    |
| selected_tab_modifier | UNDERLINED | `:937` colour + `border-color: var(--primary)` on the base 2 px bottom border                                                                                                                                 |
| cursor                | SteadyBar° | ° a drawn line, no machine voice                                                                                                                                                                              |
| reveal                | Arrive 520 | `--kp-reveal-headline` left undeclared on purpose (`anatomy.md:138-148`); the register's own `kp-cartouche-in 520ms` (`:1136`), `TIMINGS` 520 ms                                                              |

### phantom — 2 guesses

| field                 | value        | evidence                                                                                                                                                                                                                           |
| --------------------- | ------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| border                | PHANTOM_BAR  | `:736` `.kp-card { border: 0; border-left: 5px solid var(--primary); border-radius: 0 }`                                                                                                                                           |
| border_focus          | DOUBLE       | `:78` the theme's own `:focus-visible`: outline in `--focus-ring` plus a ring in `--focus-ring-contrast`; `themes/phantom/anatomy.md:94-96` "Red over black, white as the second channel … the ring sits outside the skewed plate" |
| button_face           | Square       | `:485` `border: 0; border-radius: 0`; the plate is a skewed `::before` (`:516`) and `anatomy.md:82` "The button is the parallelogram" — a skew cannot exist on a cell grid                                                         |
| button_spaced         | true         | `:495` `letter-spacing: 0.12em`                                                                                                                                                                                                    |
| uppercase_labels      | true         | `:496` on `.kp-button`, `:1234` on `.kp-tab`                                                                                                                                                                                       |
| label_prefix          | `"▮ "`       | `css/phantom-register.css:122` `.microlabel::before`                                                                                                                                                                               |
| button_brackets       | `("", "")`   | `:516` and `:525` are `content: ''` plates                                                                                                                                                                                         |
| title_modifier        | BOLD         | `anatomy.md:33` "Barlow Condensed, 900, italic, uppercase for the headings"; `:107`/`:132` `font-weight: 900`                                                                                                                      |
| focus_modifier        | empty°       | ° the ring hugs the skewed plate; the label is a separate, unskewed element (`:512`) and never changes                                                                                                                             |
| tab_divider           | `"  "`       | the selected tab is a skewed plate growing behind the label (`:1239-1257`); plates separate themselves                                                                                                                             |
| selected_tab_modifier | empty        | `:1251` `color: var(--primary-foreground)` over the plate of `:1255`                                                                                                                                                               |
| cursor                | SteadyBlock° | ° condensed uppercase machine voice; cyberpunk's reasoning                                                                                                                                                                         |
| reveal                | Arrive 620   | `--kp-reveal-headline: shout` (`:47`), `TIMINGS['kp-shout']` 620 ms, `--kp-word-stagger: 28ms` (`:33`), `anatomy.md:48` "The headline's words shout in 28 ms apart"                                                                |

`button_border`: `PLAIN` — the bar is the card's device, not the button's;
the button plate carries an ordinary 2 px frame (`:516`).

### shade-light — 3 guesses

| field                 | value      | evidence                                                                                                                                              |
| --------------------- | ---------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| border                | ROUNDED    | `--radius: 0.25rem`; card 1 px `--border-strong`, `border-radius: var(--radius)` (`:509`)                                                             |
| border_focus          | DOUBLE     | `:1291` "Both channels in DIFFERENT colours": `outline-color: var(--focus-ring-contrast)` plus the ring                                               |
| button_face           | Soft       | `:403` `border-radius: var(--radius)`                                                                                                                 |
| button_spaced         | false      | none on `.kp-button`                                                                                                                                  |
| uppercase_labels      | false      | uppercase reaches `.microlabel` only (`:107`)                                                                                                         |
| label_prefix          | `""`       | `.microlabel` is a bordered pill (`:103-113`) with no `::before`                                                                                      |
| button_brackets       | `("", "")` | none                                                                                                                                                  |
| title_modifier        | BOLD       | `:86` heading `font-weight: 600`                                                                                                                      |
| focus_modifier        | empty°     | ° two ring channels, nothing on the label                                                                                                             |
| tab_divider           | `" │ "`°   | ° as light                                                                                                                                            |
| selected_tab_modifier | UNDERLINED | `:894` `border-bottom-color: var(--primary)` + `font-weight: 600`                                                                                     |
| cursor                | SteadyBar° | ° `themes/shade-light/anatomy.md:266` "Glow or blink. No looping animation, no opacity flicker" rules out a blinking caret                            |
| reveal                | Arrive 520 | `--kp-reveal-headline: blur` (`:49`), `--kp-word-stagger: 70ms` (`:45`), `js/effects/headline.js:395` routes `blur` to `kp-word-in`, `TIMINGS` 520 ms |

### shade-dark — 3 guesses

| field                 | value      | evidence                                                                                                                             |
| --------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| border                | ROUNDED    | `--radius: 0.25rem`; card 1 px `--border`, `border-radius: var(--radius)` (`:455`)                                                   |
| border_focus          | DOUBLE     | `:86-97` `outline: 2px solid var(--foreground); outline-offset: 2px; box-shadow: 0 0 0 4px var(--background)` — two lines with a gap |
| button_face           | Soft       | `:346` `border-radius: var(--radius)`                                                                                                |
| button_spaced         | false      | none on `.kp-button`                                                                                                                 |
| uppercase_labels      | false      | uppercase reaches `.microlabel` only (`:76`)                                                                                         |
| label_prefix          | `""`       | `.microlabel` (`:72`) has no `::before`; the `'· '` at `:867` is `.kp-platforms span::before`                                        |
| button_brackets       | `("", "")` | none                                                                                                                                 |
| title_modifier        | BOLD       | `:55` heading `font-weight: 600`                                                                                                     |
| focus_modifier        | empty°     | ° the two-line ring is the whole answer                                                                                              |
| tab_divider           | `" │ "`°   | ° as light                                                                                                                           |
| selected_tab_modifier | UNDERLINED | `:760` `border-bottom: 2px solid var(--accent)`                                                                                      |
| cursor                | SteadyBar° | ° `themes/shade-dark/anatomy.md:183-200` every reveal is one-shot and at rest nothing moves                                          |
| reveal                | Arrive 600 | `--kp-reveal-headline: focus` (`:33`), `--kp-word-stagger: 90ms` (`:27`), `TIMINGS['kp-focus']` 600 ms                               |

### retro — 1 guess

| field                 | value       | evidence                                                                                                                                                                                                              |
| --------------------- | ----------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| border                | DOUBLE      | `:67-68` `--kp-raised` is 98.css's four-inset bevel — a highlight pair and a deep pair, literally two lines around every panel; `--radius: 0rem`                                                                      |
| border_focus          | DOUBLE      | `:121` the theme's own `:focus-visible` adds a ring in `--focus-ring-contrast` to the outline in `--focus-ring`; `:457` shows the two-ring stack in full                                                              |
| button_face           | Square      | `:545` `border-radius: 0`                                                                                                                                                                                             |
| button_spaced         | false       | none on `.kp-button`                                                                                                                                                                                                  |
| uppercase_labels      | false       | the register uppercases neither `.kp-tab`, `.kp-button` nor `.microlabel`                                                                                                                                             |
| label_prefix          | `"> "`      | `css/retro-register.css:162` `.microlabel::before { content: '> ' }`, the comment at `:153` calling it "the DOS prompt"                                                                                               |
| button_brackets       | `("", "")`  | none; the key underline at `:2379` is a `::first-letter` border                                                                                                                                                       |
| title_modifier        | BOLD        | `:132` heading `font-weight: 700`                                                                                                                                                                                     |
| focus_modifier        | REVERSED°   | `themes/retro/anatomy.md:82` "Every hover is the selection bar, instantly" and `:107` "the keyboard highlight is the selection bar". ° only the hover→focus substitution, the same one the terminal row already makes |
| tab_divider           | `"  "`      | `:1739` the tabs are chrome folder tabs with their own frame and bevel; `:1751` the selected one lifts two pixels and overhangs three                                                                                 |
| selected_tab_modifier | BOLD        | `:1751` `font-weight: 600` and the lift; `.kp-tab { border-bottom: 0 }` (`:1744`) removes the base underline                                                                                                          |
| cursor                | SteadyBlock | `:167-175` `.microlabel::after` is a solid block of one cell — the comment at `:153` calls it "a static block cursor"; `anatomy.md:174` "No blink, no marquee"                                                        |
| reveal                | Arrive 640  | `--kp-reveal-headline: dissolve` (`:52`), `TIMINGS['kp-dither-clear']` 640 ms; `js/effects/headline.js:399-402` "the text is whole under a dither" — no character is ever replaced, so this is not `Decipher`         |

### grotesk — 4 guesses

| field                 | value      | evidence                                                                                                                                                                   |
| --------------------- | ---------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| border                | THICK      | every frame in the register is `border: 2px solid` (`:351`, `:384`, `:527`, `:557`, `:587`); `themes/grotesk/anatomy.md:156` "No radius. `0rem`; the grid is right angles" |
| border_focus          | THICK°     | `:485` `box-shadow: 0 0 0 2px var(--focus-ring)` sits inside the same 2 px frame; nothing doubles the line. ° as cyberpunk                                                 |
| button_face           | Square     | `:382` `border-radius: 0`; `anatomy.md:84` "Every button is a plain rectangle, `--radius: 0`"                                                                              |
| button_spaced         | false      | none on `.kp-button`                                                                                                                                                       |
| uppercase_labels      | false      | uppercase reaches `.microlabel` only (`:85`)                                                                                                                               |
| label_prefix          | `""`       | no `.microlabel::before`                                                                                                                                                   |
| button_brackets       | `("", "")` | `:1364` `.kp-button__label::after` is the baseline rule, `content: ''`                                                                                                     |
| title_modifier        | BOLD       | weight is the register's own channel: `.kp-button` 700 (`:382`), `.kp-tab` 700 (`:986`), on a display face                                                                 |
| focus_modifier        | REVERSED°  | `anatomy.md:85` "the primary button's mirror-invert (colour swap on hover)". ° twice over: hover→focus, and primary button→every button                                    |
| tab_divider           | `" │ "`°   | ° as light                                                                                                                                                                 |
| selected_tab_modifier | UNDERLINED | `:992` `box-shadow: inset 0 -3px 0 var(--primary)`                                                                                                                         |
| cursor                | SteadyBar° | ° a Swiss grid theme with no machine voice                                                                                                                                 |
| reveal                | Arrive 640 | `--kp-reveal-headline: sharpen` (`:47`), `TIMINGS['kp-sharpen-in']` 640 ms; `js/effects.js:125-129` "the whole, unsplit line — no word-splitting"                          |

### lapis — 3 guesses

| field                 | value      | evidence                                                                                                                                             |
| --------------------- | ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| border                | ROUNDED    | `--radius: 0.5rem`; card 1 px `--border-strong` (`:568`)                                                                                             |
| border_focus          | THICK°     | `:415` is one ring composed with the theme's rule shadow. ° as cyberpunk                                                                             |
| button_face           | Soft       | `:423` `border-radius: var(--radius)`                                                                                                                |
| button_spaced         | false      | none on `.kp-button`                                                                                                                                 |
| uppercase_labels      | false      | uppercase reaches `.microlabel` only (`:118`)                                                                                                        |
| label_prefix          | `""`       | no `.microlabel::before`                                                                                                                             |
| button_brackets       | `("", "")` | none                                                                                                                                                 |
| title_modifier        | BOLD       | `:104` heading `font-weight: 600`                                                                                                                    |
| focus_modifier        | empty°     | ° the ring is the answer; the label is untouched                                                                                                     |
| tab_divider           | `"  "`     | `:906` the selected tab is a plate in `--primary`                                                                                                    |
| selected_tab_modifier | empty      | `:906` `background: var(--primary)` — a plate, terminal's case: the colour carries it                                                                |
| cursor                | SteadyBar° | ° `themes/lapis/anatomy.md:84` "Nothing blinks"; a gilder's page has no block caret                                                                  |
| reveal                | Arrive 900 | `--kp-reveal-headline: gild` (`:58`), `TIMINGS['kp-burnish']` 900 ms; `anatomy.md:49` "The burnish, not a decipher or a type. The headline is whole" |

### nostromo — 3 guesses

| field                 | value        | evidence                                                                                                                                                                                    |
| --------------------- | ------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| border                | ROUNDED      | `--radius: 0.75rem`, the largest of the machine themes; card 1 px `--border-strong`, `border-radius: var(--radius)` (`:531`)                                                                |
| border_focus          | DOUBLE       | `:82-93` `box-shadow: 0 0 0 2px var(--background), 0 0 0 4px var(--foreground)`; `themes/nostromo/anatomy.md:173` "the focus ring is ink over beige"                                        |
| button_face           | Soft         | `:387` overrides paint and type only, so the button keeps the base `border-radius: var(--radius)` = 0.75 rem                                                                                |
| button_spaced         | true         | `:391` `letter-spacing: 0.12em`                                                                                                                                                             |
| uppercase_labels      | true         | `:392` on `.kp-button`, `:899` on `.kp-tab`; `anatomy.md:36` "tracked, uppercase mono set on every button, microlabel and platform"                                                         |
| label_prefix          | `""`         | `.microlabel` (`:72`) has no `::before`; the `'// '` at `:277` is `.kp-platforms span::before`                                                                                              |
| button_brackets       | `("", "")`   | `:1271` `.kp-button::before` is a round LED (`border-radius: 50%`), not a bracket                                                                                                           |
| title_modifier        | BOLD°        | ° `:62` heading `font-weight: 400` on a display face; formal's reasoning                                                                                                                    |
| focus_modifier        | empty°       | ° the two-ring focus is the whole answer                                                                                                                                                    |
| tab_divider           | `"  "`       | `:896` folder plates (`border-radius: 0.2rem 0.2rem 0 0`); the selected one takes the card ground (`:904`)                                                                                  |
| selected_tab_modifier | UNDERLINED   | `:904` `box-shadow: inset 0 -2px 0 0 var(--selected)` under the plate                                                                                                                       |
| cursor                | SteadyBlock° | ° `anatomy.md:238` forbids "a blinking cursor" outright, so steady; the voice is uppercase mono, so block                                                                                   |
| reveal                | Arrive 340   | `--kp-reveal-headline: popdown` (`:43`), `TIMINGS['kp-popdown']` 340 ms; `anatomy.md:96-100` "every existing headline routine touches the text content; the demo's own headline never does" |

### titanium — 3 guesses

| field                 | value        | evidence                                                                                                                                                                           |
| --------------------- | ------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| border                | CHAMFER_RISE | `:96-107` panels cut top-left and bottom-right; `themes/titanium/anatomy.md:34` "The machined chamfer. Two corners cut at forty-five degrees … the corner is cut, never [rounded]" |
| border_focus          | DOUBLE       | `:269` `outline-color: var(--foreground)` plus `box-shadow: 0 0 0 2px var(--background)` — two channels                                                                            |
| button_face           | Square       | `--radius: 0rem`; the chamfer is a `clip-path` (`:82`), and `anatomy.rs:109-112` rejected a one-cell cut as a glyph                                                                |
| button_spaced         | false        | `.kp-button` sets none; the 0.2 em at `:167` is on `.kp-button__readout`                                                                                                           |
| uppercase_labels      | false        | uppercase reaches `.microlabel` only (`:318`)                                                                                                                                      |
| label_prefix          | `"· "`       | `css/titanium-register.css:323` `.microlabel::before { content: '· ' }`                                                                                                            |
| button_brackets       | `("", "")`   | stated in the register, `:119-122`: "No brackets. Those are the spectral instrument's notation … Titanium's two pseudo-element slots stay free"                                    |
| title_modifier        | BOLD         | `:295` heading `font-weight: 700`                                                                                                                                                  |
| focus_modifier        | empty°       | ° focus lights `.kp-button__edge` (`:130`) and the readout (`:177`), never the label — unlike dark, whose label scales                                                             |
| tab_divider           | `" │ "`°     | ° as light                                                                                                                                                                         |
| selected_tab_modifier | UNDERLINED   | `:1115` `border-bottom: 2px solid var(--primary)` + `font-weight: 600`                                                                                                             |
| cursor                | SteadyBar°   | ° a material, not a machine (`:119-122`); the quiet caret                                                                                                                          |
| reveal                | Arrive 640   | `--kp-reveal-headline: resolve` (`:41`), `--kp-word-stagger: 28ms` (`:32`), `TIMINGS['kp-resolve']` 640 ms                                                                         |

`button_border`: `CHAMFER_RISE` too — `:82` cuts the button with the same
`--kp-chamfer`.

## What I could not decide

1. **`Reveal` has no word-stagger variant.** Seven themes reveal a headline
   word by word, not whole and not per glyph: dark (`resolve`, 640 ms / 28 ms),
   phantom (`shout`, 620 / 28), brutalism (`slam`, 260 / 60), shade-light
   (`blur`, 520 / 70), shade-dark (`focus`, 600 / 90), titanium (`resolve`,
   640 / 28) and, at the page level, several more through
   `--kp-reveal-stagger`. `Arrive { ms }` loses that entirely. A fourth
   variant — `Words { ms: u32, stagger_ms: u32 }` — would carry both numbers
   and is the faithful translation. The table proposes `Arrive` with the
   keyframe's own duration so nothing is invented; the stagger is recorded in
   each theme's row.
2. **dark's brackets.** `'['` and `']'` are real, but at rest they are
   `opacity: 0` (`css/dark-register.css:222`); they close on hover and focus
   (`:267`). So `ButtonFace::Bracket`, which is a rest state, would be wrong,
   and `button_brackets` on a `Square` face would put them on every button
   permanently. It needs either a focus-only bracket in the demo or a
   deliberate decision to show them always. Flagged, not decided.
3. **`tab_divider` has no source anywhere.** The package states a breadcrumb
   glyph (`css/components.css:2920`, `'/'`, overridden only by cyberpunk) and
   nothing at all for tabs. Formal's shipped `" │ "` is itself unsourced.
   Fourteen of the nineteen therefore inherit a guess. If Kenny prefers `" / "`
   everywhere, that is one line per row and just as defensible.
4. **`title_modifier` for the light-faced themes.** sepia (`font-weight: 400`,
   "the theme's whole character is the face"), deco ("hairline-thin", 400),
   solstice (400), nostromo (400) and blueprint (no weight at all) get `BOLD`
   only because a terminal has no other weight channel — formal's own guess,
   reused. For deco in particular `Modifier::empty()` would be truer to the
   register and would leave the panel title indistinguishable from body text.
   A reviewer should pick one rule for all five.
5. **brutalism's `button_spaced` is exactly on the line.** 0.1 em, between
   cyberpunk's 0.12 em (`true`) and terminal's 0.08 em (`false`). Read `true`
   here; a spaced label on a three-pixel plate is in character, but the
   threshold itself is invented.
6. **Two modifiers on one selected tab.** dark, sepia, blueprint, shade-light,
   titanium and nostromo all set both a bottom border and `font-weight: 600`.
   The field takes one `Modifier`; `UNDERLINED` was chosen every time, so the
   weight is dropped. `Modifier::UNDERLINED.union(Modifier::BOLD)` is const and
   would carry both.
7. **The chamfer glyphs.** `CHAMFER_FALL`, `CHAMFER_RISE` and `PHANTOM_BAR`
   follow `NOTCHED`'s convention (a filled triangle in the cut corner's cell).
   Whether a filled triangle reads as a cut corner or as decoration at one cell
   is exactly the judgement that killed cyberpunk's own slit on 2026-09-17
   (`anatomy.rs:109-112`), and it is Kenny's eye to make, not a file's.
8. **retro's and grotesk's `REVERSED`.** Both sources say _hover_
   ("Every hover is the selection bar"; "the primary button's mirror-invert
   (colour swap on hover)"), and grotesk's applies to the primary button alone.
   Read onto focus because a keyboard TUI has no pointer — the same
   substitution the terminal row already documents as a guess.
