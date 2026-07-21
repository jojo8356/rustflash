use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Row, Table},
};

use crate::tui::app::{App, PartitionActionType, PartitionState};

/// Fonction publique `render`
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::vertical([
        Constraint::Length(3),  // title
        Constraint::Length(3),  // step indicator
        Constraint::Min(8),    // main content
        Constraint::Length(3), // status
    ])
    .split(area);

    render_title(frame, chunks[0]);
    render_steps(frame, chunks[1], app);
    render_content(frame, chunks[2], app);
    render_status(frame, chunks[3], app);
}

fn render_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new("Partition Manager")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, area);
}

fn render_steps(frame: &mut Frame, area: Rect, app: &App) {
    let steps = ["Device", "Table", "Action", "Params", "Confirm", "Run"];
    let current = match app.partition_state {
        PartitionState::SelectDevice => 0,
        PartitionState::ShowTable => 1,
        PartitionState::SelectAction => 2,
        PartitionState::InputParams => 3,
        PartitionState::Confirm => 4,
        PartitionState::Running => 5,
        PartitionState::Done | PartitionState::Failed => 5,
    };

    let spans: Vec<Span> = steps
        .iter()
        .enumerate()
        .flat_map(|(i, s)| {
            let style = if i < current {
                Style::default().fg(Color::Green)
            } else if i == current {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let sep = if i < steps.len() - 1 { " → " } else { "" };
            vec![Span::styled(*s, style), Span::raw(sep)]
        })
        .collect();

    let widget = Paragraph::new(Line::from(spans))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(widget, area);
}

fn render_content(frame: &mut Frame, area: Rect, app: &App) {
    match app.partition_state {
        PartitionState::SelectDevice => render_select_device(frame, area, app),
        PartitionState::ShowTable => render_show_table(frame, area, app),
        PartitionState::SelectAction => render_select_action(frame, area, app),
        PartitionState::InputParams => render_input_params(frame, area, app),
        PartitionState::Confirm => render_confirm(frame, area, app),
        PartitionState::Running => render_running(frame, area, app),
        PartitionState::Done => render_done(frame, area),
        PartitionState::Failed => render_failed(frame, area, app),
    }
}

fn render_select_device(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .devices
        .iter()
        .enumerate()
        .map(|(i, dev)| {
            let style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let size = bytesize::ByteSize(dev.size);
            let model = dev.model.as_deref().unwrap_or("Unknown");
            ListItem::new(format!("  {} — {} ({})", dev.path, model, size)).style(style)
        })
        .collect();

    let hint = if items.is_empty() {
        " No devices found. "
    } else {
        " Select a device (↑↓ Enter) "
    };

    let list = List::new(items).block(
        Block::default()
            .title(hint)
            .borders(Borders::ALL),
    );
    frame.render_widget(list, area);
}

fn render_show_table(frame: &mut Frame, area: Rect, app: &App) {
    let type_str = match app.partition_table_type {
        Some(crate::core::partition::TableType::Gpt) => "GPT",
        Some(crate::core::partition::TableType::Mbr) => "MBR",
        None => "None",
    };

    let dev = app.partition_device.as_deref().unwrap_or("?");
    let header_text = format!("  Device: {}  |  Table: {}", dev, type_str);

    let inner = Layout::vertical([
        Constraint::Length(2), // header
        Constraint::Min(4),   // table
        Constraint::Length(2), // hint
    ])
    .split(area);

    let header = Paragraph::new(header_text)
        .style(Style::default().fg(Color::White));
    frame.render_widget(header, inner[0]);

    if app.partition_table.is_empty() {
        let empty = Paragraph::new("  No partitions found.")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL).title(" Partitions "));
        frame.render_widget(empty, inner[1]);
    } else {
        let rows: Vec<Row> = app
            .partition_table
            .iter()
            .map(|p| {
                let label = p.label.as_deref().unwrap_or("");
                Row::new(vec![
                    format!("{}", p.number),
                    format!("{}", p.start_sector),
                    format!("{}", p.end_sector),
                    bytesize::ByteSize(p.size_bytes).to_string(),
                    p.fs_type.as_str().to_string(),
                    label.to_string(),
                ])
            })
            .collect();

        let widths = [
            Constraint::Length(4),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Min(10),
        ];

        let table = Table::new(rows, widths)
            .header(
                Row::new(vec!["#", "Start", "End", "Size", "Type", "Label"])
                    .style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
            )
            .block(Block::default().borders(Borders::ALL).title(" Partitions "));
        frame.render_widget(table, inner[1]);
    }

    let hint = Paragraph::new("  Press Enter for actions, Esc to go back")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(hint, inner[2]);
}

fn render_select_action(frame: &mut Frame, area: Rect, app: &App) {
    let actions = [
        "Add partition",
        "Delete partition",
        "Format partition",
        "Create new table",
        "Secure erase",
    ];

    let items: Vec<ListItem> = actions
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let style = if i == app.partition_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let marker = if i == app.partition_selected {
                "▸ "
            } else {
                "  "
            };
            ListItem::new(format!("{marker}{a}")).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" Select Action (↑↓ Enter) ")
            .borders(Borders::ALL),
    );
    frame.render_widget(list, area);
}

fn render_input_params(frame: &mut Frame, area: Rect, app: &App) {
    let (title, help) = match app.partition_action {
        Some(PartitionActionType::Add) => (
            "Add Partition",
            "Format: <type> <size> [label]\nExample: ext4 4G mydata\nTypes: ext4, fat32, ntfs, exfat, swap\nSizes: 256M, 4G, 1T, remaining",
        ),
        Some(PartitionActionType::Delete) => (
            "Delete Partition",
            "Enter partition number to delete.\nExample: 1",
        ),
        Some(PartitionActionType::Format) => (
            "Format Partition",
            "Format: <number> <type>\nExample: 1 ext4\nTypes: ext4, fat32, ntfs, exfat, swap",
        ),
        _ => ("Parameters", ""),
    };

    let inner = Layout::vertical([
        Constraint::Length(5), // help text
        Constraint::Length(3), // input field
        Constraint::Min(1),   // spacer
    ])
    .split(area);

    let help_widget = Paragraph::new(help)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().title(format!(" {title} ")).borders(Borders::ALL));
    frame.render_widget(help_widget, inner[0]);

    let input_display = format!("▸ {}_", app.partition_input);
    let input_widget = Paragraph::new(input_display)
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .title(" Input (Enter to confirm, Esc to cancel) ")
                .borders(Borders::ALL),
        );
    frame.render_widget(input_widget, inner[1]);
}

fn render_confirm(frame: &mut Frame, area: Rect, app: &App) {
    let dev = app.partition_device.as_deref().unwrap_or("?");
    let action_desc = match app.partition_action {
        Some(PartitionActionType::Add) => format!("Add partition: {}", app.partition_input),
        Some(PartitionActionType::Delete) => {
            format!("Delete partition: {}", app.partition_input)
        }
        Some(PartitionActionType::Format) => {
            format!("Format partition: {}", app.partition_input)
        }
        Some(PartitionActionType::CreateTable) => "Create new partition table (GPT)".into(),
        Some(PartitionActionType::Erase) => {
            "SECURE ERASE — All data will be destroyed!".into()
        }
        None => "Unknown action".into(),
    };

    let text = format!(
        "  Device: {dev}\n  Action: {action_desc}\n\n  Press Enter to execute, Esc to cancel."
    );

    let style = if matches!(app.partition_action, Some(PartitionActionType::Erase)) {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Yellow)
    };

    let widget = Paragraph::new(text).style(style).block(
        Block::default()
            .title(" Confirm ")
            .borders(Borders::ALL),
    );
    frame.render_widget(widget, area);
}

fn render_running(frame: &mut Frame, area: Rect, app: &App) {
    if let Some(ref p) = app.partition_progress {
        let ratio = if p.total_bytes > 0 {
            p.bytes_written as f64 / p.total_bytes as f64
        } else {
            0.0
        };
        let pct = (ratio * 100.0) as u16;
        let speed = bytesize::ByteSize(p.speed_bytes_per_sec as u64);
        let label = format!(
            "{}/{} ({}/s) — {}%",
            bytesize::ByteSize(p.bytes_written),
            bytesize::ByteSize(p.total_bytes),
            speed,
            pct
        );

        let inner = Layout::vertical([
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(area);

        let msg = app
            .status_message
            .as_deref()
            .unwrap_or("Working...");
        let info = Paragraph::new(format!("  {msg}"))
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title(" Running "));
        frame.render_widget(info, inner[0]);

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Red))
            .ratio(ratio.min(1.0))
            .label(label);
        frame.render_widget(gauge, inner[1]);
    } else {
        let widget = Paragraph::new("  Working...")
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title(" Running "));
        frame.render_widget(widget, area);
    }
}

fn render_done(frame: &mut Frame, area: Rect) {
    let widget = Paragraph::new("  Operation completed successfully!\n\n  Press Enter or Esc to continue.")
        .style(Style::default().fg(Color::Green))
        .block(
            Block::default()
                .title(" Done ")
                .borders(Borders::ALL),
        );
    frame.render_widget(widget, area);
}

fn render_failed(frame: &mut Frame, area: Rect, app: &App) {
    let err = app.partition_error.as_deref().unwrap_or("Unknown error");
    let text = format!("  Error: {err}\n\n  Press Enter or Esc to go back.");
    let widget = Paragraph::new(text)
        .style(Style::default().fg(Color::Red))
        .block(
            Block::default()
                .title(" Failed ")
                .borders(Borders::ALL),
        );
    frame.render_widget(widget, area);
}

fn render_status(frame: &mut Frame, area: Rect, app: &App) {
    let msg = app.status_message.as_deref().unwrap_or("");
    let widget = Paragraph::new(format!("  {msg}"))
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(widget, area);
}
