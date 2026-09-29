//! The generated palette, included verbatim.
//!
//! `generated_palette.rs` is written by `gates/generate-tui-palette.mjs`
//! from the kp-themes tokens in `vendor/kp-themes/`. It is pulled in with
//! `include!` rather than declared as a module, because `cargo fmt` walks
//! modules and would realign the generator's columns; an included file it
//! leaves alone, and `--check` stays byte-exact. Moving to a newer kp-themes
//! is two commands, both in `../../../README.md`.

include!("generated_palette.rs");
