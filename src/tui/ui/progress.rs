use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Gauge},
};

/// Structure publique `ProgressBar`
pub struct ProgressBar {
    /// Champ public `label` de la structure correspondante.
    pub label: String,
    /// Champ public `ratio` de la structure correspondante.
    pub ratio: f64,
    /// Champ public `color` de la structure correspondante.
    pub color: Color,
}

impl ProgressBar {
    /// Fonction publique `new`
    pub fn new(label: impl Into<String>, ratio: f64) -> Self {
        Self {
            label: label.into(),
            ratio: ratio.clamp(0.0, 1.0),
            color: Color::Cyan,
        }
    }

    /// Fonction publique `render`
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL))
            .gauge_style(Style::default().fg(self.color))
            .ratio(self.ratio)
            .label(&*self.label);
        frame.render_widget(gauge, area);
    }
}
