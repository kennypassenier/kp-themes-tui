//! Headless rendering with `TestBackend`: one frame per theme, the switch,
//! the persistence, the fallbacks and the reduced-motion path.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use kp_tui::{
    anatomy::Reveal,
    app::{App, areas},
    color::{ColorDepth, nearest_256},
    config::Config,
    fx::{self, Motion},
    theme::{Theme, ThemeId},
    widgets::{Button, ButtonState},
};
use ratatui::{Terminal, backend::TestBackend, buffer::Buffer, layout::Rect, style::Color, widgets::Widget};

const W: u16 = 100;
const H: u16 = 30;

fn render(app: &App) -> Buffer {
    let mut term = Terminal::new(TestBackend::new(W, H)).unwrap();
    term.draw(|f| app.draw(f)).unwrap();
    term.backend().buffer().clone()
}

fn rgb(c: kp_tui::color::Rgb) -> Color {
    Color::Rgb(c.0, c.1, c.2)
}

fn app(theme: ThemeId) -> App {
    App::new(Config { theme, motion: Motion::Reduced }, ColorDepth::TrueColor, None)
}

#[test]
fn one_frame_per_theme_has_its_border_glyph_and_colours() {
    let area = Rect::new(0, 0, W, H);
    let a = areas(area);
    for (id, corner, focused_corner) in
        [(ThemeId::Formal, "╭", "╔"), (ThemeId::Cyberpunk, "┌", "┏"), (ThemeId::Terminal, "┌", "┌")]
    {
        let buf = render(&app(id));
        let p = id.palette();
        let story = &buf[(a.story.x, a.story.y)];
        assert_eq!(story.symbol(), corner, "{}: story panel corner", id.name());
        assert_eq!(story.fg, rgb(p.border_strong), "{}: rest border colour", id.name());
        let actions = &buf[(a.actions.x, a.actions.y)];
        assert_eq!(actions.symbol(), focused_corner, "{}: focused panel corner", id.name());
        assert_eq!(actions.fg, rgb(p.ring), "{}: focus ring colour", id.name());
        assert_eq!(buf[(a.header.right() - 1, a.header.y)].bg, rgb(p.background), "{}: ground", id.name());
        assert_eq!(buf[(a.story.x + 1, a.story.y + 1)].bg, rgb(p.card), "{}: card", id.name());
    }
}

#[test]
fn cyberpunk_labels_are_prefixed_and_uppercase() {
    let buf = render(&app(ThemeId::Cyberpunk));
    let top: String = (0..W).map(|x| buf[(x, areas(Rect::new(0, 0, W, H)).story.y)].symbol().to_string()).collect();
    assert!(top.contains("/// RELEASE"), "{top}");
}

#[test]
fn a_key_switches_the_theme_for_the_next_frame_and_persists_it() {
    let path = std::env::temp_dir().join(format!("kp-tui-demo-test-{}/config", std::process::id()));
    let mut app =
        App::new(Config { theme: ThemeId::Formal, motion: Motion::Full }, ColorDepth::TrueColor, Some(path.clone()));
    let before = render(&app);
    app.key(KeyEvent::new_with_kind(KeyCode::Char('t'), KeyModifiers::NONE, KeyEventKind::Press));
    let after = render(&app);
    assert_eq!(before[(W - 1, 0)].bg, rgb(ThemeId::Formal.palette().background));
    assert_eq!(after[(W - 1, 0)].bg, rgb(ThemeId::Cyberpunk.palette().background));
    assert_eq!(Config::load(&path).theme, ThemeId::Cyberpunk);
    app.key(KeyEvent::new_with_kind(KeyCode::Char('m'), KeyModifiers::NONE, KeyEventKind::Press));
    assert_eq!(Config::load(&path), Config { theme: ThemeId::Cyberpunk, motion: Motion::Reduced });
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

#[test]
fn button_states_differ_by_plate_and_frame() {
    for id in ThemeId::ALL {
        let th = Theme::new(id, ColorDepth::TrueColor);
        let p = id.palette();
        let draw = |state| {
            let mut buf = Buffer::empty(Rect::new(0, 0, 14, 3));
            Button::new(&th, "Deploy").state(state).render(buf.area, &mut buf);
            buf
        };
        let (rest, focus, pressed) = (draw(ButtonState::Rest), draw(ButtonState::Focus), draw(ButtonState::Pressed));
        assert_eq!(rest[(1, 1)].bg, rgb(p.primary), "{}", id.name());
        assert_eq!(focus[(1, 1)].bg, rgb(p.primary_hover), "{}", id.name());
        assert_eq!(pressed[(1, 1)].bg, rgb(p.primary_active), "{}", id.name());
        assert_eq!(focus[(0, 0)].fg, rgb(p.ring), "{}", id.name());
    }
    let th = Theme::new(ThemeId::Cyberpunk, ColorDepth::TrueColor);
    let mut buf = Buffer::empty(Rect::new(0, 0, 14, 3));
    Button::new(&th, "Deploy").render(buf.area, &mut buf);
    assert_eq!(buf[(13, 2)].symbol(), "◢", "the notch");
}

#[test]
fn fallbacks_resolve_to_indexed_and_named_colours() {
    let t256 = Theme::new(ThemeId::Cyberpunk, ColorDepth::Ansi256);
    assert!(matches!(t256.c.primary, Color::Indexed(i) if i >= 16));
    // hsl(56, 98%, 51%) = rgb(253, 236, 8) by hand; nearest in the cube is (255, 255, 0), index 226.
    assert_eq!(nearest_256(ThemeId::Cyberpunk.palette().primary), 226);
    let t16 = Theme::new(ThemeId::Terminal, ColorDepth::Ansi16);
    assert_eq!(t16.c.background, Color::Reset);
    assert!(matches!(t16.c.primary, Color::Green | Color::LightGreen));
    assert!(matches!(t16.c.destructive, Color::Red | Color::LightRed));
    let f16 = Theme::new(ThemeId::Formal, ColorDepth::Ansi16);
    assert!(matches!(f16.c.primary, Color::Blue | Color::LightBlue));
    // Every theme still renders a frame in 256 and 16 colours.
    for id in ThemeId::ALL {
        for depth in [ColorDepth::Ansi256, ColorDepth::Ansi16] {
            let buf = render(&App::new(Config { theme: id, motion: Motion::Full }, depth, None));
            assert!(buf.content.iter().all(|c| !matches!(c.fg, Color::Rgb(..)) && !matches!(c.bg, Color::Rgb(..))));
        }
    }
}

#[test]
fn colour_depth_detection() {
    let env = |pairs: &'static [(&'static str, &'static str)]| {
        move |k: &str| pairs.iter().find(|(n, _)| *n == k).map(|(_, v)| v.to_string())
    };
    assert_eq!(ColorDepth::detect(None, env(&[("COLORTERM", "truecolor")])), ColorDepth::TrueColor);
    assert_eq!(ColorDepth::detect(None, env(&[("TERM", "xterm-256color")])), ColorDepth::Ansi256);
    assert_eq!(ColorDepth::detect(None, env(&[("TERM", "linux")])), ColorDepth::Ansi16);
    assert_eq!(ColorDepth::detect(Some("16"), env(&[("COLORTERM", "24bit")])), ColorDepth::Ansi16);
    assert_eq!(ColorDepth::detect(None, env(&[("KP_COLORS", "256"), ("COLORTERM", "24bit")])), ColorDepth::Ansi256);
}

#[test]
fn reduced_motion_shows_the_final_state_on_the_first_frame() {
    for id in ThemeId::ALL {
        let f = fx::frame("Version 2.4 is ready", id.anatomy().reveal, 0, Motion::Reduced);
        assert_eq!((f.text.as_str(), f.caret, f.opacity, f.done), ("Version 2.4 is ready", None, 1.0, true));
    }
}

#[test]
fn each_reveal_runs_its_own_routine() {
    let text = "Version 2.4";
    // terminal types: nothing at 0 ms but the caret, three glyphs at 110 ms.
    let r = ThemeId::Terminal.anatomy().reveal;
    assert_eq!(fx::frame(text, r, 0, Motion::Full).caret, Some(0));
    assert_eq!(fx::frame(text, r, 110, Motion::Full).text, "Ver");
    // cyberpunk deciphers through the web's glyph set, never a block glyph.
    let r = ThemeId::Cyberpunk.anatomy().reveal;
    let mid = fx::frame(text, r, 100, Motion::Full).text;
    assert_eq!(mid.chars().count(), text.chars().count());
    assert!(mid.chars().zip(text.chars()).all(|(m, t)| m == t || fx::GLYPHS.contains(&m)));
    assert!(!mid.contains(['░', '▒', '▓']));
    assert_eq!(fx::frame(text, r, fx::duration_ms(r, text), Motion::Full).text, text);
    // formal never touches a character: only the opacity moves.
    let r = ThemeId::Formal.anatomy().reveal;
    assert!(matches!(r, Reveal::Arrive { ms: 450 }));
    let f = fx::frame(text, r, 100, Motion::Full);
    assert_eq!(f.text, text);
    assert!(f.opacity > 0.0 && f.opacity < 1.0);
}
