# kp-themes-tui

Was kp-tui until 2026-09-29; the crates keep their names.

The kp-themes house themes in a terminal: twenty-two palettes, the anatomy
a terminal can carry beyond colour, and ratatui widgets that take a theme
and name no colour of their own.

Two crates, and the split is the point:

| Crate             | What                                                                                   | Written by             |
| ----------------- | -------------------------------------------------------------------------------------- | ---------------------- |
| `kp-tui-palette`  | The 22 palettes, 36 colours each, with the role every colour plays                     | generated here         |
| `kp-tui`          | The anatomy per theme (border glyphs, case, prefixes, cursor, reveal) and the widgets  | by hand, with sources  |
| `kp-compare`      | The proof's own tool: homelab's screens beside the rebuilds, not published             | by hand                |

`crates/kp-tui-palette/src/generated_palette.rs` is **generated, not
authored**: `gates/generate-tui-palette.mjs` writes it from the kp-themes
tokens in `vendor/kp-themes/`, the unpacked `tokens.tar` of the kp-themes
release `vendor/PIN` names, with its sha256. The file is committed, so a
binary built from this workspace needs no node and no network; node runs only
when the pin moves and in the gates. Until kp-themes 8.0.0 the generator lived
in kp-themes and this repository vendored its output; it moved here with its
history.

`.claude/hooks/gates.sh` and CI run `node gates/check-vendor.mjs`, which
rebuilds the tar from the copy and compares it with the pin, and
`node gates/generate-tui-palette.mjs --check`, which refuses a palette that
drifted from the tokens. The file is pulled into the crate with `include!`
rather than as a module, so `cargo fmt` cannot realign the generator's
columns.

Moving to a newer kp-themes is two commands and a review:

```sh
scripts/vendor-kp-themes.sh 8.1.0          # download, verify, unpack, rewrite vendor/PIN
node gates/generate-tui-palette.mjs        # then read the diff and commit
```

## Where this comes from

The research came from kp-themes and is here now, in `research/ratatui/`: `README.md` measured what a
terminal can carry of a theme, and `ANATOMY_PROPOSAL.md`
worked out the anatomy of the nineteen themes that had none, citing the
register line behind every field and marking every guess. Kenny took that
research as this project's scope on 2026-09-17 rather than opening a fresh
Phase 0, because the research had already answered the questions a scope
round would ask, with measurements.

`research/ratatui/demo` in kp-themes is the working demo the widgets come
from: a live dashboard reading `/proc` and `journalctl`, plus a component
screen, in all 22 themes.

## What is in it

`kp-tui-palette`: the 22 palettes, `Palette<C>` with 36 colours, the `Role`
each colour plays, and `THEMES` in the package's order.

`kp-tui`: the `Anatomy` of every theme (border set, focus set, button face,
case, panel-title prefix, tab divider, cursor, reveal), `ColorDepth` and its
fallback to 256 and 16 colours, `Theme` (a palette resolved for the terminal
plus its anatomy), and the widgets:

- **Panel** — a titled frame in the theme's border glyphs, with its prefix.
- **ThemedTabs** — the tab strip and the theme's own divider.
- **Button** — a filled plate, the plate's end from the theme's radius, the
  focus ring along its last row, the charge sweep where a register has one.
- **RevealText** — a headline in the theme's own routine: whole, deciphered,
  typed, or word by word with its measured stagger.
- **dashboard** — charts over a window, a bar chart, sparklines, the log
  pane with a colour per severity, the threshold pulse.
- **logs** — `journalctl --output=json` parsed into coloured parts, a buffer
  with pause, filter and scroll.
- **live** — CPU, memory, load, network and disk read from `/proc`.

## Status

Started 2026-09-17, first release `0.1.0` on 2026-09-20, latest `0.1.1` on
2026-09-26 ([CHANGELOG.md](CHANGELOG.md)), pinned to the kp-themes 7.1.0
palette. The crates build, 80 tests pass, clippy is clean, and CI runs fmt,
clippy and the suite on every push.

The demo draws eleven screens — `dashboard`, `components`, `console`,
`effects`, `fleet`, `ops`, `settings`, `logs`, `deploy`, `doctor`,
`splash` — of which the last seven are homelab's own screens rebuilt on
nothing but this crate, measured and put side by side with homelab's
client in [docs/HOMELAB_PROOF.md](docs/HOMELAB_PROOF.md). Eight of
homelab's nine are rebuilt; the shell tab is a terminal inside a terminal
and is left alone. Every rebuilt screen answers to homelab's own keys —
the palette, the help, the plan, the restore confirm, the wizard — and a
test presses each of them.

Twenty-five design directions, five per rebuilt screen, were drawn on the
way there. Kenny picked one per screen on 2026-09-19 and unpicked one of
them the next day, so four of the five are what the rebuilt screens draw —
card grid, grouped cards with a diff under them, a density band, and two
panes — and the dashboard is homelab's own layout again. The twenty that
were not chosen are kept as pictures in
[docs/archive/ontwerpen-2026-09-19.html](docs/archive/ontwerpen-2026-09-19.html)
rather than as code nothing calls.
