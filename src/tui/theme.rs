use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Énumération publique `ThemeKind`
pub enum ThemeKind {
    /// Variante d'énumération `Dark` du type énuméré.
    Dark,
    /// Variante d'énumération `Light` du type énuméré.
    Light,
    /// Variante d'énumération `HighContrast` du type énuméré.
    HighContrast,
}

/// Structure publique `Theme`
pub struct Theme {
    /// Champ public `bg` de la structure correspondante.
    pub bg: Color,
    /// Champ public `fg` de la structure correspondante.
    pub fg: Color,
    /// Champ public `accent` de la structure correspondante.
    pub accent: Color,
    /// Champ public `warning` de la structure correspondante.
    pub warning: Color,
    /// Champ public `error` de la structure correspondante.
    pub error: Color,
    /// Champ public `success` de la structure correspondante.
    pub success: Color,
    /// Champ public `muted` de la structure correspondante.
    pub muted: Color,
    /// Champ public `border` de la structure correspondante.
    pub border: Color,
    /// Champ public `highlight` de la structure correspondante.
    pub highlight: Style,
    /// Champ public `title` de la structure correspondante.
    pub title: Style,
}

impl Theme {
    /// Fonction publique `from_kind`
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
