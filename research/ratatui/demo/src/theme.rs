//! A resolved theme: the generated palette in the terminal's colour depth,
//! plus the hand-written anatomy.

use ratatui::style::{Color, Style};

use crate::anatomy::{self, Anatomy};
use crate::color::{ColorDepth, Rgb};
use crate::generated::{self, Palette};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeId {
    Formal,
    Cyberpunk,
    Terminal,
}

impl ThemeId {
    pub const ALL: [ThemeId; 3] = [ThemeId::Formal, ThemeId::Cyberpunk, ThemeId::Terminal];

    pub fn name(self) -> &'static str {
        match self {
            ThemeId::Formal => "formal",
            ThemeId::Cyberpunk => "cyberpunk",
            ThemeId::Terminal => "terminal",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.name() == name)
    }

    pub fn next(self) -> Self {
        let i = Self::ALL.iter().position(|t| *t == self).unwrap();
        Self::ALL[(i + 1) % Self::ALL.len()]
    }

    pub fn palette(self) -> &'static Palette<Rgb> {
        match self {
            ThemeId::Formal => &generated::FORMAL,
            ThemeId::Cyberpunk => &generated::CYBERPUNK,
            ThemeId::Terminal => &generated::TERMINAL,
        }
    }

    pub fn tokens(self) -> &'static [(&'static str, Rgb)] {
        match self {
            ThemeId::Formal => generated::FORMAL_TOKENS,
            ThemeId::Cyberpunk => generated::CYBERPUNK_TOKENS,
            ThemeId::Terminal => generated::TERMINAL_TOKENS,
        }
    }

    pub fn anatomy(self) -> &'static Anatomy {
        match self {
            ThemeId::Formal => &anatomy::FORMAL,
            ThemeId::Cyberpunk => &anatomy::CYBERPUNK,
            ThemeId::Terminal => &anatomy::TERMINAL,
        }
    }

    pub fn dark(self) -> bool {
        match self {
            ThemeId::Formal => generated::FORMAL_DARK,
            ThemeId::Cyberpunk => generated::CYBERPUNK_DARK,
            ThemeId::Terminal => generated::TERMINAL_DARK,
        }
    }

    pub fn fx_duration_ms(self) -> u32 {
        match self {
            ThemeId::Formal => generated::FORMAL_FX_DURATION_MS,
            ThemeId::Cyberpunk => generated::CYBERPUNK_FX_DURATION_MS,
            ThemeId::Terminal => generated::TERMINAL_FX_DURATION_MS,
        }
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
