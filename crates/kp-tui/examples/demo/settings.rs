//! homelab's settings tab, rebuilt on this crate — the third proof.
//!
//! The original is `client/src/tui/view/settings.rs`, 116 lines: the host's
//! runtime configuration, edited over the TLS line. It was chosen because it
//! leans on nothing the first two screens leaned on — no table, no meter, no
//! panel grid — but on fields, a row you step through, help text under every
//! setting, an inline text edit and a dirty marker.
//!
//! It found two gaps on the way [docs/HOMELAB_PROOF.md]: a value that is
//! stepped through rather than typed, and a state dot that only knew up
//! from down. Both are in the crate now, so nothing here is marked `GAP`
//! and no colour on this screen is chosen by hand.
//!
//! Kenny asked whether two of the five directions could go together —
//! **Grouped cards** and **Diff** — so this is both: the settings live in
//! three cards, and under them stands what the host runs today beside what
//! S would make of it. Every label is padded to one column, so no value
//! sits against the word in front of it [fix-64].

use crossterm::event::KeyCode;
use kp_tui::{
    Badge, Choice, Field, KeyHints, Stage, Theme, Tone, fx::Motion, label_column, widgets::Panel,
};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Tier {
    pub every_days: u32,
    pub span_days: Option<u32>,
}

/// The steps homelab offers for a tier's interval and its span, in its
/// own order (`client/src/tui/model.rs`, `EVERY_PRESETS`, `SPAN_PRESETS`).
const EVERY_PRESETS: &[u32] = &[1, 2, 3, 7, 14, 21, 30, 45, 60, 90, 120, 180];
const SPAN_PRESETS: &[u32] = &[7, 14, 21, 30, 60, 90, 120, 180, 365, 730];

/// The host's configuration as this screen edits it: what the host runs,
/// what the operator has made of it, and the webhook while it is typed.
/// homelab keeps the same three things on its model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub backup_hour: Option<u32>,
    pub tiers: Vec<Tier>,
    pub webhook: Option<String>,
}

pub struct Settings {
    /// What the host answered last; `r` reads it back, `S` replaces it.
    pub host: Config,
    /// What the screen shows and the arrows change.
    pub edit: Config,
    /// The row the arrows are on: the hour, two per tier, then the hook.
    pub row: usize,
    /// The webhook while it is being typed, which swallows every key.
    pub typing: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        let host = Config {
            backup_hour: Some(4),
            tiers: vec![
                Tier {
                    every_days: 1,
                    span_days: Some(7),
                },
                Tier {
                    every_days: 14,
                    span_days: Some(90),
                },
                Tier {
                    every_days: 60,
                    span_days: None,
                },
            ],
            webhook: None,
        };
        // Two edits already made, so the diff has something to show when
        // the screen opens, as it always did.
        let mut edit = host.clone();
        edit.backup_hour = Some(3);
        edit.tiers[1].span_days = Some(60);
        Settings {
            host,
            edit,
            row: 0,
            typing: None,
        }
    }
}

impl Settings {
    /// The rows the arrow keys walk: the hour, then two per tier, then the hook.
    pub fn rows(&self) -> usize {
        2 + self.edit.tiers.len() * 2
    }

    fn webhook_row(&self) -> usize {
        self.rows() - 1
    }

    /// One key, the way homelab's `settings_key` and
    /// `settings_webhook_edit_key` read it. Returns what the status line
    /// should say, if anything.
    pub fn key(&mut self, code: KeyCode) -> Option<String> {
        if let Some(buf) = self.typing.as_mut() {
            match code {
                KeyCode::Esc => self.typing = None,
                KeyCode::Enter => {
                    let text = buf.trim().to_string();
                    self.edit.webhook = if text.is_empty() { None } else { Some(text) };
                    self.typing = None;
                }
                KeyCode::Backspace => {
                    buf.pop();
                }
                KeyCode::Char(c) => buf.push(c),
                _ => {}
            }
            return None;
        }
        let row = self.row;
        let hook = self.webhook_row();
        match code {
            KeyCode::Up | KeyCode::Char('k') => self.row = row.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => self.row = (row + 1).min(hook),
            KeyCode::Left | KeyCode::Right => {
                let dir: i32 = if code == KeyCode::Left { -1 } else { 1 };
                if row == 0 {
                    self.edit.backup_hour = match (self.edit.backup_hour, dir) {
                        (None, 1) => Some(0),
                        (None, _) => Some(23),
                        (Some(0), -1) => None,
                        (Some(23), 1) => None,
                        (Some(h), 1) => Some(h + 1),
                        (Some(h), _) => Some(h - 1),
                    };
                } else if row < hook {
                    let t = &mut self.edit.tiers[(row - 1) / 2];
                    if (row - 1).is_multiple_of(2) {
                        t.every_days = step(EVERY_PRESETS, t.every_days, dir);
                    } else {
                        let top = *SPAN_PRESETS.last().unwrap_or(&730);
                        t.span_days = match (t.span_days, dir) {
                            (None, -1) => Some(top),
                            (None, _) => None,
                            (Some(v), 1) if v >= top => None,
                            (Some(v), d) => Some(step(SPAN_PRESETS, v, d)),
                        };
                    }
                }
            }
            KeyCode::Char('a') => {
                // A new tier goes in before the one that is kept forever.
                let at = self
                    .edit
                    .tiers
                    .iter()
                    .position(|t| t.span_days.is_none())
                    .unwrap_or(self.edit.tiers.len());
                self.edit.tiers.insert(
                    at,
                    Tier {
                        every_days: 30,
                        span_days: Some(90),
                    },
                );
            }
            KeyCode::Char('d') => {
                if row >= 1 && row < hook && self.edit.tiers.len() > 1 {
                    self.edit.tiers.remove((row - 1) / 2);
                    self.row = self.row.min(self.webhook_row());
                }
            }
            KeyCode::Enter if row == hook => {
                self.typing = Some(self.edit.webhook.clone().unwrap_or_default());
            }
            KeyCode::Char('S') => {
                self.host = self.edit.clone();
                return Some("settings sent to host".into());
            }
            KeyCode::Char('r') => {
                self.edit = self.host.clone();
                self.row = self.row.min(self.webhook_row());
                return Some("host settings reloaded".into());
            }
            _ => {}
        }
        None
    }

    /// Whether this screen is holding the keyboard for text.
    pub fn typing(&self) -> bool {
        self.typing.is_some()
    }
}

fn step(presets: &[u32], current: u32, dir: i32) -> u32 {
    let pos = presets.iter().position(|p| *p >= current).unwrap_or(0);
    let next = (pos as i32 + dir).clamp(0, presets.len() as i32 - 1) as usize;
    presets[next]
}

/// What a tier is called in the card, by its interval; homelab numbers
/// them, and a number says less than "daily" does.
pub fn tier_name(t: &Tier, i: usize) -> String {
    match t.every_days {
        1 => "daily".into(),
        7 => "weekly".into(),
        14 => "fortnightly".into(),
        30 => "monthly".into(),
        60 => "bimonthly".into(),
        90 => "quarterly".into(),
        _ => format!("tier {}", i + 1),
    }
}

pub const KEYS: [(&str, &str); 7] = [
    ("↑↓", "field"),
    ("←→", "value"),
    ("a / d", "add / delete tier"),
    ("enter", "edit webhook"),
    ("S", "save"),
    ("r", "reload"),
    ("q", "quit"),
];

pub fn draw(frame: &mut Frame, th: &Theme, st: &Settings, reveal_ms: u32, motion: Motion) {
    let screen = frame.area();
    let stage = Stage::new(reveal_ms, motion);
    let [cards, diff, footer] = Layout::vertical([
        Constraint::Length(8),
        Constraint::Min(6),
        Constraint::Length(1),
    ])
    .areas(screen);

    // Every label on the screen, so one column serves all three cards: a
    // value may not start where its own label happens to end [fix-64].
    let names: Vec<String> = st
        .edit
        .tiers
        .iter()
        .enumerate()
        .map(|(i, t)| tier_name(t, i))
        .collect();
    let mut labels = vec!["nightly run", "window", "on failure"];
    labels.extend(names.iter().map(String::as_str));
    let column = label_column(th, &labels) + 2;
    let cols = Layout::horizontal([Constraint::Ratio(1, 3); 3]).split(cards);
    draw_schedule(frame, th, cols[0], st, column, stage, reveal_ms, motion);
    draw_retention(frame, th, cols[1], st, column, stage, reveal_ms, motion);
    draw_notify(frame, th, cols[2], st, column, stage, reveal_ms, motion);
    draw_diff(frame, th, diff, st, stage, reveal_ms, motion);

    Paragraph::new(KeyHints::new(th, &KEYS).footer())
        .style(Style::new().bg(th.c.background))
        .render(footer, frame.buffer_mut());
}

/// The rows of one card, drawn on the card's own plate.
fn put(frame: &mut Frame, th: &Theme, inner: Rect, mut lines: Vec<Line<'static>>) {
    // A card with more tiers than rows gives up its help text first.
    lines.truncate(inner.height as usize);
    let areas = Layout::vertical(vec![Constraint::Length(1); lines.len().max(1)]).split(inner);
    for (n, line) in lines.into_iter().enumerate() {
        Paragraph::new(line)
            .style(Style::new().fg(th.c.card_foreground).bg(th.c.card))
            .render(areas[n], frame.buffer_mut());
    }
}

fn help(th: &Theme, text: &str) -> Line<'static> {
    Line::from(Span::styled(
        text.to_string(),
        Style::new().fg(th.c.muted_foreground),
    ))
}

#[allow(clippy::too_many_arguments)]
fn draw_schedule(
    frame: &mut Frame,
    th: &Theme,
    area: Rect,
    st: &Settings,
    column: usize,
    stage: Stage,
    reveal_ms: u32,
    motion: Motion,
) {
    let selected = st.row;
    let panel = Panel::new(th, "Schedule")
        .focused(selected == 0)
        .stage(stage)
        .reveal(reveal_ms, motion);
    let inner = panel.block().inner(area);
    frame.render_widget(panel, area);

    let hour = match st.edit.backup_hour {
        Some(h) => format!("{h:02}:00"),
        None => "off".into(),
    };
    let nightly = Choice::new(th, "nightly run", &hour).focused(selected == 0);
    let window = Choice::new(th, "window", "2 hours");
    put(
        frame,
        th,
        inner,
        vec![
            Line::from([nightly.label_spans(column), nightly.value_spans()].concat()),
            Line::from([window.label_spans(column), window.value_spans()].concat()),
            Line::default(),
            help(th, "backup and auto-updates"),
            help(th, "for every managed stack"),
            help(th, "at this hour"),
        ],
    );
}

#[allow(clippy::too_many_arguments)]
fn draw_retention(
    frame: &mut Frame,
    th: &Theme,
    area: Rect,
    st: &Settings,
    column: usize,
    stage: Stage,
    reveal_ms: u32,
    motion: Motion,
) {
    let selected = st.row;
    let focused = (1..=st.edit.tiers.len() * 2).contains(&selected);
    let panel = Panel::new(th, "Retention")
        .focused(focused)
        .stage(stage)
        .reveal(reveal_ms, motion);
    let inner = panel.block().inner(area);
    frame.render_widget(panel, area);

    let mut lines = Vec::new();
    for (i, tier) in st.edit.tiers.iter().enumerate() {
        // Short in the card, spelled out in the diff under it: a card
        // this narrow cannot carry both words and both steppers.
        //
        // Both values are padded to the width of the longest of their own
        // kind, so the second stepper starts in the same column on every
        // tier — a 14 may not push it one cell further than a 1 [fix-64].
        let every = format!("{:>3}", format!("{}d", tier.every_days));
        let span = format!(
            "{:<7}",
            match tier.span_days {
                Some(d) => format!("for {d}d"),
                None => "always".into(),
            }
        );
        let name = tier_name(tier, i);
        let left = Choice::new(th, &name, &every).focused(selected == 1 + i * 2);
        let right = Choice::new(th, "", &span).focused(selected == 2 + i * 2);
        lines.push(Line::from(
            [
                left.label_spans(column),
                left.value_spans(),
                vec![Span::raw("  ")],
                right.value_spans(),
            ]
            .concat(),
        ));
    }
    lines.push(Line::default());
    lines.push(help(th, "one snapshot per interval,"));
    lines.push(help(th, "kept for the span beside it"));
    put(frame, th, inner, lines);
}

#[allow(clippy::too_many_arguments)]
fn draw_notify(
    frame: &mut Frame,
    th: &Theme,
    area: Rect,
    st: &Settings,
    column: usize,
    stage: Stage,
    reveal_ms: u32,
    motion: Motion,
) {
    let selected = st.row;
    let hook_row = st.rows() - 1;
    let panel = Panel::new(th, "Notify")
        .focused(selected == hook_row)
        .stage(stage)
        .reveal(reveal_ms, motion);
    let inner = panel.block().inner(area);
    frame.render_widget(panel, area);

    // The one row that is typed into rather than stepped through: the
    // theme's own caret, blinking at the rate its register states.
    let shown = match (&st.typing, &st.edit.webhook) {
        (Some(typed), _) => typed.clone(),
        (None, Some(url)) => url.clone(),
        (None, None) => "off — enter to set".into(),
    };
    let field = Field::new(th, "webhook", &shown)
        .focused(selected == hook_row)
        .label_width(column)
        .blink(reveal_ms / 16);
    let on_failure = Choice::new(th, "on failure", "always");
    put(
        frame,
        th,
        inner,
        vec![
            Line::from(field.spans()),
            Line::from([on_failure.label_spans(column), on_failure.value_spans()].concat()),
            Line::default(),
            help(th, "one POST per finished"),
            help(th, "operation, {op, ok, error}"),
        ],
    );
}

/// What the host runs today beside what saving would make of it: the
/// second half of Kenny's question, and the reason the dirty dot can say
/// how many rows it stands for.
fn draw_diff(
    frame: &mut Frame,
    th: &Theme,
    area: Rect,
    st: &Settings,
    stage: Stage,
    reveal_ms: u32,
    motion: Motion,
) {
    let rows = diff_rows(&st.host, &st.edit);
    let changed = rows.iter().filter(|(_, h, w)| h != w).count();
    let title = match changed {
        0 => "In sync with the host".to_string(),
        1 => "1 unsaved change".to_string(),
        n => format!("{n} unsaved changes"),
    };
    let panel = Panel::new(th, &title)
        .stage(stage)
        .reveal(reveal_ms, motion);
    let inner = panel.block().inner(area);
    frame.render_widget(panel, area);

    // Three columns at fixed starts: the setting, the host, the edit.
    let names: Vec<&str> = rows.iter().map(|(n, _, _)| n.as_str()).collect();
    let name_w = label_column(th, &names) + 2;
    let host_w = rows
        .iter()
        .map(|(_, v, _)| v.chars().count())
        .max()
        .unwrap_or(0)
        + 2;
    let mut lines = vec![Line::from(vec![
        Span::styled(
            format!("{:<name_w$}", ""),
            Style::new().fg(th.c.muted_foreground),
        ),
        Span::styled(
            format!("{:<host_w$}", "on the host"),
            Style::new().fg(th.c.muted_foreground),
        ),
        Span::styled(
            "after S".to_string(),
            Style::new().fg(th.c.muted_foreground),
        ),
    ])];
    for (name, host, want) in rows.iter() {
        let differs = host != want;
        lines.push(Line::from(vec![
            Span::styled(
                format!("{name:<name_w$}"),
                Style::new().fg(th.c.card_foreground),
            ),
            Span::styled(
                format!("{host:<host_w$}"),
                Style::new().fg(th.c.muted_foreground),
            ),
            Span::styled(
                if differs {
                    want.clone()
                } else {
                    "—".to_string()
                },
                if differs {
                    Style::new()
                        .fg(th.ink(Tone::Warning, th.id.palette().card))
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::new().fg(th.c.muted_foreground)
                },
            ),
        ]));
    }
    lines.push(Line::default());

    // The dirty marker: a dot in the tone the state deserves, and the word
    // beside it in an ink that reads on this surface.
    let (tone, says) = if changed > 0 {
        (Tone::Warning, "S applies them; R reloads the host's values")
    } else {
        (Tone::Success, "nothing to apply")
    };
    lines.push(Line::from(vec![
        Badge::state(th, tone),
        Span::raw(" "),
        Span::styled(
            says.to_string(),
            Style::new().fg(th.ink(tone, th.id.palette().card)),
        ),
    ]));
    put(frame, th, inner, lines);
}

/// The diff's rows: the setting, what the host runs, what S would send.
/// Tiers are paired by position, so one that exists on one side only
/// reads as added or removed rather than shifting every row under it.
fn diff_rows(host: &Config, edit: &Config) -> Vec<(String, String, String)> {
    let hour = |h: Option<u32>| match h {
        Some(h) => format!("{h:02}:00"),
        None => "off".into(),
    };
    let mut rows = vec![(
        "nightly run".to_string(),
        hour(host.backup_hour),
        hour(edit.backup_hour),
    )];
    for i in 0..host.tiers.len().max(edit.tiers.len()) {
        let (h, e) = (host.tiers.get(i), edit.tiers.get(i));
        let name = e.or(h).map(|t| tier_name(t, i)).unwrap_or_default();
        rows.push((
            format!("keep {name}"),
            h.map(tier_words).unwrap_or_else(|| "—".into()),
            e.map(tier_words).unwrap_or_else(|| "removed".into()),
        ));
    }
    let hook = |w: &Option<String>| w.clone().unwrap_or_else(|| "off".into());
    rows.push(("webhook".into(), hook(&host.webhook), hook(&edit.webhook)));
    rows
}

fn tier_words(t: &Tier) -> String {
    match t.span_days {
        Some(d) => format!("every {}d for {d} days", t.every_days),
        None => format!("every {}d forever", t.every_days),
    }
}
