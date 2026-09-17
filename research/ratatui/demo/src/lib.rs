//! kp-themes in a terminal: generated palettes, hand-written anatomy,
//! widgets that take `&Theme`, and a reveal that honours reduced motion.

pub mod anatomy;
pub mod app;
pub mod color;
pub mod config;
pub mod fx;
pub mod theme;
pub mod widgets;

/// The build script's output: palettes from `themes/*/tokens.json` and
/// the derived states from `css/themes.css`.
pub mod generated {
    pub use crate::color::{Rgb, Role};
    include!(concat!(env!("OUT_DIR"), "/palettes.rs"));
}
