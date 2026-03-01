use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::tui::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::vertical([
        Constraint::Length(3),  // title
        Constraint::Length(10), // menu
        Constraint::Min(5),    // devices
        Constraint::Length(3), // status bar
    ])
    .split(area);

    render_title(frame, chunks[0]);
    render_menu(frame, chunks[1]);
    render_devices(frame, chunks[2], app);
    render_status_bar(frame, chunks[3], app);
}

fn render_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new("RustFlash")
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, area);
}

fn render_menu(frame: &mut Frame, area: Rect) {
    let menu_items = vec![
        Line::from(vec![
            Span::styled("  [F] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("Flash image"),
            Span::raw("          "),
            Span::styled("[C] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("Clone disk"),
        ]),
        Line::from(vec![
            Span::styled("  [B] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("Backup"),
            Span::raw("               "),
            Span::styled("[R] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("Restore"),
        ]),
        Line::from(vec![
            Span::styled("  [P] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("Partitions"),
            Span::raw("           "),
            Span::styled("[S] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("Settings"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [Q] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw("Quit"),
        ]),
    ];

    let menu = Paragraph::new(menu_items)
        .block(Block::default().title(" Menu ").borders(Borders::ALL));
    frame.render_widget(menu, area);
}

fn render_devices(frame: &mut Frame, area: Rect, app: &App) {
    let mut lines = Vec::new();

    if app.devices.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No removable devices detected. Plug in a USB drive or SD card.",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for dev in &app.devices {
            let size_str = bytesize::ByteSize(dev.size).to_string();
            let model = dev.model.as_deref().unwrap_or("Unknown");
            lines.push(Line::from(vec![
                Span::styled("  ● ", Style::default().fg(Color::Green)),
                Span::styled(&dev.path, Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!(" - {model} ({size_str})")),
            ]));
        }
    }

    let devices = Paragraph::new(lines).block(
        Block::default()
            .title(" Detected Devices ")
            .borders(Borders::ALL),
    );
    frame.render_widget(devices, area);
}

fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let msg = app
        .status_message
        .as_deref()
        .unwrap_or("Tab=navigate  Enter=select  Esc=back  Q=quit");

    let bar = Paragraph::new(msg)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::TOP));
    frame.render_widget(bar, area);
}
