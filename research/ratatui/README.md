# kp-themes as Ratatui components that switch theme at runtime

Kenny, 2026-09-17: "Eerst een Ratatui-onderzoek". Build a demo app with three themes and four widgets to show the
shape before anything is fixed.

Demo: [`demo/`](demo/), a standalone Cargo project (ratatui 0.30.2, crossterm 0.29). This topic has no `demo.html`
because the deliverable is a terminal app. `cargo run` starts it on the live dashboard (see [Dashboard](#dashboard));
`s` switches to the components screen. `cargo run --example snapshot [-- 256|16] [dashboard]` prints one finished
frame per theme. `cargo test` runs the checks, and `--features tachyonfx` swaps the reveal for the tachyonfx version.
Keys on the components screen: `t` theme, `←/→` tab, `Tab` focus, `Enter` press, `r` replay, `m` motion, `q` quit.

**Measured** (2026-09-17, 16 cores, target directory outside the repository):

- **Build:** `cargo build` succeeds. A clean `--release` build takes 6.22 s. Clippy and `cargo fmt --check` are clean
  with and without the feature.
- **Tests:** 12 pass (4 parity, 8 render), or 13 with `--features tachyonfx`. With the dashboard: 22 (10 more), 23
  with the feature.
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

## Dashboard

Kenny, 2026-09-17, after running the first screen: "wel maar wat mager". The binary now opens on a live dashboard
drawn through the same `&Theme`, so `t` repaints every chart, bar and log line on the next frame. Run it with
`cargo run --release` (flags: `--fps N`, `--synthetic-logs`, `--reduced-motion`, `--screen components`).

- **Machine stats, real** (`src/live.rs`): per-core and total CPU from `/proc/stat` (idle + iowait counted as idle),
  memory from `MemAvailable`, load average, network rx/tx over every interface except `lo`, and disk read/write over
  physical block devices only (partitions, zram, loop and dm are left out, so nothing is counted twice). Sampled
  every 500 ms. **`/proc` directly, not `sysinfo`:** on Linux it reads the same files, its disk I/O is per mounted
  filesystem (so it would need de-duplicating), and a probe with the same readings (sysinfo 0.39.6) was 167,840
  bytes larger than an empty binary. Pick `sysinfo` when a consumer runs on macOS or Windows.
- **Charts:** `Chart` with braille lines over a 60 s window: CPU total (`--chart-1`) plus load per core
  (`--chart-5`), and network received (`--chart-3`) and sent (`--chart-4`) on a rounded auto-scale. Also a
  `BarChart` of the cores (`--chart-1`, `--warning-foreground` from 70 %, `--destructive` from 90 %) and
  `Sparkline`s for memory (`--chart-2`) and disk. Axis lines use `--border-strong` and labels `--muted-foreground`.
  The palette gained six generated fields for this (`chart-1` to `chart-5`, `warning`), 35 per theme now.
- **Logs:** `journalctl --follow --output=json` runs as a read-only child. On this machine it was readable without
  privileges, starting with the last 300 lines. If it cannot start, or ends without a line, the pane switches to a
  generated stream titled "synthetic, not real". The journal here writes about 20 lines a minute (1,181 in the last
  hour), so `--synthetic-logs` shows a busier, labelled stream. Colours: timestamp `--muted-foreground`, host
  `--border-strong`, unit `--primary`. Debug `--muted-foreground`, info `--info-foreground`, notice
  `--success-foreground`, warning `--warning-foreground` in bold, error `--destructive` in bold, and critical bold on
  a `--destructive` plate. Every line carries a fixed-width text tag (`DEBUG`…`CRIT`), because at 16 colours
  terminal's info and success foregrounds both land on green. Keys: `p` pause (the view pins, and the title counts
  new lines), `f` filter (all, then info and up, through critical, then back), `↑/↓ PgUp/PgDn Home` scroll (this
  pins too), `End` follow. Control characters in messages become spaces.
- **Effects:** panel titles reveal at start in the theme's own routine, 90 ms apart. CPU (70 %), memory (85 %) and
  load (one per core) pulse when they cross upwards: two fading beats over 1.2 s on the soft `--warning` plate, or
  inverse video at 16 colours. Over the threshold the value stays in `--warning-foreground`. Reduced motion shows
  whole titles and no pulse.
- **Frame budget:** draws at a steady 15 fps (`--fps`), sleeping in `event::poll` until the next frame or sample.
  In a 140×45 pty, release build, 60 s: 901 frames, a mean draw of 0.34 ms, and **0.57 % of one core** for the demo
  itself (0.34 s CPU, from its own `/proc/self/stat`; bash `time` over the demo and journalctl gives 0.320 s user
  and 0.041 s sys). journalctl used under one clock tick. `--fps 60` costs 2.07 %. The run wrote 4,281 bytes a second,
  with no full-screen clear (`\e[2J` 0 times): ratatui's buffer diff sends only changed cells, so nothing flickers.
- **Size and tests:** stripped release binary 1,132,472 bytes, up 340,840 (+43 %) on the first screen. The added
  normal dependency is `serde_json`, already a build dependency; it brings `itoa`, `memchr`, `serde_core` and `zmij`.
  `tests/dashboard.rs` holds 10 headless tests. In every theme they check the `100%`, `-60s` and `now` labels in
  `--muted-foreground`, the CPU and received series colours, and each severity's colour plus the timestamp, host and
  unit roles on a log line. They also check that `t` changes the CPU series from formal's `--chart-1` to
  cyberpunk's, that 16 colours leave no RGB and keep every tag, and that pause, filter and scroll behave. The rest
  cover the pulse with and without motion, the title reveal, and the `/proc` and journal parsers on fixtures. None
  reads `/proc` or spawns a process.

## Recommendation

Build the two crates in the next round, starting from this demo's shape. The split that works is the generated
palette, the hand-written anatomy, and widgets that take `&Theme` and name nothing. Switching then costs one struct
rebuild before the next frame, and the parity test makes drift a build failure. Put the anatomy for all 22 themes in
front of Kenny as a review, because it is judgement, not data: 9 of the 36 choices here are guesses, each marked in
`src/anatomy.rs`. Leave tachyonfx out for now. It adds 29 % to the binary, and none of its built-ins except the fade
reproduces a house routine. The hand-written reveal is 41 pure lines that match `js/effects.js` rate for rate.
Reconsider tachyonfx when modals and toasts need enter and exit transitions, which the house has no routine for yet.
Migrate `tui-preview` after round one, keeping its effect level and semantic helpers.
