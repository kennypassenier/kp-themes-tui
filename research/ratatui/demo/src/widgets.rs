//! The four widgets of the demo. Each takes `&Theme` and nothing else about
//! looks: no colour, glyph or modifier is named here.

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs, Widget},
};

use crate::anatomy::Reveal;
use crate::color::{ColorDepth, Rgb, Role};
use crate::fx::{self, Motion};
use crate::theme::Theme;

// ── Panel ───────────────────────────────────────────────────────────────

/// A titled frame: `.kp-card` in a cell grid.
pub struct Panel<'a> {
    theme: &'a Theme,
    title: &'a str,
    focused: bool,
    /// Reveal the title in the theme's routine: elapsed ms and motion.
    reveal: Option<(u32, Motion)>,
    /// A right-aligned note in the top border (the log pane's state).
    status: Option<Line<'static>>,
}

impl<'a> Panel<'a> {
    pub fn new(theme: &'a Theme, title: &'a str) -> Self {
        Panel { theme, title, focused: false, reveal: None, status: None }
    }

    pub fn reveal(mut self, elapsed_ms: u32, motion: Motion) -> Self {
        self.reveal = Some((elapsed_ms, motion));
        self
    }

    pub fn status(mut self, status: Line<'static>) -> Self {
        self.status = Some(status);
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// The block, for wrapping other content the way `Block` is used.
    pub fn block(&self) -> Block<'static> {
        let (t, a) = (&self.theme.c, self.theme.a);
        let (set, line) = if self.focused { (a.border_focus, t.ring) } else { (a.border, t.border_strong) };
        let title_style = Style::new()
            .fg(if self.focused { t.foreground } else { t.muted_foreground })
            .add_modifier(a.title_modifier);
        let label = self.theme.label(self.title);
        let title = match self.reveal {
            None => Line::from(Span::styled(format!(" {label} "), title_style)),
            Some((elapsed, motion)) => {
                let p = self.theme.id.palette();
                let ink = if self.focused { p.foreground } else { p.muted_foreground };
                let mut spans = vec![Span::styled(" ", title_style)];
                spans.extend(reveal_spans(self.theme, &label, elapsed, motion, ink, title_style));
                spans.push(Span::styled(" ", title_style));
                Line::from(spans)
            }
        };
        let mut block = Block::new()
            .borders(Borders::ALL)
            .border_set(set)
            .border_style(Style::new().fg(line))
            .title(title)
            .style(Style::new().bg(t.card).fg(t.card_foreground));
        if let Some(status) = &self.status {
            block = block.title_top(status.clone().right_aligned());
        }
        block
    }
}

impl Widget for Panel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.block().render(area, buf);
    }
}

// ── Tabs ────────────────────────────────────────────────────────────────

pub struct ThemedTabs<'a> {
    theme: &'a Theme,
    titles: &'a [&'a str],
    selected: usize,
}

impl<'a> ThemedTabs<'a> {
    pub fn new(theme: &'a Theme, titles: &'a [&'a str], selected: usize) -> Self {
        ThemedTabs { theme, titles, selected }
    }
}

impl Widget for ThemedTabs<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (t, a) = (&self.theme.c, self.theme.a);
        let titles = self
            .titles
            .iter()
            .map(|s| if a.uppercase_labels { format!(" {} ", s.to_uppercase()) } else { format!(" {s} ") });
        // terminal selects with a plate (`--primary` ground); the others
        // with the primary as ink plus a line modifier.
        let plate = a.selected_tab_modifier.is_empty();
        let highlight = if plate {
            Style::new().bg(t.primary).fg(t.primary_foreground)
        } else {
            Style::new().fg(t.primary).add_modifier(a.selected_tab_modifier)
        };
        Tabs::new(titles)
            .select(self.selected)
            .divider(Span::styled(a.tab_divider, Style::new().fg(t.border_strong)))
            .padding("", "")
            .style(Style::new().bg(t.background).fg(t.muted_foreground))
            .highlight_style(highlight)
            .render(area, buf);
    }
}

// ── Button ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonState {
    Rest,
    Focus,
    Pressed,
    Disabled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonKind {
    Primary,
    Destructive,
}

/// Three rows: the theme's button frame around a plate.
pub struct Button<'a> {
    theme: &'a Theme,
    label: &'a str,
    kind: ButtonKind,
    state: ButtonState,
}

impl<'a> Button<'a> {
    pub fn new(theme: &'a Theme, label: &'a str) -> Self {
        Button { theme, label, kind: ButtonKind::Primary, state: ButtonState::Rest }
    }
    pub fn kind(mut self, kind: ButtonKind) -> Self {
        self.kind = kind;
        self
    }
    pub fn state(mut self, state: ButtonState) -> Self {
        self.state = state;
        self
    }

    /// Plate, ink and frame colour for the current state.
    pub fn colors(&self) -> (Color, Color, Color) {
        let t = &self.theme.c;
        match (self.kind, self.state) {
            (_, ButtonState::Disabled) => (t.muted, t.muted_foreground, t.border),
            (ButtonKind::Primary, ButtonState::Rest) => (t.primary, t.primary_foreground, t.primary),
            // Focus is the hover colour plus the ring, as `:focus-visible`
            // shares `:hover`'s plate in the registers.
            (ButtonKind::Primary, ButtonState::Focus) => (t.primary_hover, t.primary_foreground, t.ring),
            (ButtonKind::Primary, ButtonState::Pressed) => (t.primary_active, t.primary_foreground, t.ring),
            (ButtonKind::Destructive, ButtonState::Rest) => (t.destructive, t.destructive_foreground, t.destructive),
            (ButtonKind::Destructive, ButtonState::Focus) => (t.destructive, t.destructive_foreground, t.ring),
            (ButtonKind::Destructive, ButtonState::Pressed) => (t.secondary_active, t.destructive, t.ring),
        }
    }
}

impl Widget for Button<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let a = self.theme.a;
        let (plate, ink, frame) = self.colors();
        let set = match self.state {
            ButtonState::Focus | ButtonState::Pressed if a.border_focus != a.border => a.border_focus,
            _ => a.button_border,
        };
        let mut label_style = Style::new().fg(ink).bg(plate).add_modifier(Modifier::BOLD);
        match self.state {
            ButtonState::Focus => label_style = label_style.add_modifier(a.focus_modifier),
            ButtonState::Disabled => label_style = label_style.add_modifier(Modifier::DIM),
            _ => {}
        }
        let text = if a.uppercase_labels { self.label.to_uppercase() } else { self.label.to_string() };
        let label = format!("{}{}{}", a.button_brackets.0, text, a.button_brackets.1);
        let block = Block::new()
            .borders(Borders::ALL)
            .border_set(set)
            .border_style(Style::new().fg(frame).bg(plate))
            .style(Style::new().bg(plate));
        Paragraph::new(Line::from(Span::styled(label, label_style)))
            .alignment(Alignment::Center)
            .block(block)
            .render(area, buf);
    }
}

// ── Reveal ──────────────────────────────────────────────────────────────

/// A headline revealed in the theme's own routine.
pub struct RevealText<'a> {
    theme: &'a Theme,
    text: &'a str,
    elapsed_ms: u32,
    motion: Motion,
}

impl<'a> RevealText<'a> {
    pub fn new(theme: &'a Theme, text: &'a str, elapsed_ms: u32, motion: Motion) -> Self {
        RevealText { theme, text, elapsed_ms, motion }
    }
}

/// A text in the middle of its reveal, as spans: the routine's text, the
/// arrive fade as a colour mix from the card towards `ink`, and the typing
/// caret as a lit cell. Under reduced motion this is the finished text.
pub fn reveal_spans(
    th: &Theme,
    text: &str,
    elapsed_ms: u32,
    motion: Motion,
    ink: Rgb,
    style: Style,
) -> Vec<Span<'static>> {
    let f = fx::frame(text, th.a.reveal, elapsed_ms, motion);
    let n = f.text.chars().count();
    let mut spans = match (th.depth, th.a.reveal) {
        // 16 colours cannot blend: the fade is one step at half time.
        (ColorDepth::Ansi16, Reveal::Arrive { .. }) if f.opacity < 0.5 => vec![Span::styled(" ".repeat(n), style)],
        (_, Reveal::Arrive { .. }) if th.depth != ColorDepth::Ansi16 => {
            let fg = th.depth.resolve(Role::Ink, mix(th.id.palette().card, ink, f.opacity));
            vec![Span::styled(f.text, style.fg(fg))]
        }
        _ => vec![Span::styled(f.text, style)],
    };
    if f.caret.is_some() {
        spans.push(Span::styled(" ", Style::new().bg(th.c.foreground)));
        // Keep the title's width while it types, so the border does not jump.
        let rest = text.chars().count().saturating_sub(n + 1);
        spans.push(Span::styled(" ".repeat(rest), style));
    }
    spans
}

pub fn mix(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Rgb(l(a.0, b.0), l(a.1, b.1), l(a.2, b.2))
}

impl Widget for RevealText<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let th = self.theme;
        let f = fx::frame(self.text, th.a.reveal, self.elapsed_ms, self.motion);
        let p = th.id.palette();
        let (fg, text) = match (th.depth, th.a.reveal) {
            // 16 colours cannot blend: the fade becomes one step at half time.
            (ColorDepth::Ansi16, Reveal::Arrive { .. }) if f.opacity < 0.5 => {
                (th.c.card_foreground, " ".repeat(f.text.chars().count()))
            }
            (ColorDepth::Ansi16, _) => (th.c.card_foreground, f.text),
            (_, Reveal::Arrive { .. }) => {
                (th.depth.resolve(Role::Ink, mix(p.card, p.card_foreground, f.opacity)), f.text)
            }
            _ => (th.c.card_foreground, f.text),
        };
        let style = Style::new().fg(fg).bg(th.c.card).add_modifier(th.a.title_modifier);
        let mut spans = vec![Span::styled(text, style)];
        if f.caret.is_some() {
            spans.push(Span::styled(" ", Style::new().bg(th.c.foreground)));
        }
        Paragraph::new(Line::from(spans)).render(area, buf);
    }
}
