//! A resolved theme: the generated palette in the terminal's colour depth,
//! plus the hand-written anatomy.

use ratatui::style::{Color, Style};

use crate::anatomy::{self, Anatomy};
use crate::color::{ColorDepth, Rgb};
use crate::generated::{self, Palette};

/// One of the package's themes, by its place in `themes/order.json`.
///
/// It was an enum of three while the demo carried three; all twenty-two
/// have an anatomy now [scope-127], and twenty-two variants with six
/// matches each is a table written out by hand. The named constants below
/// keep every call site reading the same (`ThemeId::Cyberpunk`), and a new
/// theme is a line in `order.json` and a row in `anatomy.rs`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemeId(pub usize);

impl ThemeId {
    pub const FORMAL: ThemeId = ThemeId(0);
    pub const LIGHT: ThemeId = ThemeId(1);
    pub const DARK: ThemeId = ThemeId(2);
    pub const CYBERPUNK: ThemeId = ThemeId(3);
    pub const SYNTHWAVE: ThemeId = ThemeId(4);
    pub const PASTEL: ThemeId = ThemeId(5);
    pub const TERMINAL: ThemeId = ThemeId(6);
    pub const FOREST: ThemeId = ThemeId(7);
    pub const HIGH_CONTRAST: ThemeId = ThemeId(8);
    pub const SEPIA: ThemeId = ThemeId(9);
    pub const BLUEPRINT: ThemeId = ThemeId(10);
    pub const SOLSTICE: ThemeId = ThemeId(11);
    pub const BRUTALISM: ThemeId = ThemeId(12);
    pub const DECO: ThemeId = ThemeId(13);
    pub const PHANTOM: ThemeId = ThemeId(14);
    pub const SHADE_LIGHT: ThemeId = ThemeId(15);
    pub const SHADE_DARK: ThemeId = ThemeId(16);
    pub const RETRO: ThemeId = ThemeId(17);
    pub const GROTESK: ThemeId = ThemeId(18);
    pub const LAPIS: ThemeId = ThemeId(19);
    pub const NOSTROMO: ThemeId = ThemeId(20);
    pub const TITANIUM: ThemeId = ThemeId(21);

    // The three the demo carried first, spelled the way every call site
    // already spells them.
    #[allow(non_upper_case_globals)]
    pub const Formal: ThemeId = Self::FORMAL;
    #[allow(non_upper_case_globals)]
    pub const Cyberpunk: ThemeId = Self::CYBERPUNK;
    #[allow(non_upper_case_globals)]
    pub const Terminal: ThemeId = Self::TERMINAL;

    /// Every theme, in the package's own order.
    pub const ALL: [ThemeId; 22] = {
        let mut all = [ThemeId(0); 22];
        let mut i = 0;
        while i < 22 {
            all[i] = ThemeId(i);
            i += 1;
        }
        all
    };

    pub fn name(self) -> &'static str {
        generated::NAMES[self.0]
    }

    pub fn from_name(name: &str) -> Option<Self> {
        generated::NAMES.iter().position(|n| *n == name).map(ThemeId)
    }

    pub fn next(self) -> Self {
        ThemeId((self.0 + 1) % generated::NAMES.len())
    }

    pub fn palette(self) -> &'static Palette<Rgb> {
        &generated::PALETTES[self.0]
    }

    pub fn tokens(self) -> &'static [(&'static str, Rgb)] {
        generated::TOKENS[self.0]
    }

    pub fn anatomy(self) -> &'static Anatomy {
        anatomy::ANATOMIES[self.0]
    }

    pub fn dark(self) -> bool {
        generated::IS_DARK[self.0]
    }

    pub fn fx_duration_ms(self) -> u32 {
        generated::FX_DURATION_MS[self.0]
    }
}

/// What every widget takes. Cheap to build (a few dozen matches), so a
/// theme switch simply builds a new one before the next frame.
#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub id: ThemeId,
    pub depth: ColorDepth,
    pub c: Palette<Color>,
    pub a: &'static Anatomy,
}

impl Theme {
    pub fn new(id: ThemeId, depth: ColorDepth) -> Self {
        Theme { id, depth, c: id.palette().map(|role, rgb| depth.resolve(role, rgb)), a: id.anatomy() }
    }

    pub fn base(&self) -> Style {
        Style::new().bg(self.c.background).fg(self.c.foreground)
    }

    pub fn label(&self, text: &str) -> String {
        let body = if self.a.uppercase_labels { text.to_uppercase() } else { text.to_string() };
        format!("{}{}", self.a.label_prefix, body)
    }
}
