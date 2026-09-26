//! What homelab draws over a tab rather than in it: the help overlay, the
//! typed confirmation a restore asks for, the change plan a deploy shows
//! before it runs, the new-stack wizard and the status line. The command
//! palette is the crate's own widget; its actions are homelab's.
//!
//! Each of these is homelab's behaviour key for key, read from
//! `client/src/tui/model.rs` (`on_key`, `confirm_key`, `open_plan`,
//! `wizard_key`, `PALETTE`), because a rebuilt screen does everything the
//! original did [fix-65, fix-68]. What only a live host can do — send the
//! restore, scaffold the files — ends in the status line saying so.

use crossterm::event::KeyCode;
use kp_tui::{Choice, Field, KeyHints, Popup, PopupKind, Stepper, Theme, Tone};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::fleet::FLEET;

/// homelab's palette, label for label (`model.rs`, `PALETTE`).
pub const PALETTE: [(&str, &str); 19] = [
    ("go: dashboard", "tab.dashboard"),
    ("go: stacks", "tab.stacks"),
    ("go: log stream", "tab.logs"),
    ("go: doctor", "tab.doctor"),
    ("go: settings", "tab.settings"),
    ("go: shell", "tab.shell"),
    ("refresh state", "refresh"),
    ("run doctor", "doctor"),
    ("backup: selected stack", "op.backup"),
    ("update: selected stack", "op.update"),
    ("restore: selected stack (asks first)", "op.restore"),
    ("guards: apply to selected stack", "op.guards"),
    ("adopt: native services of selected stack", "op.adopt"),
    (
        "install-native: binaries of selected stack",
        "op.install-native",
    ),
    ("fleet check: repo against reality", "op.check"),
    ("incidents: list captured bundles", "op.incidents"),
    ("cycle effects (F2)", "fx"),
    ("help", "help"),
    ("quit", "quit"),
];

pub fn palette_labels() -> [&'static str; 19] {
    PALETTE.map(|(label, _)| label)
}

/// The keys every screen shares, as homelab's help lists them.
pub const GLOBAL_KEYS: [(&str, &str); 8] = [
    ("tab / shift+tab", "next / previous screen"),
    ("1–5  & é \" ' (", "jump to a homelab screen"),
    ("ctrl+k / ctrl+p", "command palette"),
    ("F2", "effects"),
    ("t", "theme"),
    ("r", "refresh"),
    ("h", "this help"),
    ("q", "quit"),
];

/// A bar at the foot of the screen for what the last action did, gone
/// after a few seconds, as homelab's status line is replaced by the next.
pub fn draw_status(frame: &mut Frame, th: &Theme, text: &str) {
    let area = frame.area();
    if area.height == 0 {
        return;
    }
    let row = Rect {
        y: area.bottom() - 1,
        height: 1,
        ..area
    };
    Paragraph::new(Line::from(vec![
        Span::styled(" ▸ ", Style::new().fg(th.c.primary)),
        Span::styled(
            text.to_string(),
            Style::new()
                .fg(th.c.popover_foreground)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .style(Style::new().bg(th.c.popover))
    .render(row, frame.buffer_mut());
}

/// The help overlay: this screen's keys, then the ones every screen has,
/// both from the same tables the footers read.
pub fn draw_help(frame: &mut Frame, th: &Theme, screen_keys: &[(&str, &str)]) {
    let area = frame.area();
    let rows = (screen_keys.len() + GLOBAL_KEYS.len() + 4) as u16;
    let popup = Popup::new(th, "Help", (56.min(area.width), rows.min(area.height)));
    let inner = popup.render_over(area, frame.buffer_mut());
    let mut lines = vec![muted(th, "this screen")];
    lines.extend(KeyHints::new(th, screen_keys).overlay());
    lines.push(Line::default());
    lines.push(muted(th, "every screen"));
    lines.extend(KeyHints::new(th, &GLOBAL_KEYS).overlay());
    lines.push(muted(th, "esc, h or enter closes"));
    Paragraph::new(lines)
        .style(Style::new().bg(th.c.popover))
        .render(inner, frame.buffer_mut());
}

fn muted(th: &Theme, text: &str) -> Line<'static> {
    Line::from(Span::styled(
        text.to_string(),
        Style::new().fg(th.c.muted_foreground),
    ))
}

/// Restore is the one operation that can destroy data, so homelab asks
/// for the stack's name to be typed; a keystroke away from a live restore
/// is not a gate.
pub struct Confirm {
    pub stack: &'static str,
    pub typed: String,
}

/// What a key did to an overlay.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Open,
    /// Closed, with what the status line should say.
    Closed(String),
    /// The plan was accepted: run the deploy it previewed.
    Deploy,
}

impl Confirm {
    pub fn new(stack: usize) -> Self {
        Confirm {
            stack: FLEET[stack % FLEET.len()].name,
            typed: String::new(),
        }
    }

    pub fn key(&mut self, code: KeyCode) -> Outcome {
        match code {
            KeyCode::Esc => return Outcome::Closed("cancelled".into()),
            KeyCode::Char(c) => self.typed.push(c),
            KeyCode::Backspace => {
                self.typed.pop();
            }
            KeyCode::Enter => {
                // Deliberately not "try again": the mismatch is the answer.
                return Outcome::Closed(if self.typed.trim() == self.stack {
                    format!(
                        "restore {}: needs a live host — the demo has none",
                        self.stack
                    )
                } else {
                    format!(
                        "typed name does not match '{}' — nothing was done",
                        self.stack
                    )
                });
            }
            _ => {}
        }
        Outcome::Open
    }

    pub fn draw(&self, frame: &mut Frame, th: &Theme, blink: u32) {
        let area = frame.area();
        let title = format!("Restore {}", self.stack);
        let popup = Popup::new(th, &title, (56.min(area.width), 8)).kind(PopupKind::Danger);
        let inner = popup.render_over(area, frame.buffer_mut());
        let [text, field, keys] = Layout::vertical([
            Constraint::Min(3),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(inner);
        Paragraph::new(vec![
            Line::from(Span::styled(
                format!("This replaces {}'s /appdata with the snapshot.", self.stack),
                Style::new().fg(th.c.popover_foreground),
            )),
            Line::default(),
            muted(th, "Type the stack's name to confirm:"),
        ])
        .style(Style::new().bg(th.c.popover))
        .render(text, frame.buffer_mut());
        Field::new(th, "name", &self.typed)
            .focused(true)
            .blink(blink)
            .render(field, frame.buffer_mut());
        Paragraph::new(KeyHints::new(th, &[("enter", "confirm"), ("esc", "cancel")]).footer())
            .style(Style::new().bg(th.c.popover))
            .render(keys, frame.buffer_mut());
    }
}

/// The change plan: what a deploy of the stack in hand would do, shown
/// before anything runs. Enter runs it, Esc drops it.
pub struct Plan {
    pub stack: usize,
}

impl Plan {
    pub fn key(&mut self, code: KeyCode) -> Outcome {
        match code {
            KeyCode::Esc => Outcome::Closed("plan dropped".into()),
            KeyCode::Enter => Outcome::Deploy,
            _ => Outcome::Open,
        }
    }

    /// homelab's `build_plan_lines` for a stack that is already up: an
    /// update, the files that changed, and the safety lines under it.
    pub fn lines(&self) -> Vec<(char, String)> {
        let s = &FLEET[self.stack % FLEET.len()];
        let mut lines = vec![
            (' ', "dry-run — nothing runs until enter".to_string()),
            (' ', String::new()),
            (' ', "plan:".to_string()),
        ];
        if s.online {
            lines.push((
                '~',
                format!("  UPDATE   {} (already provisioned)", s.hostname),
            ));
        } else {
            lines.push(('+', format!("  CREATE   {} (vmid {})", s.hostname, s.vmid)));
        }
        for (i, (app, _, _)) in s.apps.iter().enumerate() {
            if i == 0 {
                lines.push(('~', format!("  UPDATE   {}/{app}", s.name)));
                lines.push(('~', format!("    ~ {app}/compose.yaml")));
                lines.push(('+', "      + image: pinned by digest".to_string()));
                lines.push(('-', "      - image: :latest".to_string()));
            } else {
                lines.push((' ', format!("  SKIP     {}/{app} (no changes)", s.name)));
            }
        }
        lines.push((' ', String::new()));
        lines.push((' ', "safety:".to_string()));
        lines.push((' ', "  ✓ hostname guard verifies before any change".into()));
        lines.push((
            ' ',
            "  ✓ fail-closed: errors abort with an incident bundle".into(),
        ));
        lines.push((' ', "  ✓ no-touch list enforced host-side".into()));
        lines
    }

    pub fn draw(&self, frame: &mut Frame, th: &Theme) {
        let area = frame.area();
        let lines = self.lines();
        let title = format!("Change plan · {}", FLEET[self.stack % FLEET.len()].name);
        let h = (lines.len() as u16 + 4).min(area.height);
        let popup = Popup::new(th, &title, (64.min(area.width), h));
        let inner = popup.render_over(area, frame.buffer_mut());
        let [body, keys] =
            Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(inner);
        let pv = th.id.palette().popover;
        let drawn: Vec<Line<'static>> = lines
            .into_iter()
            .map(|(sign, text)| {
                let style = match sign {
                    '+' => Style::new().fg(th.ink(Tone::Success, pv)),
                    '-' => Style::new().fg(th.ink(Tone::Danger, pv)),
                    '~' => Style::new().fg(th.ink(Tone::Warning, pv)),
                    _ => Style::new().fg(th.c.popover_foreground),
                };
                Line::from(Span::styled(text, style))
            })
            .collect();
        Paragraph::new(drawn)
            .style(Style::new().bg(th.c.popover))
            .render(body, frame.buffer_mut());
        Paragraph::new(KeyHints::new(th, &[("enter", "deploy"), ("esc", "cancel")]).footer())
            .style(Style::new().bg(th.c.popover))
            .render(keys, frame.buffer_mut());
    }
}

/// The presets the wizard starts from: a name, what it brings, its
/// resources, and the `/appdata` paths its templates would create.
pub struct Preset {
    pub name: &'static str,
    pub says: &'static str,
    pub ram: u32,
    pub cores: u16,
    pub disk: u16,
    pub paths: &'static [&'static str],
}

pub const PRESETS: [Preset; 4] = [
    Preset {
        name: "media",
        says: "jellyfin, sonarr, radarr",
        ram: 4096,
        cores: 4,
        disk: 32,
        paths: &["/appdata/jellyfin", "/appdata/sonarr", "/appdata/radarr"],
    },
    Preset {
        name: "web",
        says: "caddy and a static site",
        ram: 1024,
        cores: 2,
        disk: 8,
        paths: &["/appdata/caddy"],
    },
    Preset {
        name: "monitoring",
        says: "prometheus, grafana",
        ram: 2048,
        cores: 2,
        disk: 16,
        paths: &["/appdata/prometheus", "/appdata/grafana"],
    },
    Preset {
        name: "blank",
        says: "no apps yet",
        ram: 512,
        cores: 1,
        disk: 4,
        paths: &[],
    },
];

pub const STEPS: [&str; 5] = ["Preset", "Name", "Resources", "Storage", "Review"];
const FIELDS: [&str; 5] = ["ram", "cores", "disk", "swap", "vmid"];

/// homelab's new-stack wizard (`model.rs`, `Wizard` and `wizard_key`):
/// five steps, each with its own keys, Esc going one step back.
pub struct Wizard {
    pub step: usize,
    pub preset: usize,
    pub name: String,
    pub ram: u32,
    pub cores: u16,
    pub disk: u16,
    pub swap: u32,
    pub swap_touched: bool,
    pub vmid: u16,
    pub field: usize,
    pub disk_typing: bool,
    pub storage_idx: usize,
    pub no_data: Vec<bool>,
}

/// RAM ladder, as homelab's `ram_step`: 256, 512, 1024, 2048, then +1024.
pub fn ram_step(current: u32, up: bool) -> u32 {
    const LOW: [u32; 4] = [256, 512, 1024, 2048];
    if up {
        if current < 2048 {
            *LOW.iter().find(|&&v| v > current).unwrap_or(&2048)
        } else {
            (current + 1024).min(32768)
        }
    } else if current > 2048 {
        current - 1024
    } else {
        *LOW.iter().rev().find(|&&v| v < current).unwrap_or(&256)
    }
}

fn swap_for(ram: u32) -> u32 {
    (ram / 2).min(2048)
}

impl Default for Wizard {
    fn default() -> Self {
        // The lowest vmid in homelab's range that no stack here uses.
        let vmid = (108..=354u16)
            .find(|v| !FLEET.iter().any(|s| s.vmid == u32::from(*v)))
            .unwrap_or(354);
        Wizard {
            step: 0,
            preset: 0,
            name: String::new(),
            ram: 512,
            cores: 1,
            disk: 4,
            swap: 256,
            swap_touched: false,
            vmid,
            field: 0,
            disk_typing: false,
            storage_idx: 0,
            no_data: Vec::new(),
        }
    }
}

impl Wizard {
    fn paths(&self) -> &'static [&'static str] {
        PRESETS[self.preset].paths
    }

    pub fn key(&mut self, code: KeyCode) -> Outcome {
        let n = PRESETS.len();
        match self.step {
            0 => match code {
                KeyCode::Esc => return Outcome::Closed("wizard closed".into()),
                KeyCode::Up => self.preset = (self.preset + n - 1) % n,
                KeyCode::Down => self.preset = (self.preset + 1) % n,
                KeyCode::Enter => {
                    if self.name.is_empty() {
                        self.name = PRESETS[self.preset].name.to_string();
                    }
                    self.step = 1;
                }
                _ => {}
            },
            1 => match code {
                KeyCode::Esc => self.step = 0,
                KeyCode::Char(c) if c.is_ascii_alphanumeric() || c == '-' => {
                    self.name.push(c.to_ascii_lowercase());
                }
                KeyCode::Backspace => {
                    self.name.pop();
                }
                KeyCode::Enter if !self.name.is_empty() => {
                    let p = &PRESETS[self.preset];
                    self.ram = p.ram;
                    self.cores = p.cores;
                    self.disk = p.disk;
                    if !self.swap_touched {
                        self.swap = swap_for(self.ram);
                    }
                    self.step = 2;
                }
                _ => {}
            },
            2 => match code {
                KeyCode::Esc => self.step = 1,
                KeyCode::Up => {
                    self.field = (self.field + FIELDS.len() - 1) % FIELDS.len();
                    self.disk_typing = false;
                }
                KeyCode::Down => {
                    self.field = (self.field + 1) % FIELDS.len();
                    self.disk_typing = false;
                }
                KeyCode::Left | KeyCode::Right => {
                    let up = code == KeyCode::Right;
                    self.disk_typing = false;
                    match self.field {
                        0 => {
                            self.ram = ram_step(self.ram, up);
                            if !self.swap_touched {
                                self.swap = swap_for(self.ram);
                            }
                        }
                        1 if up => self.cores = (self.cores + 1).min(16),
                        1 => self.cores = self.cores.saturating_sub(1).max(1),
                        2 if up => self.disk = (self.disk + 2).min(999),
                        2 => self.disk = self.disk.saturating_sub(2).max(2),
                        3 => {
                            self.swap = if up {
                                (self.swap + 256).min(4096)
                            } else {
                                self.swap.saturating_sub(256)
                            };
                            self.swap_touched = true;
                        }
                        _ if up => self.vmid = (self.vmid + 1).min(354),
                        _ => self.vmid = self.vmid.saturating_sub(1).max(108),
                    }
                }
                // A disk size can be typed as well as stepped.
                KeyCode::Char(c) if self.field == 2 && c.is_ascii_digit() => {
                    if !self.disk_typing {
                        self.disk = 0;
                        self.disk_typing = true;
                    }
                    let d = c.to_digit(10).unwrap_or(0) as u16;
                    self.disk = (self.disk * 10 + d).min(999);
                }
                KeyCode::Backspace if self.field == 2 => {
                    self.disk_typing = true;
                    self.disk /= 10;
                }
                KeyCode::Enter => {
                    self.disk = self.disk.max(2);
                    self.disk_typing = false;
                    self.storage_idx = 0;
                    self.no_data = vec![false; self.paths().len()];
                    self.step = if self.paths().is_empty() { 4 } else { 3 };
                }
                _ => {}
            },
            3 => {
                let len = self.paths().len().max(1);
                match code {
                    KeyCode::Esc => self.step = 2,
                    KeyCode::Up => self.storage_idx = (self.storage_idx + len - 1) % len,
                    KeyCode::Down => self.storage_idx = (self.storage_idx + 1) % len,
                    // Space toggles the row: keeps files, or keeps nothing.
                    KeyCode::Char(' ') => {
                        if let Some(v) = self.no_data.get_mut(self.storage_idx) {
                            *v = !*v;
                        }
                    }
                    KeyCode::Enter => self.step = 4,
                    _ => {}
                }
            }
            _ => match code {
                KeyCode::Esc => self.step = if self.paths().is_empty() { 2 } else { 3 },
                KeyCode::Enter => {
                    return Outcome::Closed(format!(
                        "stack {} scaffolded from {} — p shows its plan",
                        self.name, PRESETS[self.preset].name
                    ));
                }
                _ => {}
            },
        }
        Outcome::Open
    }

    pub fn draw(&self, frame: &mut Frame, th: &Theme, blink: u32) {
        let area = frame.area();
        let popup = Popup::new(th, "New stack", (64.min(area.width), 16.min(area.height)));
        let inner = popup.render_over(area, frame.buffer_mut());
        let [steps, _, body, keys] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .areas(inner);
        Stepper::new(th, &STEPS, self.step).render(steps, frame.buffer_mut());
        let column = kp_tui::label_column(th, &FIELDS) + 2;
        let fg = Style::new().fg(th.c.popover_foreground);
        let mut lines: Vec<Line<'static>> = Vec::new();
        let hints: &[(&str, &str)] = match self.step {
            0 => {
                for (i, p) in PRESETS.iter().enumerate() {
                    let mark = if i == self.preset { "▸ " } else { "  " };
                    let style = if i == self.preset {
                        Style::new().fg(th.c.primary).add_modifier(Modifier::BOLD)
                    } else {
                        fg
                    };
                    lines.push(Line::from(vec![
                        Span::styled(format!("{mark}{:<12}", p.name), style),
                        Span::styled(p.says.to_string(), Style::new().fg(th.c.muted_foreground)),
                    ]));
                }
                &[("↑↓", "preset"), ("enter", "next"), ("esc", "close")]
            }
            1 => {
                lines.push(muted(th, "lowercase letters, digits and -"));
                lines.push(Line::default());
                lines.push(Line::from(
                    Field::new(th, "name", &self.name)
                        .focused(true)
                        .blink(blink)
                        .label_width(column)
                        .spans(),
                ));
                &[("type", "name"), ("enter", "next"), ("esc", "back")]
            }
            2 => {
                let values = [
                    format!("{} MB", self.ram),
                    format!("{}", self.cores),
                    format!("{} GB", self.disk),
                    if self.swap == 0 {
                        "none".into()
                    } else {
                        format!("{} MB", self.swap)
                    },
                    format!("{}", self.vmid),
                ];
                for (i, (label, value)) in FIELDS.iter().zip(values.iter()).enumerate() {
                    let c = Choice::new(th, label, value).focused(i == self.field);
                    lines.push(Line::from(
                        [c.label_spans(column), c.value_spans()].concat(),
                    ));
                }
                &[
                    ("↑↓", "field"),
                    ("←→", "value"),
                    ("digits", "disk size"),
                    ("enter", "next"),
                    ("esc", "back"),
                ]
            }
            3 => {
                lines.push(muted(th, "which paths does the app keep something in?"));
                for (i, path) in self.paths().iter().enumerate() {
                    let nothing = self.no_data.get(i).copied().unwrap_or(false);
                    let mark = if i == self.storage_idx { "▸ " } else { "  " };
                    let (word, tone) = if nothing {
                        ("keeps nothing", Tone::MutedInk)
                    } else {
                        ("keeps files", Tone::Success)
                    };
                    lines.push(Line::from(vec![
                        Span::styled(format!("{mark}{path:<22}"), fg),
                        Span::styled(
                            word.to_string(),
                            Style::new().fg(th.ink(tone, th.id.palette().popover)),
                        ),
                    ]));
                }
                &[
                    ("↑↓", "path"),
                    ("space", "toggle"),
                    ("enter", "next"),
                    ("esc", "back"),
                ]
            }
            _ => {
                let p = &PRESETS[self.preset];
                let kept = self.no_data.iter().filter(|n| !**n).count();
                for (label, value) in [
                    ("name", self.name.clone()),
                    ("preset", p.name.to_string()),
                    (
                        "size",
                        format!("{} MB · {} cores · {} GB", self.ram, self.cores, self.disk),
                    ),
                    ("vmid", self.vmid.to_string()),
                    ("appdata", format!("{kept} of {} keep files", p.paths.len())),
                ] {
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("{label:<column$}"),
                            Style::new().fg(th.c.muted_foreground),
                        ),
                        Span::styled(value, fg),
                    ]));
                }
                &[("enter", "scaffold"), ("esc", "back")]
            }
        };
        Paragraph::new(lines)
            .style(Style::new().bg(th.c.popover))
            .render(body, frame.buffer_mut());
        Paragraph::new(KeyHints::new(th, hints).footer())
            .style(Style::new().bg(th.c.popover))
            .render(keys, frame.buffer_mut());
    }
}
