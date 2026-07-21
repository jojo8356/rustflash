use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
};

/// Structure publique `ConfirmDialog`
pub struct ConfirmDialog {
    /// Champ public `title` de la structure correspondante.
    pub title: String,
    /// Champ public `message` de la structure correspondante.
    pub message: String,
    /// Champ public `confirm_text` de la structure correspondante.
    pub confirm_text: String,
    /// Champ public `cancel_text` de la structure correspondante.
    pub cancel_text: String,
    /// Champ public `selected_confirm` de la structure correspondante.
    pub selected_confirm: bool,
}

impl ConfirmDialog {
    /// Fonction publique `new`
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            confirm_text: "Confirm".into(),
            cancel_text: "Cancel".into(),
            selected_confirm: false,
        }
    }

    /// Fonction publique `destructive`
    pub fn destructive(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            confirm_text: "Yes, proceed".into(),
            cancel_text: "Cancel".into(),
            selected_confirm: false,
        }
    }

    /// Fonction publique `render`
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let dialog_area = centered_rect(60, 40, area);

        frame.render_widget(Clear, dialog_area);

        let block = Block::default()
            .title(format!(" {} ", self.title))
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Yellow));

        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        let chunks = Layout::vertical([
            Constraint::Min(3),    // message
            Constraint::Length(3), // buttons
        ])
        .split(inner);

        let msg = Paragraph::new(self.message.as_str()).alignment(Alignment::Center);
        frame.render_widget(msg, chunks[0]);

        let confirm_style = if self.selected_confirm {
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            Style::default().fg(Color::Red)
        };

        let cancel_style = if !self.selected_confirm {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            Style::default().fg(Color::White)
        };

        let buttons = Paragraph::new(Line::from(vec![
            ratatui::text::Span::styled(format!(" {} ", self.cancel_text), cancel_style),
            ratatui::text::Span::raw("   "),
            ratatui::text::Span::styled(format!(" {} ", self.confirm_text), confirm_style),
        ]))
        .alignment(Alignment::Center);

        frame.render_widget(buttons, chunks[1]);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)])
        .flex(Flex::Center)
        .split(area);
    Layout::horizontal([Constraint::Percentage(percent_x)])
        .flex(Flex::Center)
        .split(vertical[0])[0]
}
