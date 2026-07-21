use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::tui::app::App;

/// Fonction publique `render`
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(12),
        Constraint::Length(3),
    ])
    .split(area);

    render_title(frame, chunks[0]);
    render_content(frame, chunks[1], app);
    render_status_bar(frame, chunks[2], app);
}

fn render_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new("Settings")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, area);
}

fn render_content(frame: &mut Frame, area: Rect, _app: &App) {
    let info = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Application notes", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  - Device safety-by-default is enabled (system disks hidden unless expert mode)."),
        Line::from("  - Cross-platform device enumeration is available (Linux, Windows, macOS)."),
        Line::from("  - Flash/clone/delete flows are centralized in the core workflow."),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Upcoming options", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  - Persisted preferences"),
        Line::from("  - Device filtering presets"),
        Line::from("  - Runtime log level"),
        Line::from(""),
        Line::from(vec![Span::styled("  For now this screen is informational.", Style::default().fg(Color::DarkGray))]),
    ];

    let list = List::new(
        info.into_iter()
            .map(ListItem::new)
            .collect::<Vec<ListItem>>(),
    )
    .block(Block::default().title(" Settings ").borders(Borders::ALL));

    frame.render_widget(list, area);
}

fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let msg = app
        .status_message
        .as_deref()
        .unwrap_or("Esc = back to home");

    let bar = Paragraph::new(msg)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::TOP));
    frame.render_widget(bar, area);
}
