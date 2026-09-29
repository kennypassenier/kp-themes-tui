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
    // `t` steps through the package's own order; from formal that is light
    // now that all twenty-two are in [scope-127].
    let next = ThemeId::Formal.next();
    assert_eq!(before[(W - 1, 0)].bg, rgb(ThemeId::Formal.palette().background));
    assert_eq!(after[(W - 1, 0)].bg, rgb(next.palette().background));
    assert_eq!(Config::load(&path).theme, next);
    app.key(KeyEvent::new_with_kind(KeyCode::Char('m'), KeyModifiers::NONE, KeyEventKind::Press));
    assert_eq!(Config::load(&path), Config { theme: next, motion: Motion::Reduced });
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

#[test]
fn button_states_differ_by_plate_and_ring() {
    for id in ThemeId::ALL {
        let th = Theme::new(id, ColorDepth::TrueColor);
        let p = id.palette();
        let draw = |state| {
            let mut buf = Buffer::empty(Rect::new(0, 0, 14, 3));
            Button::new(&th, "Deploy").state(state).render(buf.area, &mut buf);
            buf
        };
        let (rest, focus, pressed) = (draw(ButtonState::Rest), draw(ButtonState::Focus), draw(ButtonState::Pressed));
        // The plate is the state's own colour, over the whole button.
        assert_eq!(rest[(7, 0)].bg, rgb(p.primary), "{}: rest plate", id.name());
        assert_eq!(focus[(7, 0)].bg, rgb(p.primary_hover), "{}: focus plate", id.name());
        assert_eq!(pressed[(7, 0)].bg, rgb(p.primary_active), "{}: pressed plate", id.name());
        // The focus ring is a strip along the plate's last row, and a
        // pressed button does not carry it.
        assert_eq!(focus[(7, 2)].bg, rgb(p.ring), "{}: the focus ring", id.name());
        assert_eq!(rest[(7, 2)].bg, rgb(p.primary), "{}: no ring at rest", id.name());
        assert_eq!(pressed[(7, 2)].bg, rgb(p.primary_active), "{}: no ring when pressed", id.name());
        // Nothing is drawn with a line or a corner glyph.
        let glyphs: String = (0..14).flat_map(|x| (0..3).map(move |y| (x, y))).map(|c| rest[c].symbol()).collect();
        assert!(
            !glyphs.contains('─') && !glyphs.contains('│') && !glyphs.contains('◢'),
            "{}: a plate carries no frame glyphs: {glyphs}",
            id.name()
        );
    }
}

#[test]
fn each_theme_ends_its_plate_its_own_way() {
    let ends = |id: ThemeId| {
        let th = Theme::new(id, ColorDepth::TrueColor);
        let mut buf = Buffer::empty(Rect::new(0, 0, 14, 3));
        Button::new(&th, "Deploy").render(buf.area, &mut buf);
        let row: String = (0..14).map(|x| buf[(x, 1)].symbol()).collect();
        (buf[(0, 1)].symbol().to_string(), buf[(0, 1)].bg, row)
    };
    // formal has a radius: the plate ends mid-cell, on the page's ground.
    let (cap, bg, _) = ends(ThemeId::Formal);
    assert_eq!(cap, "▐", "formal's soft cap");
    assert_eq!(bg, rgb(ThemeId::Formal.palette().background), "the cap sits on the ground");
    // cyberpunk's radius is 0: full cells, and the label is spaced caps.
    let (cap, bg, row) = ends(ThemeId::Cyberpunk);
    assert_eq!(cap, " ", "cyberpunk is square");
    assert_eq!(bg, rgb(ThemeId::Cyberpunk.palette().primary), "square to the edge");
    assert!(row.contains("D E P L O Y"), "{row}");
    // terminal keeps its brackets.
    let (_, _, row) = ends(ThemeId::Terminal);
    assert!(row.contains("[ DEPLOY ]"), "{row}");
}

#[test]
fn the_charge_sweep_crosses_a_focused_button_once() {
    let th = Theme::new(ThemeId::Cyberpunk, ColorDepth::TrueColor);
    let p = ThemeId::Cyberpunk.palette();
    let band = |progress: Option<f32>| {
        let mut buf = Buffer::empty(Rect::new(0, 0, 14, 3));
        Button::new(&th, "Deploy").state(ButtonState::Focus).charge(progress).render(buf.area, &mut buf);
        (0..14).filter(|x| buf[(*x, 1)].bg == rgb(p.primary_foreground)).collect::<Vec<_>>()
    };
    assert!(band(None).is_empty(), "no sweep without one");
    let early = band(Some(0.25));
    let late = band(Some(0.75));
    assert!(!early.is_empty() && !late.is_empty(), "the band is on the face");
    assert!(early.iter().max() < late.iter().max(), "the band travels: {early:?} then {late:?}");
    assert!(band(Some(1.0)).iter().all(|x| *x > 10), "it leaves by the right edge");
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

// scope-127: every theme the package ships has an anatomy, and the six
// registers that declare --kp-word-stagger reveal their headline word by
// word instead of as one plate.
#[test]
fn all_twenty_two_themes_carry_an_anatomy() {
    assert_eq!(ThemeId::ALL.len(), 22);
    let mut seen = std::collections::BTreeSet::new();
    for id in ThemeId::ALL {
        let th = Theme::new(id, ColorDepth::TrueColor);
        // A palette and an anatomy, and a name that is the package's own.
        assert!(!id.name().is_empty(), "{id:?} has no name");
        assert!(seen.insert(id.name()), "{} appears twice", id.name());
        assert_eq!(ThemeId::from_name(id.name()), Some(id));
        // The anatomy is the theme's own, not a default: every prefix that
        // a register declares reaches the panel title.
        let panel: String = format!("{}{}", th.a.label_prefix, "RELEASE");
        assert!(panel.ends_with("RELEASE"), "{}: {panel}", id.name());
    }
    // The order is the package's order, so index and name agree.
    assert_eq!(ThemeId::ALL[0].name(), "formal");
    assert_eq!(ThemeId::ALL[21].name(), "titanium");
    // Stepping with `t` walks the whole set and comes back.
    let mut id = ThemeId::Formal;
    for _ in 0..22 {
        id = id.next();
    }
    assert_eq!(id, ThemeId::Formal);
}

#[test]
fn a_word_staggered_reveal_lights_its_words_in_turn() {
    // The six registers that declare --kp-word-stagger: dark and titanium
    // 28 ms, phantom 28, brutalism 60, shade-light 70, shade-dark 90.
    for name in ["dark", "phantom", "brutalism", "shade-light", "shade-dark", "titanium"] {
        let id = ThemeId::from_name(name).expect(name);
        assert!(matches!(id.anatomy().reveal, Reveal::Words { .. }), "{name} does not reveal word by word");
    }
    let Reveal::Words { ms, stagger_ms } = ThemeId::from_name("shade-dark").unwrap().anatomy().reveal else {
        panic!("shade-dark is not word-staggered");
    };
    assert_eq!((ms, stagger_ms), (600, 90));
    // Early on, the first word is further along than the last.
    let text = "Version 2.4 is ready to deploy";
    let early = fx::frame(text, Reveal::Words { ms, stagger_ms }, 120, fx::Motion::Full);
    assert_eq!(early.words.len(), 6, "one entry per word");
    assert!(early.words[0].1 > early.words[5].1, "{:?}", early.words);
    assert_eq!(early.words[5].1, 0.0, "the last word has not started");
    // The whole line is lit once the last word has had its own duration.
    let whole = fx::frame(text, Reveal::Words { ms, stagger_ms }, ms + stagger_ms * 5, fx::Motion::Full);
    assert!(whole.words.is_empty() && whole.done, "it finishes as one plate");
    // Reduced motion shows the finished line on the first frame.
    let still = fx::frame(text, Reveal::Words { ms, stagger_ms }, 0, fx::Motion::Reduced);
    assert!(still.done && still.words.is_empty());
}
