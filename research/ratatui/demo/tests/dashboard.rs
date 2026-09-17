//! The dashboard, headless: fixed samples and log lines drawn with
//! `TestBackend` in every theme, plus the parsers and the log buffer on
//! fixtures. Nothing here reads `/proc` or spawns `journalctl`.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use kp_tui::{
    app::{App, Screen},
    color::ColorDepth,
    config::Config,
    dashboard::{Metric, PULSE_MS, dash_areas, rate},
    fx::Motion,
    live::{self, Sample},
    logs::{self, LogBuffer, LogLine, Severity},
    theme::ThemeId,
};
use ratatui::{Terminal, backend::TestBackend, buffer::Buffer, layout::Rect, style::Color};

const W: u16 = 120;
const H: u16 = 40;

fn rgb(c: kp_tui::color::Rgb) -> Color {
    Color::Rgb(c.0, c.1, c.2)
}

fn press(app: &mut App, c: char) {
    app.key(KeyEvent::new_with_kind(KeyCode::Char(c), KeyModifiers::NONE, KeyEventKind::Press));
}

fn sample(cpu: f64) -> Sample {
    Sample {
        cpu_total: cpu,
        cores: vec![cpu; 8],
        mem_used_pct: 40.0,
        mem_used_bytes: 8 << 30,
        mem_total_bytes: 20 << 30,
        load: [1.5, 1.2, 1.0],
        rx_bps: 300_000.0,
        tx_bps: 40_000.0,
        disk_read_bps: 0.0,
        disk_write_bps: 1_200_000.0,
        self_cpu_pct: 0.8,
        child_cpu_pct: 0.0,
    }
}

const LINES: [(Severity, &str); 6] = [
    (Severity::Debug, "eno1: link speed 1000 Mbps"),
    (Severity::Info, "Accepted publickey for kenny"),
    (Severity::Notice, "Finished logrotate.service"),
    (Severity::Warning, "temperature reached the warning limit"),
    (Severity::Error, "certificate renewal failed"),
    (Severity::Critical, "inode 1318 has a bad checksum"),
];

fn dashboard(theme: ThemeId, depth: ColorDepth, motion: Motion) -> App {
    let mut app = App::new(Config { theme, motion }, depth, None);
    app.screen = Screen::Dashboard;
    for i in 0..=120 {
        app.dash.push_sample(i as f64 * 0.5, sample(50.0));
    }
    app.dash.feed_live = true;
    for (sev, msg) in LINES {
        app.dash.logs.push(LogLine::new("19:13:27.763", "linux", "smartd[744]", sev, msg));
    }
    app
}

fn render(app: &App) -> Buffer {
    let mut term = Terminal::new(TestBackend::new(W, H)).unwrap();
    term.draw(|f| app.draw(f)).unwrap();
    term.backend().buffer().clone()
}

/// Where `needle` starts inside `area`, row by row.
fn find(buf: &Buffer, area: Rect, needle: &str) -> Option<(u16, u16)> {
    for y in area.y..area.bottom() {
        let cells: Vec<(u16, String)> = (area.x..area.right()).map(|x| (x, buf[(x, y)].symbol().to_string())).collect();
        let row: String = cells.iter().map(|(_, s)| s.as_str()).collect();
        if let Some(byte) = row.find(needle) {
            // Map the byte offset back to a column.
            let mut acc = 0;
            for (x, s) in &cells {
                if acc == byte {
                    return Some((*x, y));
                }
                acc += s.len();
            }
        }
    }
    None
}

fn braille_colours(buf: &Buffer, area: Rect) -> Vec<Color> {
    let mut out = vec![];
    for y in area.y..area.bottom() {
        for x in area.x..area.right() {
            let cell = &buf[(x, y)];
            if cell.symbol().chars().any(|c| ('\u{2801}'..='\u{28FF}').contains(&c)) {
                out.push(cell.fg);
            }
        }
    }
    out
}

#[test]
fn one_frame_per_theme_has_axis_labels_in_the_muted_ink() {
    let a = dash_areas(Rect::new(0, 0, W, H));
    for id in ThemeId::ALL {
        let buf = render(&dashboard(id, ColorDepth::TrueColor, Motion::Reduced));
        let p = id.palette();
        for label in ["100%", "-60s", "now"] {
            let (x, y) = find(&buf, a.cpu, label).unwrap_or_else(|| panic!("{}: CPU axis label {label}", id.name()));
            assert_eq!(buf[(x, y)].fg, rgb(p.muted_foreground), "{}: {label} colour", id.name());
        }
        // The network axis is scaled to the window: 300 KiB/s peaks under 500K.
        assert!(find(&buf, a.net, "500K/s").is_some(), "{}: network axis label", id.name());
        // CPU at 50 % is drawn in the first chart colour.
        assert!(braille_colours(&buf, a.cpu).contains(&rgb(p.chart_1)), "{}: CPU series colour", id.name());
        assert!(braille_colours(&buf, a.net).contains(&rgb(p.chart_3)), "{}: received series colour", id.name());
    }
}

#[test]
fn log_lines_colour_their_parts_and_severity_from_theme_roles() {
    let a = dash_areas(Rect::new(0, 0, W, H));
    for id in ThemeId::ALL {
        let buf = render(&dashboard(id, ColorDepth::TrueColor, Motion::Reduced));
        let p = id.palette();
        let at = |needle: &str| find(&buf, a.logs, needle).unwrap_or_else(|| panic!("{}: {needle}", id.name()));
        let (x, y) = at("ERROR");
        assert_eq!(buf[(x, y)].fg, rgb(p.destructive), "{}: error", id.name());
        let (x, y) = at("WARN");
        assert_eq!(buf[(x, y)].fg, rgb(p.warning_foreground), "{}: warning", id.name());
        let (x, y) = at("INFO");
        assert_eq!(buf[(x, y)].fg, rgb(p.info_foreground), "{}: info", id.name());
        let (x, y) = at("NOTICE");
        assert_eq!(buf[(x, y)].fg, rgb(p.success_foreground), "{}: notice", id.name());
        let (x, y) = at("CRIT");
        assert_eq!(buf[(x, y)].bg, rgb(p.destructive), "{}: critical plate", id.name());
        // timestamp, host and unit each in their own role
        let (x, y) = at("19:13:27.763");
        assert_eq!(buf[(x, y)].fg, rgb(p.muted_foreground), "{}: timestamp", id.name());
        assert_eq!(buf[(x + 13, y)].fg, rgb(p.border_strong), "{}: host", id.name());
        assert_eq!(buf[(x + 19, y)].fg, rgb(p.primary), "{}: unit", id.name());
    }
}

#[test]
fn the_theme_key_changes_a_series_colour_on_the_next_frame() {
    let a = dash_areas(Rect::new(0, 0, W, H));
    let mut app = dashboard(ThemeId::Formal, ColorDepth::TrueColor, Motion::Reduced);
    let before = braille_colours(&render(&app), a.cpu);
    press(&mut app, 't');
    let after = braille_colours(&render(&app), a.cpu);
    let (formal, cyberpunk) = (rgb(ThemeId::Formal.palette().chart_1), rgb(ThemeId::Cyberpunk.palette().chart_1));
    assert_ne!(formal, cyberpunk);
    assert!(before.contains(&formal) && !before.contains(&cyberpunk));
    assert!(after.contains(&cyberpunk) && !after.contains(&formal));
}

#[test]
fn sixteen_colours_keep_the_level_readable() {
    let a = dash_areas(Rect::new(0, 0, W, H));
    for id in ThemeId::ALL {
        let buf = render(&dashboard(id, ColorDepth::Ansi16, Motion::Reduced));
        assert!(buf.content.iter().all(|c| !matches!(c.fg, Color::Rgb(..) | Color::Indexed(_))));
        // The tag is text, so the level survives even where two roles share a hue.
        for tag in ["DEBUG", "INFO", "NOTICE", "WARN", "ERROR", "CRIT"] {
            assert!(find(&buf, a.logs, tag).is_some(), "{}: {tag}", id.name());
        }
        let (x, y) = find(&buf, a.logs, "ERROR").unwrap();
        assert!(matches!(buf[(x, y)].fg, Color::Red | Color::LightRed), "{}: error is red", id.name());
    }
}

#[test]
fn pause_filter_and_scroll() {
    let mut b = LogBuffer::new(100);
    let push = |b: &mut LogBuffer, sev, msg: &str| b.push(LogLine::new("00:00:00.000", "h", "u", sev, msg));
    for i in 0..10 {
        push(&mut b, if i % 2 == 0 { Severity::Info } else { Severity::Error }, &format!("line {i}"));
    }
    let msgs = |b: &LogBuffer, h| b.visible(h).iter().map(|l| l.message.clone()).collect::<Vec<_>>();
    assert_eq!(msgs(&b, 2), ["line 8", "line 9"]);
    b.toggle_pause();
    push(&mut b, Severity::Info, "line 10");
    assert_eq!(msgs(&b, 2), ["line 8", "line 9"], "paused view does not move");
    assert_eq!(b.unseen(), 1);
    b.scroll_up(3);
    assert_eq!(msgs(&b, 2), ["line 5", "line 6"]);
    b.follow();
    assert_eq!(msgs(&b, 1), ["line 10"]);
    // f: debug (all) -> info -> notice -> warning -> error
    for _ in 0..4 {
        b.cycle_filter();
    }
    assert_eq!(b.filter, Severity::Error);
    assert_eq!(msgs(&b, 3), ["line 5", "line 7", "line 9"]);
    b.cycle_filter();
    b.cycle_filter();
    assert_eq!(b.filter, Severity::Debug, "wraps back to everything");
}

#[test]
fn keys_on_the_dashboard_reach_the_log_pane() {
    let mut app = dashboard(ThemeId::Terminal, ColorDepth::TrueColor, Motion::Reduced);
    press(&mut app, 'p');
    assert!(app.dash.logs.paused());
    press(&mut app, 'f');
    assert_eq!(app.dash.logs.filter, Severity::Info);
    let buf = render(&app);
    let a = dash_areas(Rect::new(0, 0, W, H));
    assert!(find(&buf, a.logs, "paused, 0 new").is_some());
    assert!(find(&buf, a.logs, "info and up").is_some());
    assert!(find(&buf, a.logs, "DEBUG").is_none(), "debug is filtered out");
    press(&mut app, 's');
    assert_eq!(app.screen, Screen::Components);
}

#[test]
fn a_threshold_crossing_pulses_only_with_motion() {
    let a = dash_areas(Rect::new(0, 0, W, H));
    for motion in [Motion::Full, Motion::Reduced] {
        let mut app = dashboard(ThemeId::Formal, ColorDepth::TrueColor, motion);
        app.dash.push_sample(61.0, sample(85.0));
        app.tick(PULSE_MS / 4); // the top of the first beat
        let k = app.dash.pulse(Metric::Cpu, motion);
        let buf = render(&app);
        let p = ThemeId::Formal.palette();
        let (x, y) = find(&buf, a.tiles[0], "85.0 %").expect("CPU value");
        // Over the threshold the value is in the warning ink either way.
        assert_eq!(buf[(x, y)].fg, rgb(p.warning_foreground));
        match motion {
            Motion::Full => {
                assert!(k > 0.5, "pulse {k}");
                assert_ne!(buf[(x, y)].bg, rgb(p.card), "the ground lights");
            }
            Motion::Reduced => {
                assert_eq!(k, 0.0);
                assert_eq!(buf[(x, y)].bg, rgb(p.card), "no pulse under reduced motion");
            }
        }
        app.tick(PULSE_MS);
        assert_eq!(app.dash.pulse(Metric::Cpu, motion), 0.0, "it ends");
    }
}

#[test]
fn panel_titles_reveal_at_start_and_not_under_reduced_motion() {
    let a = dash_areas(Rect::new(0, 0, W, H));
    let row = |b: &Buffer, r: Rect| (r.x..r.right()).map(|x| b[(x, r.y)].symbol().to_string()).collect::<String>();
    for id in [ThemeId::Cyberpunk, ThemeId::Terminal] {
        let full = render(&dashboard(id, ColorDepth::TrueColor, Motion::Full));
        let reduced = render(&dashboard(id, ColorDepth::TrueColor, Motion::Reduced));
        assert_ne!(row(&full, a.cpu), row(&reduced, a.cpu), "{}: the title is mid-reveal", id.name());
        let mut app = dashboard(id, ColorDepth::TrueColor, Motion::Full);
        app.tick(10_000);
        assert_eq!(row(&render(&app), a.cpu), row(&reduced, a.cpu), "{}: and ends whole", id.name());
    }
}

#[test]
fn proc_parsers_on_fixtures() {
    let stat = "cpu  100 0 50 800 50 0 0 0 0 0\ncpu0 60 0 20 400 20 0 0 0 0 0\ncpu1 40 0 30 400 30 0 0 0 0 0\nintr 1\n";
    let later = "cpu  200 0 100 900 100 0 0 0 0 0\ncpu0 160 0 20 420 20 0 0 0 0 0\ncpu1 40 0 80 480 30 0 0 0 0 0\n";
    let (a, b) = (live::parse_stat(stat), live::parse_stat(later));
    assert_eq!(a.len(), 3);
    assert_eq!(a[0], live::CpuTimes { total: 1000, idle: 850 });
    let prev = live::Counters { cpu: a, self_ticks: 10, ..Default::default() };
    let cur = live::Counters {
        cpu: b,
        mem_total_kib: 1000,
        mem_available_kib: 250,
        self_ticks: 12,
        net_rx_bytes: 2048,
        ..Default::default()
    };
    let s = live::rates(&prev, &cur, 2.0);
    // total: 300 jiffies passed, 150 of them idle or iowait
    assert_eq!(s.cpu_total, 50.0);
    assert_eq!(s.cores.len(), 2);
    assert!((s.cores[0] - 100.0 * 100.0 / 120.0).abs() < 1e-9);
    assert_eq!(s.mem_used_pct, 75.0);
    assert_eq!(s.rx_bps, 1024.0);
    assert_eq!(s.self_cpu_pct, 1.0, "2 ticks in 2 s at 100 Hz");

    assert_eq!(
        live::parse_meminfo("MemTotal:  64960804 kB\nMemFree: 1 kB\nMemAvailable:   46957828 kB\n"),
        (64960804, 46957828)
    );
    assert_eq!(live::parse_loadavg("2.10 2.59 3.69 2/2584 1287722\n"), [2.10, 2.59, 3.69]);
    let net = "Inter-|   Receive\n face |bytes\n    lo: 999 1 0 0 0 0 0 0 999 1 0 0 0 0 0 0\n  eno1: 4000 3 0 0 0 0 0 5 300 6 0 0 0 0 0 0\n";
    assert_eq!(live::parse_net_dev(net), (4000, 300), "loopback is left out");
    let disks = " 259 0 nvme0n1 300 0 100 106 50 0 20 0 0 27 106\n 259 1 nvme0n1p1 45 0 100 20 5 0 20 0 0 15 20\n";
    assert_eq!(live::parse_diskstats(disks, |n| n == "nvme0n1"), (100 * 512, 20 * 512));
    assert_eq!(live::parse_proc_ticks("42 (kp tui) demo) S 1 2 3 4 5 6 7 8 9 10 250 30 0 0"), 280);
}

#[test]
fn journal_json_lines_parse_to_coloured_parts() {
    let l = logs::parse_journal_json(
        r#"{"__REALTIME_TIMESTAMP":"1789665207763379","PRIORITY":"4","SYSLOG_IDENTIFIER":"smartd","_PID":"744","_HOSTNAME":"linux","MESSAGE":"hot\u001b[31m"}"#,
        7200,
    )
    .unwrap();
    assert_eq!((l.host.as_str(), l.unit.as_str(), l.severity), ("linux", "smartd[744]", Severity::Warning));
    assert_eq!(l.message, "hot [31m", "control characters never reach the terminal");
    assert_eq!(l.time, logs::clock(1789665207763379, 7200));
    assert!(l.time.ends_with(".763"));
    // A byte-array MESSAGE, no priority (defaults to info), priority 0 folds into critical.
    let b = logs::parse_journal_json(r#"{"__REALTIME_TIMESTAMP":"1","MESSAGE":[104,105],"_COMM":"x"}"#, 0).unwrap();
    assert_eq!((b.message.as_str(), b.severity, b.unit.as_str()), ("hi", Severity::Info, "x"));
    assert_eq!(Severity::from_priority(0), Severity::Critical);
    assert_eq!(logs::clock(0, 3600 + 60), "01:01:00.000");
    assert_eq!((rate(812.0), rate(1300.0), rate(35.0 * 1024.0 * 1024.0)), ("812B".into(), "1.3K".into(), "35M".into()));
}
