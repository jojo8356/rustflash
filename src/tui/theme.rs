use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeKind {
    Dark,
    Light,
    HighContrast,
}

pub struct Theme {
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub warning: Color,
    pub error: Color,
    pub success: Color,
    pub muted: Color,
    pub border: Color,
    pub highlight: Style,
    pub title: Style,
}

impl Theme {
    pub fn from_kind(kind: ThemeKind) -> Self {
        match kind {
            ThemeKind::Dark => Self::dark(),
            ThemeKind::Light => Self::light(),
            ThemeKind::HighContrast => Self::high_contrast(),
        }
    }

    fn dark() -> Self {
        Self {
            bg: Color::Rgb(30, 30, 46),
            fg: Color::Rgb(205, 214, 244),
            accent: Color::Rgb(137, 180, 250),
            warning: Color::Rgb(249, 226, 175),
            error: Color::Rgb(243, 139, 168),
            success: Color::Rgb(166, 227, 161),
            muted: Color::Rgb(108, 112, 134),
            border: Color::Rgb(69, 71, 90),
            highlight: Style::default()
                .fg(Color::Rgb(30, 30, 46))
                .bg(Color::Rgb(137, 180, 250))
                .add_modifier(Modifier::BOLD),
            title: Style::default()
                .fg(Color::Rgb(137, 180, 250))
                .add_modifier(Modifier::BOLD),
        }
    }

    fn light() -> Self {
        Self {
            bg: Color::Rgb(239, 241, 245),
            fg: Color::Rgb(76, 79, 105),
            accent: Color::Rgb(30, 102, 245),
            warning: Color::Rgb(223, 142, 29),
            error: Color::Rgb(210, 15, 57),
            success: Color::Rgb(64, 160, 43),
            muted: Color::Rgb(140, 143, 161),
            border: Color::Rgb(172, 176, 190),
            highlight: Style::default()
                .fg(Color::Rgb(239, 241, 245))
                .bg(Color::Rgb(30, 102, 245))
                .add_modifier(Modifier::BOLD),
            title: Style::default()
                .fg(Color::Rgb(30, 102, 245))
                .add_modifier(Modifier::BOLD),
        }
    }

    fn high_contrast() -> Self {
        Self {
            bg: Color::Black,
            fg: Color::White,
            accent: Color::Cyan,
            warning: Color::Yellow,
            error: Color::Red,
            success: Color::Green,
            muted: Color::DarkGray,
            border: Color::White,
            highlight: Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD),
            title: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        }
    }
}
