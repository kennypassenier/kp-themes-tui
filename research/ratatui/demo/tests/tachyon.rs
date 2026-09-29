//! Only with `--features tachyonfx`: the library effects run headless over
//! a drawn frame and end on the same cells the hand-written reveal ends on.
#![cfg(feature = "tachyonfx")]

use std::time::Duration;

use kp_tui::{app::App, color::ColorDepth, config::Config, fx::Motion, theme::ThemeId};
use ratatui::{Terminal, backend::TestBackend, buffer::Buffer};

#[test]
fn tachyonfx_reveal_starts_hidden_and_ends_whole() {
    for id in ThemeId::ALL {
        let app = App::new(Config { theme: id, motion: Motion::Reduced }, ColorDepth::TrueColor, None);
        let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
        term.draw(|f| app.draw(f)).unwrap();
        let whole: Buffer = term.backend().buffer().clone();
        let (area, text) = app.headline(whole.area);
        let mut effect = kp_tui::fx::tachyon::effect(id.anatomy().reveal, app.theme.c.card, text.chars().count());

        let mut first = whole.clone();
        effect.process(Duration::from_millis(1).into(), &mut first, area);
        assert_ne!(first, whole, "{}: the first frame differs from the finished one", id.name());

        let mut buf = whole.clone();
        for _ in 0..200 {
            buf = whole.clone();
            effect.process(Duration::from_millis(33).into(), &mut buf, area);
            if effect.done() {
                break;
            }
        }
        assert!(effect.done(), "{}: finishes", id.name());
        let row = |b: &Buffer| (area.x..area.right()).map(|x| b[(x, area.y)].symbol().to_string()).collect::<String>();
        assert_eq!(row(&buf), row(&whole), "{}: ends on the text", id.name());
    }
}
