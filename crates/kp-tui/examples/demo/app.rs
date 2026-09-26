//! Demo state and the draw functions of both screens. The terminal loop
//! lives in `main.rs`; tests drive this with `TestBackend`.

use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
};

use crate::config::Config;
use kp_tui::KP_THEMES_VERSION as PACKAGE_VERSION;
use kp_tui::color::ColorDepth;
use kp_tui::dashboard::{self, Dashboard};
use kp_tui::fx::{self, Motion};
use kp_tui::widgets::{Button, ButtonKind, ButtonState, Panel, RevealText, ThemedTabs};
use kp_tui::{
    AlarmPanel, CommandPalette, Field, KeyHints, Meter, Popup, PopupKind, Rail, SelectList, Stage,
    Stepper, Surface, Texture, Theme, Ticker, roll, source_colour, spinner,
};

pub const TABS: [&str; 3] = ["Overview", "Deployments", "Settings"];

/// Tab and shift+tab walk the screens in this order: homelab's five
/// rebuilt tabs in homelab's own order, its deploy window and splash,
/// then the demo's own four.
pub const ORDER: [Screen; 11] = [
    Screen::Ops,
    Screen::Fleet,
    Screen::LogStream,
    Screen::Doctor,
    Screen::Settings,
    Screen::Deploy,
    Screen::Splash,
    Screen::Dashboard,
    Screen::Components,
    Screen::Console,
    Screen::Effects,
];

/// homelab's tabs by number (`model.rs`, `Tab::ALL`): dashboard, stacks,
/// logs, doctor, settings. The sixth, the shell, is not rebuilt.
pub const HOMELAB: [Screen; 5] = [
    Screen::Ops,
    Screen::Fleet,
    Screen::LogStream,
    Screen::Doctor,
    Screen::Settings,
];

/// homelab's `azerty_tab_index`: the digit, or the symbol an azerty
/// keyboard types on that key without shift.
pub fn tab_index(c: char) -> Option<usize> {
    match c {
        '1' | '&' => Some(0),
        '2' | 'é' => Some(1),
        '3' | '"' => Some(2),
        '4' | '\'' => Some(3),
        '5' | '(' => Some(4),
        '6' | '§' => Some(5),
        _ => None,
    }
}

/// The stack operations only a live host can carry out, by homelab's key.
const HOST_OPS: [(char, &str); 10] = [
    ('u', "update the host binary"),
    ('D', "deploy"),
    ('B', "backup"),
    ('U', "update"),
    ('g', "guards"),
    ('A', "adopt"),
    ('I', "install-native"),
    ('c', "fleet check"),
    ('i', "incidents"),
    ('e', "park"),
];

/// The keys of the two stack screens, for the help overlay.
pub const STACK_KEYS: [(&str, &str); 6] = [
    ("↑↓ j k", "stack"),
    ("p", "change plan, enter deploys"),
    ("R", "restore, asks for the name"),
    ("n", "new stack"),
    ("r", "refresh"),
    ("u D B U g A I c i e", "host operations"),
];

/// How long the status line stays up.
const STATUS_MS: u32 = 4000;

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

/// cyberpunk-register.css `animation: kp-charge 520ms`, the sweep across a
/// button's face.
pub const CHARGE_MS: u32 = 520;

/// A button takes the width it asks for, at the start of the box it was
/// given, and never more than the box holds.
fn fit(slot: Rect, want: u16) -> Rect {
    Rect {
        width: want.min(slot.width),
        ..slot
    }
}

pub const BUTTONS: [(&str, ButtonKind); 2] = [
    ("Deploy", ButtonKind::Primary),
    ("Roll back", ButtonKind::Destructive),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    /// Live machine stats, charts and the journal. The binary opens here.
    Dashboard,
    /// The first round's panels, tabs and buttons. `App::new` starts here,
    /// so the first round's tests read the same frame they always did.
    Components,
    /// What homelab hand-rolls today: a popup over a screen, fields with
    /// the theme's caret, meters with their thresholds, and one keymap
    /// drawn as a footer and as an overlay [docs/HOMELAB_INVENTORY.md].
    Console,
    /// The theme's own motion: the texture on the ground, the sweep where
    /// a register declares one, the alarm, the spinner and the ticker —
    /// homelab's six hand-rolled effects, answered by the theme.
    Effects,
    /// homelab's own stacks screen, rebuilt on this crate [docs/HOMELAB_PROOF.md].
    Fleet,
    /// homelab's own dashboard, rebuilt on this crate — the second proof
    /// [docs/HOMELAB_PROOF.md].
    Ops,
    /// homelab's own settings tab, rebuilt on this crate — the third proof
    /// [docs/HOMELAB_PROOF.md].
    Settings,
    /// homelab's own log stream, rebuilt on this crate — the fourth proof
    /// [docs/HOMELAB_PROOF.md].
    LogStream,
    /// homelab's own deploy window, rebuilt on this crate — the fifth
    /// proof, and the only one that is an overlay [docs/HOMELAB_PROOF.md].
    Deploy,
    /// homelab's own doctor tab, rebuilt on this crate — the sixth proof
    /// [docs/HOMELAB_PROOF.md].
    Doctor,
    /// homelab's own boot splash, rebuilt on this crate — the seventh,
    /// and the last of its nine screens [docs/HOMELAB_PROOF.md].
    Splash,
}

pub struct App {
    pub screen: Screen,
    pub dash: Dashboard,
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
    /// Milliseconds since the focused button took focus, while its charge
    /// sweep runs. cyberpunk sweeps on hover; a keyboard TUI has no
    /// pointer, so focus is where it lands (GUESS, as with the focus
    /// modifier).
    pub charge_ms: Option<u32>,
    /// Milliseconds since the alarm was raised: the strike and the glitch
    /// read it, and `a` sets it back to zero.
    pub alarm_ms: u32,
    /// The ticker's segments, rebuilt from the live sample each tick, so
    /// the line that slides past is this machine and not a fixture.
    pub ticker: Vec<String>,
    /// The command palette: open, what is typed in it, and which row is in
    /// hand. `p` opens it, Esc closes it.
    pub palette_open: bool,
    pub palette_query: String,
    pub palette_sel: usize,
    /// Which stack is in hand in the console's list.
    pub stack_sel: usize,
    /// Which source the log screen's selector points at; 0 is all of them.
    pub source_sel: usize,
    /// The settings screen's own state: what the host runs, what has been
    /// edited, the row in hand, and the webhook while it is typed.
    pub settings: crate::settings::Settings,
    /// The deploy window's own state: the question, the scroll, the end.
    pub deploy: crate::deploy::Deploy,
    /// Where Esc or enter takes the deploy window back to.
    pub back: Screen,
    /// homelab's overlays, each of which holds the keyboard while it is up.
    pub help_open: bool,
    pub confirm: Option<crate::overlays::Confirm>,
    pub plan: Option<crate::overlays::Plan>,
    pub wizard: Option<crate::overlays::Wizard>,
    /// What the last action did, and how long ago it said so.
    pub status: Option<(String, u32)>,
    pub stream: kp_tui::logs::LogBuffer,
    /// Which step of the wizard the breadcrumb shows.
    pub step: usize,
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
    let [story, actions] =
        Layout::horizontal([Constraint::Percentage(62), Constraint::Percentage(38)]).areas(body);
    Areas {
        header,
        tabs,
        story,
        actions,
        states,
        footer,
    }
}

impl App {
    pub fn new(config: Config, depth: ColorDepth, config_path: Option<PathBuf>) -> Self {
        App {
            screen: Screen::Components,
            dash: Dashboard::default(),
            config,
            depth,
            theme: Theme::new(config.theme, depth),
            tab: 1,
            focus: 0,
            pressed: None,
            reveal_ms: 0,
            charge_ms: None,
            alarm_ms: 0,
            ticker: Vec::new(),
            palette_open: false,
            palette_query: String::new(),
            palette_sel: 0,
            stack_sel: 1,
            source_sel: 0,
            settings: crate::settings::Settings::default(),
            deploy: crate::deploy::Deploy::default(),
            back: Screen::Ops,
            help_open: false,
            confirm: None,
            plan: None,
            wizard: None,
            status: None,
            stream: crate::logstream::feed(),
            step: 2,
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
    /// Point the buffer at the source the selector shows, so the bar
    /// really filters rather than only painting itself [fix-65].
    pub fn apply_source(&mut self) {
        let names = crate::logstream::sources();
        self.stream.select_source(match self.source_sel {
            0 => None,
            i => names.get(i).copied(),
        });
    }

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
        self.dash.tick(ms);
        self.charge_ms = self.charge_ms.map(|c| c + ms).filter(|c| *c < CHARGE_MS);
        self.alarm_ms = self.alarm_ms.saturating_add(ms);
        self.ticker = match self.dash.history.back() {
            Some((_, s)) => vec![
                format!("cpu {:.0}%", s.cpu_total),
                format!("mem {:.0}%", s.mem_used_pct),
                format!("load {:.2}", s.load[0]),
                format!("net {:.0} kB/s in", s.rx_bps / 1000.0),
                format!("{} log lines", self.dash.logs.len()),
                format!("draw {:.2} ms", self.dash.draw_ms),
            ],
            None => vec![
                "no sample yet".into(),
                format!("theme {}", self.theme.id.name()),
                format!("{} log lines", self.dash.logs.len()),
            ],
        };
        self.deploy.tick(ms);
        self.status = self
            .status
            .take()
            .map(|(text, age)| (text, age.saturating_add(ms)))
            .filter(|(_, age)| *age < STATUS_MS);
        if let Some((i, left)) = self.pressed {
            self.pressed = left.checked_sub(ms).filter(|l| *l > 0).map(|l| (i, l));
        }
    }

    fn say(&mut self, text: impl Into<String>) {
        self.status = Some((text.into(), 0));
    }

    fn go(&mut self, screen: Screen) {
        self.screen = screen;
        self.reveal_ms = 0;
    }

    /// The keys of the screen on show, for the help overlay.
    pub fn screen_keys(&self) -> &'static [(&'static str, &'static str)] {
        match self.screen {
            Screen::Fleet | Screen::Ops => &STACK_KEYS,
            Screen::Settings => &crate::settings::KEYS,
            Screen::LogStream => &crate::logstream::KEYS,
            Screen::Deploy => self.deploy.keys(),
            Screen::Doctor => &crate::doctor::KEYS,
            Screen::Console => CONSOLE_KEYS,
            _ => &[],
        }
    }

    /// One key, in homelab's order (`client/src/tui/model.rs`, `on_key`):
    /// the splash, then whatever overlay holds the keyboard, then the keys
    /// every screen shares, then the screen's own. On a rebuilt screen
    /// every key does what it does in homelab; the demo's own keys sit
    /// where homelab leaves room [fix-68].
    pub fn key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        let code = key.code;
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        // homelab's splash gives way to any key at all.
        if self.screen == Screen::Splash {
            self.go(Screen::Ops);
            return;
        }
        if self.palette_open {
            self.palette_key(code);
            return;
        }
        // The deploy window has the keyboard while it is up: a waiting
        // question first, then the feed, Esc and enter.
        if self.screen == Screen::Deploy {
            match self.deploy.key(code) {
                crate::deploy::After::Stay => {}
                crate::deploy::After::Close => {
                    self.deploy = crate::deploy::Deploy::default();
                    self.go(self.back);
                }
                crate::deploy::After::Background => {
                    self.go(self.back);
                    self.say("deploy keeps running — feed in the log stream");
                }
            }
            return;
        }
        if self.help_open {
            if matches!(code, KeyCode::Esc | KeyCode::Char('h') | KeyCode::Enter) {
                self.help_open = false;
            }
            return;
        }
        if let Some(c) = self.confirm.as_mut() {
            if let crate::overlays::Outcome::Closed(says) = c.key(code) {
                self.confirm = None;
                self.say(says);
            }
            return;
        }
        if let Some(w) = self.wizard.as_mut() {
            if let crate::overlays::Outcome::Closed(says) = w.key(code) {
                self.wizard = None;
                self.say(says);
            }
            return;
        }
        if let Some(p) = self.plan.as_mut() {
            match p.key(code) {
                crate::overlays::Outcome::Open => {}
                crate::overlays::Outcome::Closed(says) => {
                    self.plan = None;
                    self.say(says);
                }
                crate::overlays::Outcome::Deploy => {
                    self.plan = None;
                    self.deploy = crate::deploy::Deploy::default();
                    self.back = self.screen;
                    self.go(Screen::Deploy);
                }
            }
            return;
        }
        // The webhook being typed swallows every key; digits would jump.
        if self.screen == Screen::Settings && self.settings.typing() {
            self.settings.key(code);
            return;
        }

        // The keys every screen shares, as homelab's global match has them.
        match code {
            KeyCode::Char('q') if !ctrl => {
                self.quit = true;
                return;
            }
            KeyCode::Char('k') | KeyCode::Char('p') if ctrl => {
                self.open_palette();
                return;
            }
            KeyCode::F(2) => {
                self.toggle_motion();
                let now = match self.config.motion {
                    Motion::Full => "full",
                    Motion::Reduced => "reduced",
                };
                self.say(format!("effects → {now}"));
                return;
            }
            KeyCode::Char('h') if !ctrl => {
                self.help_open = true;
                return;
            }
            KeyCode::Tab => {
                let i = ORDER.iter().position(|s| *s == self.screen).unwrap_or(0);
                self.go(ORDER[(i + 1) % ORDER.len()]);
                return;
            }
            KeyCode::BackTab => {
                let i = ORDER.iter().position(|s| *s == self.screen).unwrap_or(0);
                self.go(ORDER[(i + ORDER.len() - 1) % ORDER.len()]);
                return;
            }
            // homelab's own tab order, on the digits and on the symbols an
            // azerty keyboard types without shift. Its sixth tab, the
            // shell, is not rebuilt.
            KeyCode::Char(c) if !ctrl => {
                if let Some(i) = tab_index(c) {
                    if let Some(screen) = HOMELAB.get(i) {
                        self.go(*screen);
                    }
                    return;
                }
            }
            KeyCode::Esc => {
                self.quit = true;
                return;
            }
            _ => {}
        }
        if ctrl {
            return;
        }
        // The theme is the demo's own key, on a letter homelab leaves free.
        if code == KeyCode::Char('t') {
            self.cycle_theme();
            return;
        }
        match self.screen {
            Screen::Settings => {
                if let Some(says) = self.settings.key(code) {
                    self.reveal_ms = 0;
                    self.say(says);
                }
            }
            Screen::Fleet | Screen::Ops => self.stack_key(code),
            Screen::LogStream => self.log_key(key),
            // homelab runs the checks again on r and on enter.
            Screen::Doctor => {
                if matches!(code, KeyCode::Char('r') | KeyCode::Enter) {
                    self.reveal_ms = 0;
                }
            }
            Screen::Dashboard => match code {
                KeyCode::Char('r') => self.reveal_ms = 0,
                _ => self.dashboard_key(code),
            },
            Screen::Console => match code {
                KeyCode::Char('p') => self.open_palette(),
                KeyCode::Char('n') => self.step = (self.step + 1) % WIZARD.len(),
                KeyCode::Char('a') => self.alarm_ms = 0,
                KeyCode::Char('r') => self.reveal_ms = 0,
                _ => {}
            },
            Screen::Effects => match code {
                // The alarm strikes once, so it needs a key to strike again.
                KeyCode::Char('a') => self.alarm_ms = 0,
                KeyCode::Char('r') => self.reveal_ms = 0,
                _ => {}
            },
            Screen::Components => self.components_key(code),
            Screen::Deploy | Screen::Splash => {}
        }
    }

    fn open_palette(&mut self) {
        self.palette_open = true;
        self.palette_query.clear();
        self.palette_sel = 0;
    }

    /// The palette's keys, as homelab's `palette_key`: typing filters,
    /// the arrows walk what matches and wrap, enter runs it.
    fn palette_key(&mut self, code: KeyCode) {
        let labels = crate::overlays::palette_labels();
        let found: Vec<usize> = kp_tui::CommandPalette::new(
            &self.theme,
            &self.palette_query,
            &labels,
            self.palette_sel,
        )
        .matches()
        .into_iter()
        .map(|(i, _)| i)
        .collect();
        let n = found.len();
        match code {
            KeyCode::Esc => self.palette_open = false,
            KeyCode::Char(c) => {
                self.palette_query.push(c);
                self.palette_sel = 0;
            }
            KeyCode::Backspace => {
                self.palette_query.pop();
                self.palette_sel = 0;
            }
            KeyCode::Down if n > 0 => self.palette_sel = (self.palette_sel + 1) % n,
            KeyCode::Up if n > 0 => self.palette_sel = (self.palette_sel + n - 1) % n,
            KeyCode::Enter => {
                self.palette_open = false;
                if let Some(&i) = found.get(self.palette_sel) {
                    self.run(crate::overlays::PALETTE[i].1);
                }
            }
            _ => {}
        }
    }

    /// What a palette entry does, as homelab's `run_action`.
    fn run(&mut self, id: &str) {
        match id {
            "tab.dashboard" => self.go(Screen::Ops),
            "tab.stacks" => self.go(Screen::Fleet),
            "tab.logs" => self.go(Screen::LogStream),
            "tab.doctor" => self.go(Screen::Doctor),
            "tab.settings" => self.go(Screen::Settings),
            "tab.shell" => {
                self.say("the shell tab is not rebuilt — it is a terminal in a terminal")
            }
            "refresh" => self.reveal_ms = 0,
            "doctor" => self.go(Screen::Doctor),
            "op.restore" => self.confirm = Some(crate::overlays::Confirm::new(self.stack_sel)),
            "fx" => self.key(KeyEvent::new(KeyCode::F(2), KeyModifiers::NONE)),
            "help" => self.help_open = true,
            "quit" => self.quit = true,
            op => {
                let name = op.trim_start_matches("op.");
                self.say(format!("{name}: needs a live host — the demo has none"));
            }
        }
    }

    /// homelab's stacks and dashboard tabs share one handler, and so do
    /// their two rebuilds.
    fn stack_key(&mut self, code: KeyCode) {
        let n = crate::fleet::FLEET.len();
        match code {
            KeyCode::Down | KeyCode::Char('j') => self.stack_sel = (self.stack_sel + 1) % n,
            KeyCode::Up | KeyCode::Char('k') => self.stack_sel = (self.stack_sel + n - 1) % n,
            KeyCode::Char('r') => self.reveal_ms = 0,
            KeyCode::Char('R') => {
                self.confirm = Some(crate::overlays::Confirm::new(self.stack_sel));
            }
            KeyCode::Char('p') => {
                self.plan = Some(crate::overlays::Plan {
                    stack: self.stack_sel,
                });
            }
            KeyCode::Char('n') => self.wizard = Some(crate::overlays::Wizard::default()),
            KeyCode::Char(c) => {
                if let Some((_, op)) = HOST_OPS.iter().find(|(k, _)| *k == c) {
                    self.say(format!("{op}: needs a live host — the demo has none"));
                }
            }
            _ => {}
        }
    }

    /// The log screen owns its arrows, its space and its l: every
    /// behaviour homelab's log tab has, and the level filter it does
    /// not [fix-65].
    fn log_key(&mut self, key: KeyEvent) {
        let sources = crate::logstream::sources().len();
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        match key.code {
            KeyCode::Left if shift => self.stream.pan_by(-8),
            KeyCode::Right if shift => self.stream.pan_by(8),
            KeyCode::Left => {
                self.source_sel = (self.source_sel + sources - 1) % sources;
                self.apply_source();
            }
            KeyCode::Right => {
                self.source_sel = (self.source_sel + 1) % sources;
                self.apply_source();
            }
            KeyCode::Up | KeyCode::Char('k') => self.stream.scroll_up(1),
            KeyCode::Down | KeyCode::Char('j') => self.stream.scroll_down(1),
            KeyCode::Char(' ') => self.stream.toggle_pause(),
            KeyCode::Char('l') => self.stream.cycle_filter(),
            // Horizontal scrolling, on keys an azerty keyboard puts where a
            // qwerty one does: h, j, k and l do not move between the two
            // layouts, where the punctuation keys all do. Shift with the
            // arrows does the same [fix-67].
            KeyCode::Char('H') => self.stream.pan_by(-8),
            KeyCode::Char('L') => self.stream.pan_by(8),
            KeyCode::Char('G') | KeyCode::End => self.stream.follow(),
            KeyCode::Char('r') => self.reveal_ms = 0,
            _ => {}
        }
    }

    /// The first round's components screen: its tabs, its two buttons and
    /// the alarm. Its focus moves on `f` now that tab changes the screen.
    fn components_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('a') => self.alarm_ms = 0,
            KeyCode::Char('r') => self.reveal_ms = 0,
            KeyCode::Left => {
                self.tab = (self.tab + TABS.len() - 1) % TABS.len();
                self.reveal_ms = 0;
            }
            KeyCode::Right => {
                self.tab = (self.tab + 1) % TABS.len();
                self.reveal_ms = 0;
            }
            KeyCode::Char('f') => {
                self.focus = (self.focus + 1) % BUTTONS.len();
                self.charge_ms = Some(0);
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.pressed = Some((self.focus, 160));
                self.message = format!("{} pressed", BUTTONS[self.focus].0);
            }
            _ => {}
        }
    }

    fn dashboard_key(&mut self, code: KeyCode) {
        let page = 10;
        let logs = &mut self.dash.logs;
        match code {
            KeyCode::Char('p') => logs.toggle_pause(),
            KeyCode::Char('f') => logs.cycle_filter(),
            KeyCode::Up | KeyCode::Char('k') => logs.scroll_up(1),
            KeyCode::Down | KeyCode::Char('j') => logs.scroll_down(1),
            KeyCode::PageUp => logs.scroll_up(page),
            KeyCode::PageDown => logs.scroll_down(page),
            KeyCode::Home => logs.scroll_up(usize::MAX),
            KeyCode::End => logs.follow(),
            _ => {}
        }
    }

    /// The screen, then whatever homelab would draw over it.
    pub fn draw(&self, frame: &mut Frame) {
        self.draw_screen(frame);
        let th = &self.theme;
        let blink = self.reveal_ms / 33;
        if let Some(p) = &self.plan {
            p.draw(frame, th);
        }
        if let Some(w) = &self.wizard {
            w.draw(frame, th, blink);
        }
        if let Some(c) = &self.confirm {
            c.draw(frame, th, blink);
        }
        if self.help_open {
            crate::overlays::draw_help(frame, th, self.screen_keys());
        }
        if self.palette_open {
            let labels = crate::overlays::palette_labels();
            CommandPalette::new(th, &self.palette_query, &labels, self.palette_sel)
                .size((52, 12))
                .blink(self.reveal_ms)
                .render_over(frame.area(), frame.buffer_mut());
        }
        if let Some((text, _)) = &self.status {
            crate::overlays::draw_status(frame, th, text);
        }
    }

    fn draw_screen(&self, frame: &mut Frame) {
        if self.screen == Screen::Dashboard {
            let header = format!(
                "{PACKAGE_VERSION} · theme {} · colours {} · motion {} · {} fps",
                self.theme.id.name(),
                self.depth.label(),
                if self.config.motion == Motion::Reduced {
                    "reduced"
                } else {
                    "full"
                },
                self.dash.fps
            );
            let view = dashboard::View {
                theme: &self.theme,
                motion: self.config.motion,
                reveal_ms: self.reveal_ms,
                header,
            };
            dashboard::draw(frame, &self.dash, &view);
            return;
        }
        if self.screen == Screen::Console {
            self.draw_console(frame);
            return;
        }
        if self.screen == Screen::Effects {
            self.draw_effects(frame);
            return;
        }
        if self.screen == Screen::Fleet {
            frame.render_widget(Block::new().style(self.theme.base()), frame.area());
            crate::fleet::draw(
                frame,
                &self.theme,
                self.stack_sel,
                self.reveal_ms,
                self.config.motion,
            );
            return;
        }
        if self.screen == Screen::Ops {
            frame.render_widget(Block::new().style(self.theme.base()), frame.area());
            crate::ops::draw(
                frame,
                &self.theme,
                self.stack_sel,
                self.reveal_ms,
                self.config.motion,
            );
            return;
        }
        if self.screen == Screen::Settings {
            frame.render_widget(Block::new().style(self.theme.base()), frame.area());
            crate::settings::draw(
                frame,
                &self.theme,
                &self.settings,
                self.reveal_ms,
                self.config.motion,
            );
            return;
        }
        if self.screen == Screen::LogStream {
            frame.render_widget(Block::new().style(self.theme.base()), frame.area());
            crate::logstream::draw(
                frame,
                &self.theme,
                &self.stream,
                self.source_sel,
                self.reveal_ms,
                self.config.motion,
            );
            return;
        }
        if self.screen == Screen::Doctor {
            frame.render_widget(Block::new().style(self.theme.base()), frame.area());
            crate::doctor::draw(
                frame,
                &self.theme,
                self.reveal_ms < 400,
                self.reveal_ms,
                self.config.motion,
            );
            return;
        }
        if self.screen == Screen::Splash {
            frame.render_widget(Block::new().style(self.theme.base()), frame.area());
            crate::splash::draw(frame, &self.theme, self.reveal_ms, self.config.motion);
            return;
        }
        if self.screen == Screen::Deploy {
            // The overlay sits over whatever was on screen; the components
            // page is what homelab's operator would have been looking at.
            frame.render_widget(Block::new().style(self.theme.base()), frame.area());
            crate::ops::draw(
                frame,
                &self.theme,
                self.stack_sel,
                self.reveal_ms,
                self.config.motion,
            );
            crate::deploy::draw(
                frame,
                &self.theme,
                &self.deploy,
                self.reveal_ms,
                self.config.motion,
            );
            return;
        }
        let th = &self.theme;
        let buf_area = frame.area();
        frame.render_widget(Block::new().style(th.base()), buf_area);
        let a = areas(buf_area);

        let header = Line::from(vec![
            Span::styled(
                th.label("kp-themes"),
                Style::new().fg(th.c.primary).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(
                    "  {PACKAGE_VERSION} · theme {} · colours {} · motion {}",
                    th.id.name(),
                    self.depth.label(),
                    if self.config.motion == Motion::Reduced {
                        "reduced"
                    } else {
                        "full"
                    }
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
        let [head, _, text, field] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(2),
            Constraint::Length(1),
        ])
        .areas(inner.inner(ratatui::layout::Margin::new(1, 0)));
        // With `--features tachyonfx` the headline is drawn whole here and
        // main.rs runs the tachyonfx effect over `App::headline`.
        let motion = self.config.motion;
        frame.render_widget(RevealText::new(th, headline, self.reveal_ms, motion), head);
        frame.render_widget(
            Paragraph::new(body)
                .wrap(Wrap { trim: true })
                .style(Style::new().fg(th.c.card_foreground)),
            text,
        );
        let prompt = th.label("filter");
        let value = "web";
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    format!("{prompt}: "),
                    Style::new().fg(th.c.muted_foreground),
                ),
                Span::styled(value, Style::new().fg(th.c.foreground)),
            ])),
            field,
        );
        let col = field.x + (prompt.chars().count() + 2 + value.len()) as u16;
        frame.set_cursor_position((col.min(field.right().saturating_sub(1)), field.y));

        // Actions panel: focus follows f, Enter presses.
        let panel = Panel::new(th, "Actions").focused(true);
        let inner = panel.block().inner(a.actions);
        frame.render_widget(panel, a.actions);
        let rows = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .areas::<4>(inner.inner(ratatui::layout::Margin::new(2, 1)));
        for (i, (label, kind)) in BUTTONS.iter().enumerate() {
            let state = match self.pressed {
                Some((p, _)) if p == i => ButtonState::Pressed,
                _ if self.focus == i => ButtonState::Focus,
                _ => ButtonState::Rest,
            };
            let charge = match (self.focus == i, self.config.motion) {
                (true, Motion::Full) => self.charge_ms.map(|c| c as f32 / CHARGE_MS as f32),
                _ => None,
            };
            let button = Button::new(th, label)
                .kind(*kind)
                .state(state)
                .charge(charge);
            let slot = fit(rows[i * 2], button.width());
            frame.render_widget(button, slot);
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
            let button = Button::new(th, label).state(state);
            let slot = fit(*cell, button.width());
            frame.render_widget(button, slot);
        }

        let keys = format!(
            " tab screen · t theme · ←/→ tab · f focus · Enter press · r replay · F2 motion · h help · q quit   {}",
            self.message
        );
        frame.render_widget(
            Paragraph::new(keys).style(
                Style::new()
                    .bg(th.c.secondary)
                    .fg(th.c.secondary_foreground),
            ),
            a.footer,
        );
        let _ = fx::GLYPHS; // the effect module is part of the public surface
    }
}

/// The console screen: the four components homelab writes out by hand, in
/// whichever theme is on. The numbers are fixed so the screen can be
/// compared between themes rather than between moments.
impl App {
    fn draw_console(&self, frame: &mut Frame) {
        let th = &self.theme;
        let screen = frame.area();
        let stage = Stage::new(self.reveal_ms, self.config.motion);
        frame.render_widget(Block::new().style(th.base()), screen);
        let rows = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(5),
            Constraint::Length(4),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .areas::<7>(screen);
        let rail_row = rows[1];
        let wizard_row = rows[2];
        let rows = [rows[0], rows[3], rows[4], rows[5], rows[6]];

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    th.label("console"),
                    Style::new().fg(th.c.primary).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(
                        "  {} · what homelab hand-rolls, from the crate",
                        th.id.name()
                    ),
                    Style::new().fg(th.c.muted_foreground),
                ),
            ]))
            .style(Style::new().bg(th.c.background)),
            rows[0],
        );

        // The rail runs out of the left over the ground beat.
        frame.render_widget(Rail::new(th).grown(stage.ground()), rail_row);
        frame.render_widget(Stepper::new(th, &WIZARD, self.step), wizard_row);

        // Three meters, one under each threshold and one over. The values
        // roll to their reading rather than snapping to it.
        let panel = Panel::new(th, "Capacity").stage(stage);
        let inner = panel.block().inner(rows[1]);
        frame.render_widget(panel, rows[1]);
        let meters = Layout::vertical([Constraint::Length(1); 3]).areas::<3>(inner);
        for (area, (label, value)) in
            meters
                .iter()
                .zip([("ram", 0.42_f32), ("ssd", 0.78), ("load", 0.94)])
        {
            let shown = roll(
                0.0,
                value as f64,
                self.reveal_ms,
                th.id.fx_duration_ms(),
                self.config.motion,
            ) as f32;
            frame.render_widget(Meter::new(th, label, shown), *area);
        }

        // Two fields; the second has the caret, blinking on the theme's own
        // clock where the theme blinks.
        let panel = Panel::new(th, "Filter").focused(true).stage(stage);
        let inner = panel.block().inner(rows[2]);
        frame.render_widget(panel, rows[2]);
        let fields = Layout::vertical([Constraint::Length(1); 2]).areas::<2>(inner);
        frame.render_widget(Field::new(th, "stack", "media"), fields[0]);
        frame.render_widget(
            Field::new(th, "filter", "web")
                .focused(true)
                .blink(self.reveal_ms / 33),
            fields[1],
        );

        // The stacks, one of them in hand, beside the help overlay that is
        // drawn from the same keymap as the footer.
        let [left, right] =
            Layout::horizontal([Constraint::Percentage(48), Constraint::Percentage(52)])
                .areas(rows[3]);
        let panel = Panel::new(th, "Stacks").focused(true).stage(stage);
        let inner = panel.block().inner(left);
        frame.render_widget(panel, left);
        let items: Vec<Line<'static>> = STACKS
            .iter()
            .map(|(name, note)| {
                Line::from(vec![
                    Span::styled(
                        (*name).to_string(),
                        Style::new().fg(source_colour(th, name)),
                    ),
                    Span::styled(format!("  {note}"), Style::new().fg(th.c.muted_foreground)),
                ])
            })
            .collect();
        frame.render_widget(SelectList::new(th, &items, self.stack_sel), inner);

        let hints = KeyHints::new(th, CONSOLE_KEYS);
        let panel = Panel::new(th, "Keys").stage(stage);
        let inner = panel.block().inner(right);
        frame.render_widget(panel, right);
        frame.render_widget(
            Paragraph::new(hints.overlay()).style(Style::new().bg(th.c.card)),
            inner,
        );

        frame.render_widget(KeyHints::new(th, CONSOLE_KEYS), rows[4]);

        // And a dialog over all of it, as a restore would be.
        let popup = Popup::new(th, "Restore backup", (52, 7)).kind(PopupKind::Danger);
        let inner = popup.render_over(screen, frame.buffer_mut());
        let lines = vec![
            Line::from(Span::styled(
                "This replaces the running stack with 2026-09-16.",
                Style::new().fg(th.c.popover_foreground),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Type the stack's name to confirm:",
                Style::new().fg(th.c.muted_foreground),
            )),
        ];
        frame.render_widget(
            Paragraph::new(lines).style(Style::new().bg(th.c.popover)),
            inner,
        );
        let field = Rect {
            y: inner.bottom() - 1,
            height: 1,
            ..inner
        };
        frame.render_widget(
            Field::new(th, "name", "medi")
                .focused(true)
                .blink(self.reveal_ms / 33),
            field,
        );
    }
}

impl App {
    /// The effects screen: everything on it is the theme's, not the
    /// screen's. homelab's `client/src/tui/fx.rs` writes the same six
    /// effects against eighteen colour literals and one fixed look.
    fn draw_effects(&self, frame: &mut Frame) {
        let th = &self.theme;
        let motion = self.config.motion;
        let screen = frame.area();
        let p = th.id.palette();
        frame.render_widget(Block::new().style(th.base()), screen);
        // The ground first: the register's static texture, and the sweep
        // for the one register that declares one.
        Surface::new(th, p.background)
            .at(self.reveal_ms, motion)
            .paint(screen, frame.buffer_mut());

        let rows = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(5),
            Constraint::Min(6),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas::<5>(screen);

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    th.label("effects"),
                    Style::new().fg(th.c.primary).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  {} · {}", th.id.name(), describe(th)),
                    Style::new().fg(th.c.muted_foreground),
                ),
            ])),
            rows[0],
        );

        // The alarm, as this register raises it.
        frame.render_widget(
            AlarmPanel::new(
                th,
                "Power lost",
                "ups on battery · 14 minutes of runtime left",
            )
            .at(self.alarm_ms, motion),
            rows[1],
        );

        let [left, right] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(rows[2]);

        // A card of its own, so the texture is visible on a plate as well
        // as on the page, and the sweep crosses it on its own clock.
        let panel = Panel::new(th, "Texture");
        let inner = panel.block().inner(left);
        frame.render_widget(panel, left);
        Surface::new(th, p.card)
            .at(self.reveal_ms, motion)
            .id(7)
            .paint(inner, frame.buffer_mut());
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled(
                    texture_note(th),
                    Style::new().fg(th.c.card_foreground),
                )),
                Line::from(Span::styled(
                    match th.a.fx.sweep {
                        Some(s) => format!(
                            "a lit row crosses every {:.0} s",
                            s.period_ms as f32 / 1000.0
                        ),
                        None => "nothing crosses it: the register's texture is static".into(),
                    },
                    Style::new().fg(th.c.muted_foreground),
                )),
            ])
            .wrap(Wrap { trim: true }),
            inner,
        );

        // The spinner turning, beside the reveal the theme already had.
        let panel = Panel::new(th, "Waiting");
        let inner = panel.block().inner(right);
        frame.render_widget(panel, right);
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(vec![
                    Span::styled(
                        spinner(th, self.reveal_ms, motion),
                        Style::new().fg(th.c.primary),
                    ),
                    Span::styled(
                        format!("  {}", "deploying 2.4 to both hosts"),
                        Style::new().fg(th.c.card_foreground),
                    ),
                ]),
                Line::from(Span::styled(
                    format!(
                        "spinner {:?} · glow {} · glitch {}",
                        th.a.fx.spinner,
                        match th.a.fx.alarm.glow_ms {
                            Some(ms) => format!("{ms} ms"),
                            None => "none".into(),
                        },
                        if th.a.fx.alarm.glitch.is_some() {
                            "yes"
                        } else {
                            "no"
                        }
                    ),
                    Style::new().fg(th.c.muted_foreground),
                )),
            ])
            .wrap(Wrap { trim: true }),
            inner,
        );

        frame.render_widget(
            Ticker::new(th, &self.ticker).at(self.reveal_ms, motion),
            rows[3],
        );
        frame.render_widget(KeyHints::new(th, EFFECT_KEYS), rows[4]);
    }
}

/// What this register declares, in one clause, so the screen says what it
/// is showing rather than only showing it.
fn describe(th: &Theme) -> String {
    let alarm = match (th.a.fx.alarm.strike, th.a.fx.alarm.glitch) {
        (Some(_), Some(_)) => "strikes and comes apart",
        (Some(_), None) => "strikes",
        (None, Some(_)) => "comes apart",
        (None, None) => match th.a.fx.alarm.glow_ms {
            Some(_) => "settles, then glows",
            None => "settles, and holds",
        },
    };
    format!("the alarm {alarm}")
}

fn texture_note(th: &Theme) -> String {
    match th.a.fx.texture {
        Texture::None => "no texture layer: this register paints its ground flat".into(),
        Texture::Scanline { every } => format!("scanlines: one row in {every}"),
        Texture::Grid { cols, rows } => format!("a drafting grid: {cols} by {rows} cells"),
        Texture::Dots { every } => format!("halftone dots: one in {every}, every other row"),
        Texture::Diagonal { every } => format!("a twill: one diagonal in {every}"),
    }
}

const EFFECT_KEYS: &[(&str, &str)] = &[
    ("a", "alarm"),
    ("tab", "screen"),
    ("t", "theme"),
    ("F2", "motion"),
    ("q", "quit"),
];

/// The five steps homelab's create-container wizard walks
/// (`client/src/tui/view/mod.rs:242`).
const WIZARD: [&str; 5] = ["Preset", "Name", "Resources", "Storage", "Review"];

/// What the console's list holds, and what the palette can find.
const STACKS: [(&str, &str); 5] = [
    ("media", "8 containers"),
    ("web", "3 containers"),
    ("backup", "idle"),
    ("monitoring", "2 containers"),
    ("dns", "1 container"),
];

/// One keymap: the footer and the overlay both read this.
const CONSOLE_KEYS: &[(&str, &str)] = &[
    ("p", "palette"),
    ("n", "step"),
    ("tab", "screen"),
    ("t", "theme"),
    ("F2", "effects"),
    ("h", "help"),
    ("q", "quit"),
];

#[cfg(test)]
mod tests {
    //! Every key homelab binds on a rebuilt screen, pressed here and read
    //! back, so the key inventory in docs/HOMELAB_PROOF.md is code and not
    //! a promise [fix-68].
    use super::*;

    fn app(screen: Screen) -> App {
        let mut app = App::new(Config::default(), ColorDepth::TrueColor, None);
        app.screen = screen;
        app
    }

    fn press(app: &mut App, code: KeyCode) {
        app.key(KeyEvent::new(code, KeyModifiers::NONE));
    }

    fn typed(app: &mut App, text: &str) {
        for c in text.chars() {
            press(app, KeyCode::Char(c));
        }
    }

    fn status(app: &App) -> &str {
        app.status.as_ref().map(|(s, _)| s.as_str()).unwrap_or("")
    }

    #[test]
    fn enter_on_the_doctor_runs_the_checks_again() {
        let mut app = app(Screen::Doctor);
        app.tick(1_000);
        press(&mut app, KeyCode::Enter);
        assert_eq!(app.reveal_ms, 0, "enter did not re-run the checks");
        assert!(app.pressed.is_none(), "enter pressed a components button");
        assert!(app.message.is_empty());
    }

    #[test]
    fn any_key_leaves_the_splash_for_the_dashboard() {
        let mut app = app(Screen::Splash);
        press(&mut app, KeyCode::Char('x'));
        assert_eq!(app.screen, Screen::Ops);
        assert!(!app.quit);
    }

    #[test]
    fn tab_digits_and_azerty_symbols_walk_homelabs_order() {
        let mut app = app(Screen::Ops);
        press(&mut app, KeyCode::Tab);
        assert_eq!(app.screen, Screen::Fleet);
        press(&mut app, KeyCode::BackTab);
        assert_eq!(app.screen, Screen::Ops);
        for (key, screen) in [
            ('3', Screen::LogStream),
            ('é', Screen::Fleet),
            ('\'', Screen::Doctor),
            ('5', Screen::Settings),
            ('&', Screen::Ops),
        ] {
            press(&mut app, KeyCode::Char(key));
            assert_eq!(app.screen, screen, "{key}");
        }
    }

    #[test]
    fn a_waiting_question_swallows_every_key_but_its_two_answers() {
        let mut app = app(Screen::Deploy);
        app.back = Screen::Fleet;
        for code in [KeyCode::Char('q'), KeyCode::Up, KeyCode::Esc, KeyCode::Tab] {
            press(&mut app, code);
            assert_eq!(app.screen, Screen::Deploy);
            assert!(!app.quit);
        }
        press(&mut app, KeyCode::Char('a'));
        assert_eq!(app.deploy.allowed, Some(true));
        assert_eq!(
            app.screen,
            Screen::Deploy,
            "the answer leaves the window up"
        );
    }

    #[test]
    fn the_deploy_window_scrolls_backgrounds_and_closes() {
        let mut app = app(Screen::Deploy);
        app.back = Screen::Fleet;
        press(&mut app, KeyCode::Char('s'));
        press(&mut app, KeyCode::Up);
        press(&mut app, KeyCode::Up);
        assert_eq!(app.deploy.scroll, 2);
        press(&mut app, KeyCode::Down);
        assert_eq!(app.deploy.scroll, 1);
        // Running: Esc sends it to the background and says so.
        press(&mut app, KeyCode::Esc);
        assert_eq!(app.screen, Screen::Fleet);
        assert!(status(&app).contains("keeps running"));
        assert!(!app.quit);
        // It runs on behind the screen, and enter closes it once done.
        app.tick(crate::deploy::FINISH_MS);
        app.go(Screen::Deploy);
        assert!(app.deploy.done);
        press(&mut app, KeyCode::Enter);
        assert_eq!(app.screen, Screen::Fleet);
        assert!(app.deploy.asking, "a closed deploy starts over");
    }

    #[test]
    fn settings_steps_values_adds_and_deletes_tiers() {
        let mut app = app(Screen::Settings);
        let alarm = app.alarm_ms;
        press(&mut app, KeyCode::Right);
        assert_eq!(app.settings.edit.backup_hour, Some(4));
        press(&mut app, KeyCode::Char('a'));
        assert_eq!(app.settings.edit.tiers.len(), 4, "a adds a tier");
        assert_eq!(app.alarm_ms, alarm, "a is not the alarm here");
        press(&mut app, KeyCode::Down);
        press(&mut app, KeyCode::Right);
        assert_eq!(app.settings.edit.tiers[0].every_days, 2);
        press(&mut app, KeyCode::Char('d'));
        assert_eq!(
            app.settings.edit.tiers.len(),
            3,
            "d deletes the tier in hand"
        );
    }

    #[test]
    fn the_webhook_is_typed_and_digits_do_not_jump() {
        let mut app = app(Screen::Settings);
        let last = app.settings.rows() - 1;
        for _ in 0..last {
            press(&mut app, KeyCode::Down);
        }
        press(&mut app, KeyCode::Enter);
        assert!(app.settings.typing());
        typed(&mut app, "https://hook/1");
        assert_eq!(app.screen, Screen::Settings, "a digit switched the screen");
        press(&mut app, KeyCode::Enter);
        assert_eq!(app.settings.edit.webhook.as_deref(), Some("https://hook/1"));
    }

    #[test]
    fn capital_s_saves_and_r_reloads() {
        let mut app = app(Screen::Settings);
        assert_ne!(app.settings.host, app.settings.edit);
        press(&mut app, KeyCode::Char('S'));
        assert_eq!(app.settings.host, app.settings.edit);
        assert_eq!(app.screen, Screen::Settings, "S left the screen");
        press(&mut app, KeyCode::Right);
        press(&mut app, KeyCode::Char('r'));
        assert_eq!(
            app.settings.host, app.settings.edit,
            "r reads the host back"
        );
    }

    #[test]
    fn a_plan_is_shown_first_and_enter_runs_the_deploy() {
        let mut app = app(Screen::Fleet);
        press(&mut app, KeyCode::Char('p'));
        assert!(app.plan.is_some());
        press(&mut app, KeyCode::Esc);
        assert!(app.plan.is_none());
        assert!(!app.quit, "Esc on a plan quit the demo");
        press(&mut app, KeyCode::Char('p'));
        press(&mut app, KeyCode::Enter);
        assert_eq!(app.screen, Screen::Deploy);
        assert_eq!(app.back, Screen::Fleet);
    }

    #[test]
    fn a_restore_asks_for_the_stack_name() {
        let mut app = app(Screen::Ops);
        app.stack_sel = 0; // media
        press(&mut app, KeyCode::Char('R'));
        typed(&mut app, "medi");
        press(&mut app, KeyCode::Enter);
        assert!(app.confirm.is_none());
        assert!(status(&app).contains("does not match"));
        press(&mut app, KeyCode::Char('R'));
        typed(&mut app, "media");
        press(&mut app, KeyCode::Enter);
        assert!(status(&app).contains("live host"));
    }

    #[test]
    fn the_wizard_walks_its_five_steps() {
        let mut app = app(Screen::Fleet);
        press(&mut app, KeyCode::Char('n'));
        press(&mut app, KeyCode::Enter); // preset: media
        press(&mut app, KeyCode::Backspace);
        typed(&mut app, "2");
        press(&mut app, KeyCode::Enter); // name: medi2
        let w = app.wizard.as_ref().expect("wizard open");
        assert_eq!((w.step, w.name.as_str(), w.ram), (2, "medi2", 4096));
        press(&mut app, KeyCode::Right);
        assert_eq!(app.wizard.as_ref().map(|w| w.ram), Some(5120));
        press(&mut app, KeyCode::Enter); // storage
        press(&mut app, KeyCode::Char(' '));
        assert_eq!(app.wizard.as_ref().map(|w| w.no_data[0]), Some(true));
        press(&mut app, KeyCode::Enter); // review
        press(&mut app, KeyCode::Enter);
        assert!(app.wizard.is_none());
        assert!(status(&app).contains("medi2"));
    }

    #[test]
    fn the_palette_opens_anywhere_and_enter_runs_the_action() {
        let mut app = app(Screen::Settings);
        app.key(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL));
        assert!(app.palette_open);
        typed(&mut app, "go: doc");
        press(&mut app, KeyCode::Enter);
        assert!(!app.palette_open);
        assert_eq!(app.screen, Screen::Doctor);
        app.key(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL));
        press(&mut app, KeyCode::Up);
        assert_eq!(
            app.palette_sel,
            crate::overlays::PALETTE.len() - 1,
            "up wraps"
        );
    }

    #[test]
    fn h_opens_the_help_and_esc_closes_it_without_quitting() {
        let mut app = app(Screen::Fleet);
        press(&mut app, KeyCode::Char('h'));
        assert!(app.help_open);
        press(&mut app, KeyCode::Char('x'));
        assert!(app.help_open);
        press(&mut app, KeyCode::Esc);
        assert!(!app.help_open);
        assert!(!app.quit);
    }

    #[test]
    fn f2_turns_the_effects_down_and_up() {
        let mut app = app(Screen::Ops);
        let before = app.config.motion;
        press(&mut app, KeyCode::F(2));
        assert_ne!(app.config.motion, before);
        assert!(status(&app).starts_with("effects"));
    }

    #[test]
    fn overlays_draw_over_every_screen() {
        use ratatui::{Terminal, backend::TestBackend};
        let mut app = app(Screen::Fleet);
        app.help_open = true;
        app.plan = Some(crate::overlays::Plan { stack: 0 });
        app.wizard = Some(crate::overlays::Wizard::default());
        app.confirm = Some(crate::overlays::Confirm::new(0));
        app.palette_open = true;
        app.status = Some(("saved".into(), 0));
        let mut term = Terminal::new(TestBackend::new(100, 30)).expect("terminal");
        term.draw(|f| app.draw(f)).expect("draw");
    }
}
