//! What a terminal can carry of a theme beyond colour: border glyphs, text
//! modifiers, case, prefixes, the cursor and the reveal routine.
//!
//! Hand-written, not generated: `tokens.json` has no field for any of this.
//! Each choice cites where it comes from; `GUESS` marks a choice no source
//! states, so a reviewer knows which ones to judge.

use crossterm::cursor::SetCursorStyle;
use ratatui::{style::Modifier, symbols::border};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Reveal {
    /// Fade in whole, no character touched. formal `arrive`.
    Arrive { ms: u32 },
    /// Characters churn through glyphs, then settle left to right.
    /// cyberpunk `decipher`.
    Decipher { cps: f32, lead_ms: u32, swap: f32 },
    /// Typed one glyph at a time with a block caret on the last.
    /// terminal `type`.
    Type { cps: f32 },
}

/// How a button is drawn. The web registers do not agree on a button being
/// a framed box: cyberpunk paints a plate with a 2px frame, a slit through
/// both sides and a cut corner, and terminal paints a bare plate in
/// brackets. A one-cell line around a label is formal's button, not
/// everyone's (Kenny, 2026-09-17: the buttons were "wat zwak", and he asked
/// whether their geometry itself could carry the theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonFace {
    /// A one-cell line in the theme's border glyphs, around the face.
    Line,
    /// A solid frame one cell thick around the face. `slit` interrupts both
    /// side bars at mid-height, `notch` cuts the bottom-right corner on the
    /// diagonal — the two marks cyberpunk's `.kp-button` carries.
    Slab { slit: bool, notch: bool },
    /// The plate alone, no frame: the label sits on the colour.
    Plate,
}

#[derive(Clone, Copy, Debug)]
pub struct Anatomy {
    pub border: border::Set<'static>,
    pub border_focus: border::Set<'static>,
    /// The button's own frame, used by `ButtonFace::Line`.
    pub button_border: border::Set<'static>,
    /// How the button is built, beyond its colours.
    pub button_face: ButtonFace,
    /// A space between the label's characters, for a register that sets
    /// `letter-spacing` on its buttons.
    pub button_spaced: bool,
    pub title_modifier: Modifier,
    pub uppercase_labels: bool,
    /// Before a microlabel (a panel title).
    pub label_prefix: &'static str,
    /// Around a button label.
    pub button_brackets: (&'static str, &'static str),
    /// Added to a focused button's label.
    pub focus_modifier: Modifier,
    pub tab_divider: &'static str,
    pub selected_tab_modifier: Modifier,
    pub cursor: SetCursorStyle,
    pub reveal: Reveal,
}

/// cyberpunk's notch: the bottom-right corner cut on the diagonal
/// (`clip-path` on `.kp-button`, `css/cyberpunk-register.css`).
pub const NOTCHED: border::Set<'static> = border::Set { bottom_right: "◢", ..border::PLAIN };

/// formal. Paper and ink; restraint; one rule that doubles.
pub const FORMAL: Anatomy = Anatomy {
    // GUESS: `--radius: 0.375rem` is the only radius > 0 of the three; a
    // rounded corner glyph is the nearest a cell grid has. PLAIN is the
    // alternative if rounded reads as too soft for print.
    border: border::ROUNDED,
    // anatomy.md "The rule doubles under the pointer (gap-4)";
    // formal-register.css `.kp-button:focus-visible::after`.
    border_focus: border::DOUBLE,
    button_border: border::ROUNDED,
    // formal-register.css `.kp-button`: a 1px border, a radius, no plate
    // decoration. The restraint is the point, so this one stays a line.
    button_face: ButtonFace::Line,
    button_spaced: false,
    // GUESS: Fraunces headings cannot exist in a terminal; bold is the only
    // weight channel left.
    title_modifier: Modifier::BOLD,
    uppercase_labels: false,
    label_prefix: "",
    button_brackets: ("", ""),
    focus_modifier: Modifier::empty(),
    tab_divider: " │ ",
    // formal-register.css `.kp-tab[aria-selected]`: primary colour and a
    // primary border; the underline is the border a cell row can carry.
    selected_tab_modifier: Modifier::UNDERLINED,
    // GUESS: nothing states a caret; a steady bar is the quietest.
    cursor: SetCursorStyle::SteadyBar,
    // anatomy.md "The headline arrives whole"; register: opacity 450ms.
    reveal: Reveal::Arrive { ms: 450 },
};

/// cyberpunk. Signal yellow on a void; square or notched; a machine voice.
pub const CYBERPUNK: Anatomy = Anatomy {
    // anatomy.md "The radius is 0".
    border: border::PLAIN,
    // GUESS: the web focus is a ring in `--ring`; THICK is the heavier line
    // a terminal has.
    border_focus: border::THICK,
    button_border: NOTCHED,
    // cyberpunk-register.css `.kp-button`: the frame is the background
    // colour of the element with the face inset 2px on all sides, the slit
    // is a 10px band of ground through both ends at 50% height, and the
    // clip-path cuts `--kp-button-notch` (14px) off the bottom-right corner.
    // All three survive a cell grid.
    button_face: ButtonFace::Slab { slit: true, notch: true },
    // `letter-spacing: 0.12em` on an uppercase label.
    button_spaced: true,
    title_modifier: Modifier::BOLD,
    // anatomy.md "uppercase, spaced, prefixed"; register `.microlabel`.
    uppercase_labels: true,
    // cyberpunk-register.css `.microlabel::before { content: '/// ' }`.
    label_prefix: "/// ",
    button_brackets: ("", ""),
    focus_modifier: Modifier::BOLD,
    // cyberpunk-register.css `.kp-breadcrumb li + li::before { content: '//' }`.
    tab_divider: " // ",
    // `.kp-tab[aria-selected] { border-bottom: 2px solid var(--primary) }`.
    selected_tab_modifier: Modifier::BOLD,
    // GUESS: no caret is stated; a steady block reads as a HUD field.
    cursor: SetCursorStyle::SteadyBlock,
    // cyberpunk-register.css `--kp-decipher-cps: 26; --kp-decipher-lead:
    // 260ms; --kp-decipher-swap: 0.5`.
    reveal: Reveal::Decipher { cps: 26.0, lead_ms: 260, swap: 0.5 },
};

/// terminal. A phosphor CRT that accepts a terminal's constraints.
pub const TERMINAL: Anatomy = Anatomy {
    // anatomy.md "the panels are TUI panels"; `--radius: 0`.
    border: border::PLAIN,
    // GUESS: the register moves the border to `--foreground` on focus and
    // keeps the line; the colour does the work, the glyph stays plain.
    border_focus: border::PLAIN,
    button_border: border::PLAIN,
    // anatomy.md "the buttons are brackets and plates": a plate, and the
    // brackets below are the button's marks. A line around it would be a
    // second frame the register does not draw.
    button_face: ButtonFace::Plate,
    button_spaced: false,
    // GUESS: the bloom on titles cannot exist; bold is the brightening a
    // terminal offers.
    title_modifier: Modifier::BOLD,
    // terminal-register.css `.kp-tab { text-transform: uppercase }`.
    uppercase_labels: true,
    // terminal-register.css `content: '$ '` before a microlabel.
    label_prefix: "$ ",
    // anatomy.md "the buttons are brackets and plates"; register `'[ '`, `' ]'`.
    button_brackets: ("[ ", " ]"),
    // anatomy.md "Every hover is inverse video" (hover, applied to focus
    // here because a keyboard TUI has no pointer: GUESS).
    focus_modifier: Modifier::REVERSED,
    tab_divider: "  ",
    // `.kp-tab[aria-selected] { background: var(--primary) }`: a plate.
    selected_tab_modifier: Modifier::empty(),
    // anatomy.md "a block of one character cell ... blinking once a second".
    cursor: SetCursorStyle::BlinkingBlock,
    // terminal-register.css `--kp-decipher-cps: 29.41`, the `type` routine.
    reveal: Reveal::Type { cps: 29.41 },
};
