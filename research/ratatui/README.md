# kp-themes as Ratatui components that switch theme at runtime

Kenny, 2026-09-17: "Eerst een Ratatui-onderzoek". Build a demo app with three themes and four widgets to show the
shape before anything is fixed.

Demo: [`demo/`](demo/), a standalone Cargo project (ratatui 0.30.2, crossterm 0.29). This topic has no `demo.html`
because the deliverable is a terminal app. `cargo run` starts it. `cargo run --example snapshot [-- 256|16]` prints
one finished frame per theme. `cargo test` runs the checks, and `--features tachyonfx` swaps the reveal for the
tachyonfx version. Keys: `t` theme, `←/→` tab, `Tab` focus, `Enter` press, `r` replay, `m` motion, `q` quit.

**Measured** (2026-09-17, 16 cores, target directory outside the repository):

- **Build:** `cargo build` succeeds. A clean `--release` build takes 6.22 s. Clippy and `cargo fmt --check` are clean
  with and without the feature.
- **Tests:** 12 pass (4 parity, 8 render), or 13 with `--features tachyonfx`.
- **Binary size** (`wc -c`, stripped): 791,632 bytes hand-written, 1,023,072 bytes with tachyonfx (+231,440, +29 %).
  Normal-dependency crates go from 86 to 95.
- **Parity with the web:** all 300 generated values (87 palette fields and 213 authored tokens) equal Firefox's
  `getComputedStyle` for the same `hsl()`. This was a one-off Playwright check with no server and no port. The
  in-repo test recomputes the values from `tokens.json` and `css/themes.css`.
- **Rendering:** `TestBackend` draws one frame per theme. The tests assert the corner glyph (`╭` formal, `┌`
  cyberpunk and terminal, `╔`/`┏`/`┌` when focused), the ring, border, card and ground colours, the notch `◢`, and
  that the plate colour differs across rest, focus and pressed.
- **Real terminal:** in a pty the release binary exits 0. After `t t q` the config file reads `theme = terminal`. It
  emits 692 truecolor sequences and cursor styles `6`, `2`, `1` and `0` (steady bar, steady block, blinking block,
  then the reset on exit).

| Name                                                     | What it is for                                                                                                                                 | Reference                                                                                                                                                                     | What the package can reuse                                                                                                                     | Build cost                                                              | Recommendation                                                                                             |
| -------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| Generated palette                                        | HSL → 8-bit RGB for 29 roles per theme, plus every authored token                                                                              | [CSS Color 4](https://www.w3.org/TR/css-color-4/)                                                                                                                             | `themes/*/tokens.json`. The derived `-hover`/`-active`/`-disabled` states come from `css/themes.css`, so there is no second OKLCh port         | small                                                                   | build now, from `npm run generate`, not a `build.rs` (see below)                                           |
| Colour-depth fallback                                    | truecolor → xterm-256 (nearest in cube and greys, index ≥ 16) → 16 by role                                                                     | [termstandard/colors](https://github.com/termstandard/colors), [NO_COLOR](https://no-color.org/)                                                                              | nothing yet. Roles (`Surface`, `Ink`, `Fill`, `OnFill`, `Line`, `Signal`) are new                                                              | small                                                                   | build now. In 16 colours, grounds and body text are `Reset`: the user's palette, not ours                  |
| Anatomy per theme                                        | border set, focus set, title modifier, case, prefix, brackets, tab divider, cursor, reveal routine                                             | [border sets](https://docs.rs/ratatui/0.30.0/ratatui/symbols/border/index.html), [SetCursorStyle](https://docs.rs/crossterm/0.29.0/crossterm/cursor/enum.SetCursorStyle.html) | `anatomy.md`, the registers' `content:` and `text-transform`, and the `--kp-decipher-*` knobs. `signature.json` exists for only 3 of 22 themes | medium: 22 hand-judged entries. The demo marks 9 of its choices `GUESS` | build, but review it per theme like a lift                                                                 |
| Panel                                                    | `.kp-card` as a `Block`: rest in `--border-strong`, focus in `--ring` with the focus set                                                       | [Block](https://docs.rs/ratatui/0.30.0/ratatui/widgets/struct.Block.html)                                                                                                     | `--card`, `--border-strong`, `--ring`, and the microlabel prefix                                                                               | small                                                                   | build now                                                                                                  |
| Tabs                                                     | `.kp-tabs`: formal underlines, cyberpunk bolds after `//`, terminal selects with a plate                                                       | [Tabs](https://docs.rs/ratatui/0.30.0/ratatui/widgets/struct.Tabs.html)                                                                                                       | the `.kp-tab[aria-selected]` rule of each register                                                                                             | small                                                                   | build now                                                                                                  |
| Button                                                   | rest, focus, pressed and disabled on `--primary`, `-hover`, `-active` and `-disabled`. Terminal adds `[ ]` and inverse video                   | [kitty keyboard protocol](https://sw.kovidgoyal.net/kitty/keyboard-protocol/)                                                                                                 | the derived states as generated. Terminal's brightening opt-out (DI3) arrives for free                                                         | small                                                                   | build now. "Pressed" is timed (160 ms), because terminals report no key release without the kitty protocol |
| Reveal, hand-written                                     | formal `arrive` (450 ms colour fade), cyberpunk `decipher` (26 cps, 260 ms lead, swap 0.5, `GLYPHS`), terminal `type` (29.41 cps, block caret) | `js/effects.js`                                                                                                                                                               | the routine names, the rates, and `GLYPHS` without block glyphs (AR40)                                                                         | small: `fx::frame` is 41 lines, pure, and needs no state                | build now                                                                                                  |
| [tachyonfx](https://github.com/ratatui/tachyonfx) 0.25.2 | a shader-like effect library over the buffer                                                                                                   | [crate](https://crates.io/api/v1/crates/tachyonfx)                                                                                                                            | `fade_from_fg` matches `arrive`. `coalesce` is not `decipher` (random cells, no glyph churn). `sweep_in` is not `type` (no caret)              | small to wire, large to make faithful (`effect_fn` per routine)         | leave out of the first round. Revisit for modal and toast transitions                                      |
| Reduced motion                                           | the first frame is the finished frame                                                                                                          | none exists for terminals                                                                                                                                                     | DI7's rule: "rest states are the finished look"                                                                                                | small                                                                   | build now: `--reduced-motion`, `KP_REDUCED_MOTION=1`, the config file, and a key                           |
| Runtime switch and config                                | `t` builds a new `Theme` (one `map` over 29 fields). The next `draw` renders in it, and the choice is written to `$XDG_CONFIG_HOME/…`          | [application patterns](https://ratatui.rs/concepts/application-patterns/)                                                                                                     | theme names as ids                                                                                                                             | small                                                                   | build now. A real crate uses `toml` + `dirs`                                                               |
| Headless render tests                                    | one frame per theme, asserting glyphs and colours                                                                                              | [TestBackend](https://docs.rs/ratatui/latest/ratatui/backend/struct.TestBackend.html)                                                                                         | the house habit of asserting the rendered value, not the code                                                                                  | small                                                                   | build now, for all 22 themes                                                                               |
| Prior art: `homelab/tui-preview` `theme.rs` and `fx.rs`  | a hand-tuned cyberpunk: 13 colour constants, semantic style helpers, and 8 effects                                                             | local, read only                                                                                                                                                              | see below                                                                                                                                      | medium to migrate                                                       | adopt the crate after round one                                                                            |

**What the generated version must keep from the prior art.** Stateless effects derived from `(tick, id)`, so nothing
is stored per element. A three-step effect level (`Off`/`Subtle`/`Full`) rather than a boolean. Semantic helpers
(`title_active`, `border_modal`, `border_danger`) instead of raw colours at call sites. A fixed identity hue per stack
(`stack_color`). Load thresholds mapped to status colours. The spinner, braille sparkline, ticker and tab-switch
flicker. What it must not keep: `decrypt` churns through `░▒▓`, which AR40 forbids, and its palette is not any
kp-theme. **Adoption:** `THEME` is a global `const`, referenced 391 times across 8 files, with 20 inline
`Color::Rgb` literals outside `theme.rs` and hard-coded `BorderType::Double`/`Rounded`. Migration means threading
`&Theme` through the `ui/*` functions (`app.theme`), mapping `cyan` → `ring`/`primary`, `magenta` → `accent`, `ok`
→ `success_foreground` and `faint` → `border`, and replacing the border types with `theme.a.border`. `fx.rs` stays
local for its homelab-only effects and takes `FxLevel` from the crate.

## Proposed crates

- **`kp-theme-tokens`** has no dependencies. It holds `Rgb`, `ThemeId` (22), `Palette<C>` with its roles, the
  derived states, `dark`, and the `fx-duration` values. It is written by the package's generator into
  `rust/kp-theme-tokens/src/generated.rs` and checked by the same freshness gate as `css/themes.css`. The demo's
  `build.rs` proves the mapping but reads `../../..`, which a git or crates.io consumer does not have. The tokens
  crate also suits Avalonia later.
- **`kp-tui`** depends on `kp-theme-tokens` and ratatui (using its crossterm re-export). It holds `Theme`
  (`ColorDepth`, the resolved palette and the anatomy), the 22 hand-written anatomies, the widgets, and `fx`. A
  `config` feature stores the choice. Both crates live in a `rust/` workspace in this repository and carry the
  package version, because a released theme never changes in place.

**First real round, components.**

- **Layouts:** a shell with header, tabs, body and status line, plus split and stack (`.kp-shell`, `.kp-row`,
  `.kp-stack`).
- **Tabs.**
- **Buttons:** primary, secondary, destructive and ghost, with their states.
- **Tables:** `.kp-table` with a selected row, a sort mark and a muted header.
- **Modals:** `.kp-dialog` and `.kp-confirm`, drawn with `Clear` and a centred panel that holds focus.
- **Status bar:** key hints and a message.
- **Toasts:** `.kp-toasts`, stacked with a timeout.
- **Effects:** the three reveals, a spinner, `.kp-progress` as cells, and badges from `--status-*`.

**What cannot exist in a terminal.**

- **Fonts:** Fraunces, Rajdhani and KP Tech Mono. The terminal owns the face, which only the terminal theme does not
  miss. Sizes (`--kp-text-display`) and letter-spacing are gone too.
- **Effects:** shadows (`--fx-shadow-offset`, formal's gold offset), blur, bloom and glow, and opacity. Fades become
  colour mixes, and a single step in 16 colours.
- **Textures:** the 0.06 scanlines, grain and vignette.
- **Shapes:** the `clip-path` notch (approximated with `◢`), radii below a cell, and line widths beyond light, heavy
  and double.
- **Input states:** hover, unless mouse capture is on, and a true key release.

## Recommendation

Build the two crates in the next round, starting from this demo's shape. The split that works is the generated
palette, the hand-written anatomy, and widgets that take `&Theme` and name nothing. Switching then costs one struct
rebuild before the next frame, and the parity test makes drift a build failure. Put the anatomy for all 22 themes in
front of Kenny as a review, because it is judgement, not data: 9 of the 36 choices here are guesses, each marked in
`src/anatomy.rs`. Leave tachyonfx out for now. It adds 29 % to the binary, and none of its built-ins except the fade
reproduces a house routine. The hand-written reveal is 41 pure lines that match `js/effects.js` rate for rate.
Reconsider tachyonfx when modals and toasts need enter and exit transitions, which the house has no routine for yet.
Migrate `tui-preview` after round one, keeping its effect level and semantic helpers.
