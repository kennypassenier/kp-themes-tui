//! The generated RGB equals what the web build resolves from the same HSL.
//!
//! The reference is computed here, independently of build.rs: tokens.json
//! and css/themes.css are read again at test time and converted with a
//! second port of `js/contrast.js` `hslToRgb`, rounded to 8 bits as a
//! browser serialises a computed `rgb()`.

use std::{collections::BTreeMap, fs, path::PathBuf};

use kp_tui::{color::Rgb, theme::ThemeId};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// js/contrast.js: `c = (1 - |2L - 1|) * S; x = c * (1 - |(h/60) % 2 - 1|); m = L - c/2`.
fn web_rgb(hsl: &str) -> Rgb {
    let n: Vec<f64> = hsl
        .trim_start_matches("hsl(")
        .trim_end_matches(')')
        .split(',')
        .map(|p| p.trim().trim_end_matches('%').parse().unwrap())
        .collect();
    let (h, s, l) = (n[0], n[1] / 100.0, n[2] / 100.0);
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let sextant = [[c, x, 0.0], [x, c, 0.0], [0.0, c, x], [0.0, x, c], [x, 0.0, c], [c, 0.0, x]];
    let [r, g, b] = sextant[(h / 60.0).floor() as usize % 6];
    let q = |u: f64| ((u + m) * 255.0).round() as u8;
    Rgb(q(r), q(g), q(b))
}

fn tokens(theme: &str) -> BTreeMap<String, String> {
    let text = fs::read_to_string(root().join(format!("themes/{theme}/tokens.json"))).unwrap();
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    json["entries"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| Some((e["token"].as_str()?.to_string(), e["value"].as_str()?.to_string())))
        .collect()
}

#[test]
fn named_tokens_match_the_web_in_every_theme() {
    for id in ThemeId::ALL {
        let t = tokens(id.name());
        let p = id.palette();
        let pairs = [
            ("background", p.background),
            ("foreground", p.foreground),
            ("card", p.card),
            ("primary", p.primary),
            ("primary-foreground", p.primary_foreground),
            ("destructive", p.destructive),
            ("border", p.border),
            ("border-strong", p.border_strong),
            ("ring", p.ring),
            ("muted-foreground", p.muted_foreground),
        ];
        for (name, generated) in pairs {
            assert_eq!(generated, web_rgb(&t[name]), "{}: --{name} {}", id.name(), t[name]);
        }
    }
}

#[test]
fn every_authored_hsl_token_matches() {
    let mut checked = 0;
    for id in ThemeId::ALL {
        let t = tokens(id.name());
        let hsl_count = t.values().filter(|v| v.starts_with("hsl(")).count();
        assert_eq!(id.tokens().len(), hsl_count, "{}: every hsl token is generated", id.name());
        for (name, generated) in id.tokens() {
            assert_eq!(*generated, web_rgb(&t[*name]), "{}: --{name}", id.name());
            checked += 1;
        }
    }
    assert!(checked > 150, "checked {checked}");
}

#[test]
fn derived_pressed_state_matches_css_themes_css() {
    let css = fs::read_to_string(root().join("css/themes.css")).unwrap();
    for id in ThemeId::ALL {
        let head = format!("\n[data-theme='{}'] {{", id.name());
        let block = &css[css.find(&head).unwrap()..];
        let line = block.lines().find(|l| l.trim_start().starts_with("--primary-active:")).unwrap();
        let value = line.split_once(':').unwrap().1.trim().trim_end_matches(';');
        assert_eq!(id.palette().primary_active, web_rgb(value), "{}: --primary-active {value}", id.name());
    }
}

/// Spot values written out by hand, so a bug shared by both ports of the
/// formula would still be caught. Worked by hand from the HSL.
#[test]
fn hand_checked_values() {
    // cyberpunk --accent hsl(184, 100%, 50%): c = 1, x = 1 - |184/60 % 2 - 1| = 0.9333, m = 0
    assert_eq!(ThemeId::Cyberpunk.palette().accent, Rgb(0, 238, 255));
    // terminal --primary hsl(120, 90%, 50%): c = 0.9, x = 0, m = 0.05
    assert_eq!(ThemeId::Terminal.palette().primary, Rgb(13, 242, 13));
    // formal --background hsl(40, 25%, 97%): c = 0.015, x = 0.01, m = 0.9625
    assert_eq!(ThemeId::Formal.palette().background, Rgb(249, 248, 245));
}
