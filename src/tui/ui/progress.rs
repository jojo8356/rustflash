use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Gauge},
    Frame,
};

pub struct ProgressBar {
    pub label: String,
    pub ratio: f64,
    pub color: Color,
}

impl ProgressBar {
    pub fn new(label: impl Into<String>, ratio: f64) -> Self {
        Self {
            label: label.into(),
            ratio: ratio.clamp(0.0, 1.0),
            color: Color::Cyan,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL))
            .gauge_style(Style::default().fg(self.color))
            .ratio(self.ratio)
            .label(&*self.label);
        frame.render_widget(gauge, area);
    }
}
