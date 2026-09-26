//! homelab's deploy focus window, rebuilt on this crate — the fifth proof.
//!
//! The original is `client/src/tui/view/focus.rs`, 242 lines: a near
//! fullscreen takeover over whatever tab you were on, with this deploy's
//! own transcript, a question drawn over that transcript when a step needs
//! a decision, the transfer visuals, a gauge and a footer.
//!
//! It is the only homelab screen that is an overlay, which is why it was
//! picked: the four before it all owned their whole area.
//!
//! It found one gap on the way: a bar with a sentence written across it
//! rather than a percentage beside it. That is `Meter::across` now, so
//! nothing here is marked `GAP` and no colour on it is chosen by hand.
//!
//! Kenny picked **Two panes** out of five directions: the steps on the
//! left, the transcript on the right, both moving at once. The transcript
//! keeps every line it had — the direction adds the answer to "where is
//! it now", it does not take the output away [fix-65]. The steps' own
//! columns are fixed, so a longer name cannot push the timing sideways
//! [fix-64].

use crossterm::event::KeyCode;
use kp_tui::{
    Badge, Glitch, KeyHints, Meter, Popup, PopupKind, Stream, Theme, Tone, fx::Motion, spinner,
};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

pub struct Step {
    pub text: &'static str,
    pub tone: Tone,
}

pub const TRANSCRIPT: [Step; 9] = [
    Step {
        text: "[sync][run ] rsync -a --delete ./stacks/media/ pve-01:/srv/media/",
        tone: Tone::Info,
    },
    Step {
        text: "[sync][exit] 0 in 4.2 s, 118 files, 41.9 MB",
        tone: Tone::Success,
    },
    Step {
        text: "[gate] compose config — valid",
        tone: Tone::Success,
    },
    Step {
        text: "[gate] image digests pinned — 6 of 6",
        tone: Tone::Success,
    },
    Step {
        text: "[run ] docker compose up -d",
        tone: Tone::Info,
    },
    Step {
        text: "jellyfin recreated",
        tone: Tone::MutedInk,
    },
    Step {
        text: "jellyfin-exporter recreated",
        tone: Tone::MutedInk,
    },
    Step {
        text: "[warn] jellyfin: health check not ready after 30 s, retrying",
        tone: Tone::Warning,
    },
    Step {
        text: "[run ] waiting for health…",
        tone: Tone::Info,
    },
];

/// The question a step raises, drawn over the transcript rather than beside
/// it: the transcript is exactly what the operator is reading, and a
/// question elsewhere is a question missed.
pub struct Ask {
    pub op: &'static str,
    pub step: &'static str,
    pub what: &'static str,
    pub if_allowed: &'static str,
    pub if_stopped: &'static str,
}

pub const ASK: Ask = Ask {
    op: "deploy media",
    step: "prune",
    what: "Remove 3 images no stack refers to any more?",
    if_allowed: "frees 2.1 GB; a rollback re-pulls them",
    if_stopped: "the deploy finishes, the images stay",
};

/// The steps this deploy walks, and where it stands in them. The
/// transcript is what each step said; this is what they are.
pub struct Phase {
    pub name: &'static str,
    pub state: Tone,
    pub says: &'static str,
}

pub const PHASES: [Phase; 6] = [
    Phase {
        name: "sync",
        state: Tone::Success,
        says: "4.2 s · 118 files",
    },
    Phase {
        name: "gates",
        state: Tone::Success,
        says: "2 of 2 passed",
    },
    Phase {
        name: "pull",
        state: Tone::Success,
        says: "6 digests pinned",
    },
    Phase {
        name: "up",
        state: Tone::Info,
        says: "recreating 2 of 6",
    },
    Phase {
        name: "health",
        state: Tone::Warning,
        says: "retrying, 30 s",
    },
    Phase {
        name: "prune",
        state: Tone::MutedInk,
        says: "waiting on an answer",
    },
];

/// The footer, by where the deploy stands; the help overlay reads it too.
pub const KEYS_ASKING: [(&str, &str); 2] = [("a", "allow"), ("s", "stop")];
pub const KEYS_RUNNING: [(&str, &str); 3] = [
    ("↑↓", "scroll"),
    ("esc", "background — the deploy keeps running"),
    ("q", "quit"),
];
pub const KEYS_DONE: [(&str, &str); 3] = [("↑↓", "scroll"), ("enter", "close"), ("q", "quit")];

/// This deploy's own state, the part homelab keeps in its `Focus` and its
/// `PendingAsk`: whether a step is waiting on an answer, what it was
/// given, how far the feed is scrolled back, and whether it is done.
pub struct Deploy {
    pub asking: bool,
    pub allowed: Option<bool>,
    /// Lines scrolled back from the foot of the feed.
    pub scroll: usize,
    /// Milliseconds since the answer; the rest of the deploy runs on it.
    pub since_answer: u32,
    pub done: bool,
}

/// How long the demo's deploy runs on after its question is answered.
pub const FINISH_MS: u32 = 1500;

impl Default for Deploy {
    fn default() -> Self {
        Deploy {
            asking: true,
            allowed: None,
            scroll: 0,
            since_answer: 0,
            done: false,
        }
    }
}

/// What a key did to the window, for the screen around it.
#[derive(Debug, PartialEq, Eq)]
pub enum After {
    Stay,
    /// The window closes; the deploy is over.
    Close,
    /// The window goes; the deploy keeps running behind it.
    Background,
}

impl Deploy {
    /// One key, the way homelab's `on_key` reads it while its focus window
    /// is up: a waiting question outranks everything, and every key but
    /// its two answers is swallowed, because scrolling past a question is
    /// how it gets missed.
    pub fn key(&mut self, code: KeyCode) -> After {
        if self.asking {
            match code {
                KeyCode::Char('a') => self.answer(true),
                KeyCode::Char('s') => self.answer(false),
                _ => {}
            }
            return After::Stay;
        }
        match code {
            KeyCode::Up => {
                self.scroll = (self.scroll + 1).min(self.feed().len().saturating_sub(1));
            }
            KeyCode::Down => self.scroll = self.scroll.saturating_sub(1),
            KeyCode::Esc if self.done => return After::Close,
            KeyCode::Esc => return After::Background,
            KeyCode::Enter if self.done => return After::Close,
            _ => {}
        }
        After::Stay
    }

    /// The step takes its answer and the deploy runs on; the demo has no
    /// host to wait on, so the end follows the answer by a moment.
    fn answer(&mut self, allow: bool) {
        self.asking = false;
        self.allowed = Some(allow);
    }

    pub fn tick(&mut self, ms: u32) {
        if self.allowed.is_some() && !self.done {
            self.since_answer = self.since_answer.saturating_add(ms);
            self.done = self.since_answer >= FINISH_MS;
        }
    }

    pub fn keys(&self) -> &'static [(&'static str, &'static str)] {
        if self.asking {
            &KEYS_ASKING
        } else if self.done {
            &KEYS_DONE
        } else {
            &KEYS_RUNNING
        }
    }

    /// The transcript as it stands, with what the answer set in motion.
    pub fn feed(&self) -> Vec<(&'static str, Tone)> {
        let mut lines: Vec<(&'static str, Tone)> =
            TRANSCRIPT.iter().map(|s| (s.text, s.tone)).collect();
        match self.allowed {
            Some(true) => lines.extend([
                ("[ask ] prune — allowed", Tone::Success),
                ("[run ] docker image prune: 3 images, 2.1 GB", Tone::Info),
            ]),
            Some(false) => lines.push(("[ask ] prune — stopped by the operator", Tone::Warning)),
            None => {}
        }
        if self.done {
            lines.push(("[ok  ] jellyfin healthy after 41 s", Tone::Success));
            lines.push(match self.allowed {
                Some(true) => ("[done] deploy media — ok", Tone::Success),
                _ => ("[done] deploy media — ok, 3 images kept", Tone::Success),
            });
        }
        lines
    }

    fn phases(&self) -> Vec<(&'static str, Tone, &'static str)> {
        let mut out: Vec<_> = PHASES.iter().map(|p| (p.name, p.state, p.says)).collect();
        if !self.asking && !self.done {
            out[5] = match self.allowed {
                Some(true) => ("prune", Tone::Info, "pruning 3 images"),
                _ => ("prune", Tone::MutedInk, "stopped by the operator"),
            };
        }
        if self.done {
            out[3] = ("up", Tone::Success, "6 of 6 recreated");
            out[4] = ("health", Tone::Success, "healthy after 41 s");
            out[5] = match self.allowed {
                Some(true) => ("prune", Tone::Success, "2.1 GB freed"),
                _ => ("prune", Tone::MutedInk, "stopped by the operator"),
            };
        }
        out
    }
}

pub fn draw(frame: &mut Frame, th: &Theme, d: &Deploy, reveal_ms: u32, motion: Motion) {
    let screen = frame.area();

    // The title glitches where a register declares it, which is the same
    // routine homelab runs over this title by hand.
    let title = if d.done {
        "Deploy media — done"
    } else {
        "Deploy media — live"
    };
    let glitched =
        th.a.fx
            .alarm
            .glitch
            .map(|g: Glitch| g.text(title, reveal_ms, 0x30DA1, motion))
            .unwrap_or_else(|| title.to_string());

    let popup = Popup::new(
        th,
        &glitched,
        (
            screen.width.saturating_sub(8),
            screen.height.saturating_sub(4),
        ),
    );
    let inner = popup.render_over(screen, frame.buffer_mut());
    let [panes, transfer, gauge, footer] = Layout::vertical([
        Constraint::Min(4),
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(inner);

    // Two panes: where the deploy stands, and what it has been saying.
    let [steps, feed] =
        Layout::horizontal([Constraint::Length(34), Constraint::Min(30)]).areas(panes);
    draw_steps(frame, th, steps, &d.phases());
    draw_feed(frame, th, feed, &d.feed(), d.scroll);
    if d.asking {
        draw_ask(frame, th, feed);
    }

    // The transfer this deploy is pushing: its name and what has gone over,
    // then the stream itself, which nobody can put a fraction on.
    let [label, flow] = Layout::vertical([Constraint::Length(1); 2]).areas(transfer);
    Paragraph::new(Line::from(vec![
        Span::styled("⇅ ", Style::new().fg(th.c.primary)),
        Span::styled(
            "media/jellyfin.tar".to_string(),
            Style::new().fg(th.c.popover_foreground),
        ),
        Span::styled(
            "  412 MB".to_string(),
            Style::new().fg(th.c.muted_foreground),
        ),
    ]))
    .style(Style::new().bg(th.c.popover))
    .render(label, frame.buffer_mut());
    Stream::new(th, reveal_ms, motion).render(flow, frame.buffer_mut());

    // The gauge, with the sentence written across it rather than a
    // percentage beside it: on one row there is no space for both, and
    // what an operator wants here is what is happening.
    let (share, says) = if d.done {
        (1.0, "done — enter closes")
    } else {
        (0.62, "streaming over TLS…")
    };
    Meter::new(th, "deploy", share)
        .thresholds(2.0, 2.0)
        .across(says)
        .render(gauge, frame.buffer_mut());

    Paragraph::new(Line::from(
        [
            vec![Span::styled(
                format!("{} ", spinner(th, reveal_ms, motion)),
                Style::new().fg(th.c.primary),
            )],
            KeyHints::new(th, d.keys()).footer().spans,
        ]
        .concat(),
    ))
    .style(Style::new().bg(th.c.popover))
    .render(footer, frame.buffer_mut());
}

/// The left pane: the steps, their state and what each one has to say
/// for itself — all three in columns that do not move.
fn draw_steps(frame: &mut Frame, th: &Theme, area: Rect, phases: &[(&str, Tone, &str)]) {
    let names: Vec<&str> = phases.iter().map(|p| p.0).collect();
    let column = kp_tui::label_column(th, &names) + 2;
    let mut lines = vec![Line::from(Span::styled(
        "  where it stands".to_string(),
        Style::new().fg(th.c.muted_foreground),
    ))];
    for &(name, state, says) in phases {
        let ink = th.ink(state, th.id.palette().popover);
        lines.push(Line::from(vec![
            Span::raw("  "),
            Badge::state(th, state),
            Span::raw(" "),
            Span::styled(
                format!("{:<column$}", name),
                Style::new().fg(ink).add_modifier(Modifier::BOLD),
            ),
            Span::styled(says.to_string(), Style::new().fg(th.c.muted_foreground)),
        ]));
    }
    Paragraph::new(lines)
        .style(Style::new().bg(th.c.popover))
        .render(area, frame.buffer_mut());
}

/// The right pane: the transcript, one line per step, each in the ink its
/// tone deserves on the surface it lands on.
fn draw_feed(frame: &mut Frame, th: &Theme, area: Rect, feed: &[(&str, Tone)], scroll: usize) {
    // Anchored at the foot, as homelab's is: scrolling back moves the
    // window up from the newest line.
    let h = area.height as usize;
    let end = feed.len().saturating_sub(scroll);
    let start = end.saturating_sub(h);
    let lines: Vec<Line<'static>> = feed[start..end]
        .iter()
        .map(|(text, tone)| {
            Line::from(Span::styled(
                format!("  {text}"),
                Style::new().fg(th.ink(*tone, th.id.palette().popover)),
            ))
        })
        .collect();
    Paragraph::new(lines)
        .style(Style::new().bg(th.c.popover))
        .render(area, frame.buffer_mut());
}

/// The question, over the foot of the transcript: what it asks, and what
/// each answer does — not only the two words.
fn draw_ask(frame: &mut Frame, th: &Theme, over: Rect) {
    let h = 9.min(over.height);
    let box_rect = Rect {
        y: over.y + over.height.saturating_sub(h),
        height: h,
        ..over
    };
    let heading = format!("{} · {}", ASK.op, ASK.step);
    let popup = Popup::new(th, &heading, (box_rect.width, h)).kind(PopupKind::Danger);
    let inner = popup.render_over(box_rect, frame.buffer_mut());
    // Both answers start their explanation in the same column, so the
    // two consequences can be read against each other [fix-64].
    let column = kp_tui::label_column(th, &["toelaten", "stoppen"]) + 2;
    let answer = |key: &'static str, word: &'static str, tone: Tone, what: &'static str| {
        Line::from(vec![
            Span::styled(
                format!("  {key} {:<column$}", word),
                Style::new()
                    .fg(th.ink(tone, th.id.palette().popover))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(what.to_string(), Style::new().fg(th.c.muted_foreground)),
        ])
    };
    Paragraph::new(vec![
        Line::from(Span::styled(
            format!("  {}", ASK.what),
            Style::new()
                .fg(th.c.popover_foreground)
                .add_modifier(Modifier::BOLD),
        )),
        Line::default(),
        answer("a", "toelaten", Tone::Success, ASK.if_allowed),
        answer("s", "stoppen", Tone::Danger, ASK.if_stopped),
        Line::default(),
        Line::from(Span::styled(
            "  no answer means unattended; the step does not go ahead".to_string(),
            Style::new().fg(th.c.muted_foreground),
        )),
    ])
    .style(Style::new().bg(th.c.popover))
    .render(inner, frame.buffer_mut());
}
