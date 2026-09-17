//! `cargo run --example snapshot [-- truecolor|256|16] [dashboard]`: one
//! finished frame per theme, rendered headless and printed with ANSI
//! colours, so the three themes can be compared in one scroll without
//! driving the app.

use std::{
    fmt::Write as _,
    time::{Duration, Instant},
};

use kp_tui::{
    app::{App, Screen},
    color::ColorDepth,
    config::Config,
    dashboard::Dashboard,
    fx::Motion,
    live::Sampler,
    logs::Feed,
    theme::ThemeId,
};
use ratatui::{
    Terminal,
    backend::TestBackend,
    style::{Color, Modifier},
};

fn sgr(c: Color, fg: bool) -> String {
    let (base, bright) = if fg { (30, 90) } else { (40, 100) };
    match c {
        Color::Rgb(r, g, b) => format!("{};2;{r};{g};{b}", if fg { 38 } else { 48 }),
        Color::Indexed(i) => format!("{};5;{i}", if fg { 38 } else { 48 }),
        Color::Reset => format!("{}", if fg { 39 } else { 49 }),
        Color::Black => format!("{base}"),
        Color::Red => format!("{}", base + 1),
        Color::Green => format!("{}", base + 2),
        Color::Yellow => format!("{}", base + 3),
        Color::Blue => format!("{}", base + 4),
        Color::Magenta => format!("{}", base + 5),
        Color::Cyan => format!("{}", base + 6),
        Color::Gray => format!("{}", base + 7),
        Color::DarkGray => format!("{bright}"),
        Color::LightRed => format!("{}", bright + 1),
        Color::LightGreen => format!("{}", bright + 2),
        Color::LightYellow => format!("{}", bright + 3),
        Color::LightBlue => format!("{}", bright + 4),
        Color::LightMagenta => format!("{}", bright + 5),
        Color::LightCyan => format!("{}", bright + 6),
        Color::White => format!("{}", bright + 7),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let depth = args.iter().find_map(|a| ColorDepth::parse(a)).unwrap_or(ColorDepth::TrueColor);
    // `dashboard`: read this machine for three seconds, then print the
    // dashboard in every theme from the same data.
    let dash = args.iter().any(|a| a == "dashboard");
    let mut live = Dashboard::default();
    if dash {
        let mut feed = Feed::journal().unwrap_or_else(|_| Feed::synthetic("journalctl could not be started"));
        let mut sampler = Sampler::new(feed.child_pid());
        sampler.sample();
        let start = Instant::now();
        for _ in 0..6 {
            std::thread::sleep(Duration::from_millis(500));
            if let Some(s) = sampler.sample() {
                live.push_sample(start.elapsed().as_secs_f64(), s);
            }
            feed.drain(&mut live.logs, 1000);
        }
        live.feed_live = feed.live();
        live.feed_label = feed.label();
    }
    for id in ThemeId::ALL {
        let mut app = App::new(Config { theme: id, motion: Motion::Reduced }, depth, None);
        let size = if dash {
            app.screen = Screen::Dashboard;
            std::mem::swap(&mut app.dash, &mut live);
            (120, 40)
        } else {
            (100, 24)
        };
        let mut term = Terminal::new(TestBackend::new(size.0, size.1)).unwrap();
        term.draw(|f| app.draw(f)).unwrap();
        if dash {
            std::mem::swap(&mut app.dash, &mut live);
        }
        let buf = term.backend().buffer();
        let mut out = format!("\n{} ({})\n", id.name(), depth.label());
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                let cell = &buf[(x, y)];
                let mut codes = vec![sgr(cell.fg, true), sgr(cell.bg, false)];
                if cell.modifier.contains(Modifier::BOLD) {
                    codes.push("1".into());
                }
                if cell.modifier.contains(Modifier::UNDERLINED) {
                    codes.push("4".into());
                }
                if cell.modifier.contains(Modifier::REVERSED) {
                    codes.push("7".into());
                }
                write!(out, "\x1b[0;{}m{}", codes.join(";"), cell.symbol()).unwrap();
            }
            out.push_str("\x1b[0m\n");
        }
        print!("{out}");
    }
}
