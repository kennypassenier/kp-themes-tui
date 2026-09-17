//! Demo state and the one draw function. The terminal loop lives in
//! `main.rs`; tests drive this with `TestBackend`.

use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
};

use crate::color::ColorDepth;
use crate::config::Config;
use crate::fx::{self, Motion};
use crate::generated::PACKAGE_VERSION;
use crate::theme::Theme;
use crate::widgets::{Button, ButtonKind, ButtonState, Panel, RevealText, ThemedTabs};

pub const TABS: [&str; 3] = ["Overview", "Deployments", "Settings"];

const CONTENT: [(&str, &str, &str); 3] = [
    (
        "Status",
        "All services are running",
        "Twelve containers on two hosts. The last backup finished at 03:10 and was verified at 03:24. \
         No certificate expires in the next thirty days.",
    ),
    (
        "Release",
        "Version 2.4 is ready to deploy",
        "The build passed on both hosts. Deploying restarts the web and worker containers one at a time; \
         the database is not touched. Rolling back restores 2.3 from the image cache.",
    ),
    (
        "Preferences",
        "Theme and motion are saved",
        "Press t to change the theme and m to turn motion off. Both choices are written to the \
         config file and read on the next start.",
    ),
];

pub const BUTTONS: [(&str, ButtonKind); 2] = [("Deploy", ButtonKind::Primary), ("Roll back", ButtonKind::Destructive)];

pub struct App {
    pub config: Config,
    pub depth: ColorDepth,
    pub theme: Theme,
    pub tab: usize,
    pub focus: usize,
    /// Which button is pressed, and for how many more milliseconds. A
    /// terminal reports key presses, not releases (unless the kitty keyboard
    /// protocol is on), so "pressed" is shown for a fixed time.
    pub pressed: Option<(usize, u32)>,
    pub reveal_ms: u32,
    pub config_path: Option<PathBuf>,
    pub message: String,
    pub quit: bool,
}

pub struct Areas {
    pub header: Rect,
    pub tabs: Rect,
    pub story: Rect,
    pub actions: Rect,
    pub states: Rect,
    pub footer: Rect,
}

pub fn areas(area: Rect) -> Areas {
    let [header, tabs, body, states, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(9),
        Constraint::Length(5),
        Constraint::Length(1),
    ])
    .areas(area);
    let [story, actions] = Layout::horizontal([Constraint::Percentage(62), Constraint::Percentage(38)]).areas(body);
    Areas { header, tabs, story, actions, states, footer }
}

impl App {
    /// The headline's row and its text, for an effect that runs over the
    /// drawn buffer.
    pub fn headline(&self, area: Rect) -> (Rect, &'static str) {
        let a = areas(area);
        let inner = Panel::new(&self.theme, "").block().inner(a.story).inner(ratatui::layout::Margin::new(1, 0));
        let head = Rect { height: 1, width: inner.width.min(CONTENT[self.tab].1.chars().count() as u16), ..inner };
        (head, CONTENT[self.tab].1)
    }

    pub fn new(config: Config, depth: ColorDepth, config_path: Option<PathBuf>) -> Self {
        App {
            config,
            depth,
            theme: Theme::new(config.theme, depth),
            tab: 1,
            focus: 0,
            pressed: None,
            reveal_ms: 0,
            config_path,
            message: String::new(),
            quit: false,
        }
    }

    fn persist(&mut self) {
        if let Some(path) = &self.config_path {
            self.message = match self.config.save(path) {
                Ok(()) => format!("saved to {}", path.display()),
                Err(e) => format!("not saved: {e}"),
            };
        }
    }

    /// The next frame renders in the next theme; nothing else is rebuilt.
    pub fn cycle_theme(&mut self) {
        self.config.theme = self.config.theme.next();
        self.theme = Theme::new(self.config.theme, self.depth);
        self.reveal_ms = 0;
        self.persist();
    }

    pub fn toggle_motion(&mut self) {
        self.config.motion = match self.config.motion {
            Motion::Full => Motion::Reduced,
            Motion::Reduced => Motion::Full,
        };
        self.persist();
    }

    pub fn tick(&mut self, ms: u32) {
        self.reveal_ms = self.reveal_ms.saturating_add(ms);
        if let Some((i, left)) = self.pressed {
            self.pressed = left.checked_sub(ms).filter(|l| *l > 0).map(|l| (i, l));
        }
    }

    pub fn key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.quit = true,
            KeyCode::Char('t') => self.cycle_theme(),
            KeyCode::Char('m') => self.toggle_motion(),
            KeyCode::Char('r') => self.reveal_ms = 0,
            KeyCode::Left => {
                self.tab = (self.tab + TABS.len() - 1) % TABS.len();
                self.reveal_ms = 0;
            }
            KeyCode::Right => {
                self.tab = (self.tab + 1) % TABS.len();
                self.reveal_ms = 0;
            }
            KeyCode::Tab | KeyCode::BackTab => self.focus = (self.focus + 1) % BUTTONS.len(),
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.pressed = Some((self.focus, 160));
                self.message = format!("{} pressed", BUTTONS[self.focus].0);
            }
            _ => {}
        }
    }

    pub fn draw(&self, frame: &mut Frame) {
        let th = &self.theme;
        let buf_area = frame.area();
        frame.render_widget(Block::new().style(th.base()), buf_area);
        let a = areas(buf_area);

        let header = Line::from(vec![
            Span::styled(th.label("kp-themes"), Style::new().fg(th.c.primary).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!(
                    "  {PACKAGE_VERSION} · theme {} · colours {} · motion {}",
                    th.id.name(),
                    self.depth.label(),
                    if self.config.motion == Motion::Reduced { "reduced" } else { "full" }
                ),
                Style::new().fg(th.c.muted_foreground),
            ),
        ]);
        frame.render_widget(Paragraph::new(header), a.header);
        frame.render_widget(ThemedTabs::new(th, &TABS, self.tab), a.tabs);

        // Story panel: the reveal, the body, and a filter field for the cursor.
        let (title, headline, body) = CONTENT[self.tab];
        let panel = Panel::new(th, title).focused(false);
        let inner = panel.block().inner(a.story);
        frame.render_widget(panel, a.story);
        let [head, _, text, field] =
            Layout::vertical([Constraint::Length(1), Constraint::Length(1), Constraint::Min(2), Constraint::Length(1)])
                .areas(inner.inner(ratatui::layout::Margin::new(1, 0)));
        // With `--features tachyonfx` the headline is drawn whole here and
        // main.rs runs the tachyonfx effect over `App::headline`.
        let motion = if cfg!(feature = "tachyonfx") { Motion::Reduced } else { self.config.motion };
        frame.render_widget(RevealText::new(th, headline, self.reveal_ms, motion), head);
        frame.render_widget(
            Paragraph::new(body).wrap(Wrap { trim: true }).style(Style::new().fg(th.c.card_foreground)),
            text,
        );
        let prompt = th.label("filter");
        let value = "web";
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(format!("{prompt}: "), Style::new().fg(th.c.muted_foreground)),
                Span::styled(value, Style::new().fg(th.c.foreground)),
            ])),
            field,
        );
        let col = field.x + (prompt.chars().count() + 2 + value.len()) as u16;
        frame.set_cursor_position((col.min(field.right().saturating_sub(1)), field.y));

        // Actions panel: focus follows Tab, Enter presses.
        let panel = Panel::new(th, "Actions").focused(true);
        let inner = panel.block().inner(a.actions);
        frame.render_widget(panel, a.actions);
        let rows =
            Layout::vertical([Constraint::Length(3), Constraint::Length(1), Constraint::Length(3), Constraint::Min(0)])
                .areas::<4>(inner.inner(ratatui::layout::Margin::new(2, 1)));
        for (i, (label, kind)) in BUTTONS.iter().enumerate() {
            let state = match self.pressed {
                Some((p, _)) if p == i => ButtonState::Pressed,
                _ if self.focus == i => ButtonState::Focus,
                _ => ButtonState::Rest,
            };
            frame.render_widget(Button::new(th, label).kind(*kind).state(state), rows[i * 2]);
        }

        // Every button state at once, so no key has to be pressed to judge them.
        let panel = Panel::new(th, "Button states");
        let inner = panel.block().inner(a.states);
        frame.render_widget(panel, a.states);
        let cells = Layout::horizontal([Constraint::Ratio(1, 4); 4])
            .spacing(2)
            .areas::<4>(inner.inner(ratatui::layout::Margin::new(1, 0)));
        for (cell, (label, state)) in cells.iter().zip([
            ("Rest", ButtonState::Rest),
            ("Focus", ButtonState::Focus),
            ("Pressed", ButtonState::Pressed),
            ("Disabled", ButtonState::Disabled),
        ]) {
            frame.render_widget(Button::new(th, label).state(state), *cell);
        }

        let keys =
            format!(" t theme · ←/→ tab · Tab focus · Enter press · r replay · m motion · q quit   {}", self.message);
        frame.render_widget(
            Paragraph::new(keys).style(Style::new().bg(th.c.secondary).fg(th.c.secondary_foreground)),
            a.footer,
        );
        let _ = fx::GLYPHS; // the effect module is part of the public surface
    }
}
