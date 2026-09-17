//! `kp-tui-demo [--theme NAME] [--colors truecolor|256|16] [--reduced-motion] [--config PATH]`

use std::{
    env, io,
    path::PathBuf,
    time::{Duration, Instant},
};

use crossterm::{cursor::SetCursorStyle, event, execute};
use kp_tui::{
    app::App,
    color::ColorDepth,
    config::{self, Config},
    fx::Motion,
    theme::ThemeId,
};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let arg = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    let getenv = |k: &str| env::var(k).ok();

    let path = arg("--config").map(PathBuf::from).or_else(|| config::default_path(getenv));
    let mut cfg = path.as_deref().map(Config::load).unwrap_or_default();
    if let Some(t) = arg("--theme").as_deref().and_then(ThemeId::from_name) {
        cfg.theme = t;
    }
    if args.iter().any(|a| a == "--reduced-motion") || getenv("KP_REDUCED_MOTION").as_deref() == Some("1") {
        cfg.motion = Motion::Reduced;
    }
    let depth = ColorDepth::detect(arg("--colors").as_deref(), getenv);

    let mut app = App::new(cfg, depth, path);
    let mut terminal = ratatui::init();
    let mut cursor = app.theme.a.cursor;
    execute!(io::stdout(), cursor)?;
    let mut last = Instant::now();
    #[cfg(feature = "tachyonfx")]
    let mut fx_last = Instant::now();
    #[cfg(feature = "tachyonfx")]
    let mut tfx: Option<(tachyonfx::Effect, (ThemeId, usize, Motion))> = None;
    let result = loop {
        #[cfg(feature = "tachyonfx")]
        {
            let key = (app.theme.id, app.tab, app.config.motion);
            let stale =
                tfx.as_ref().map(|(_, k)| *k != key).unwrap_or(app.config.motion == Motion::Full) || app.reveal_ms == 0;
            if stale {
                tfx = (app.config.motion == Motion::Full).then(|| {
                    let text_len = app.headline(terminal.get_frame().area()).1.chars().count();
                    (kp_tui::fx::tachyon::effect(app.theme.a.reveal, app.theme.c.card, text_len), key)
                });
            }
        }
        #[cfg(feature = "tachyonfx")]
        let dt = fx_last.elapsed();
        #[cfg(feature = "tachyonfx")]
        {
            fx_last = Instant::now();
        }
        let drawn = terminal.draw(|f| {
            app.draw(f);
            #[cfg(feature = "tachyonfx")]
            if let Some((effect, _)) = tfx.as_mut() {
                let (area, _) = app.headline(f.area());
                effect.process(dt.into(), f.buffer_mut(), area);
            }
        });
        if let Err(e) = drawn {
            break Err(e);
        }
        if event::poll(Duration::from_millis(33))?
            && let event::Event::Key(k) = event::read()?
        {
            app.key(k);
        }
        let now = Instant::now();
        app.tick(now.duration_since(last).as_millis() as u32);
        last = now;
        if app.theme.a.cursor != cursor {
            cursor = app.theme.a.cursor;
            execute!(io::stdout(), cursor)?;
        }
        if app.quit {
            break Ok(());
        }
    };
    execute!(io::stdout(), SetCursorStyle::DefaultUserShape)?;
    ratatui::restore();
    result
}
