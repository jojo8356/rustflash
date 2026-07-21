use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
};

use crate::tui::app::{App, BackupState, RestoreState, Screen};

/// Fonction publique `render`
pub fn render(frame: &mut Frame, app: &App) {
    match app.screen {
        Screen::Backup => render_backup(frame, app),
        Screen::Restore => render_restore(frame, app),
        _ => {}
    }
}

// ── Backup ────────────────────────────────────────────────────────

fn render_backup(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(5),
        Constraint::Min(8),
        Constraint::Length(3),
    ])
    .split(area);

    // Title
    let title = Paragraph::new("Backup Device")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, chunks[0]);

    // Steps
    let steps = [
        ("1. Source", BackupState::SelectSource),
        ("2. Output", BackupState::SelectOutput),
        ("3. Confirm", BackupState::Confirm),
        ("4. Running", BackupState::Running),
        ("5. Done", BackupState::Done),
    ];
    let spans: Vec<Span> = steps
        .iter()
        .map(|(label, state)| {
            let style = if *state == app.backup_state {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else if backup_step_order(*state) < backup_step_order(app.backup_state) {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            Span::styled(format!("  {label}  "), style)
        })
        .collect();
    let steps_widget =
        Paragraph::new(Line::from(spans)).block(Block::default().borders(Borders::ALL));
    frame.render_widget(steps_widget, chunks[1]);

    // Content
    match app.backup_state {
        BackupState::SelectSource => render_device_select(frame, chunks[2], app, "Select Source Device"),
        BackupState::SelectOutput => render_output_select(frame, chunks[2], app),
        BackupState::Confirm => render_backup_confirm(frame, chunks[2], app),
        BackupState::Running => {
            let w = Paragraph::new("\n  Creating backup...").block(
                Block::default()
                    .title(" Backup in Progress ")
                    .borders(Borders::ALL),
            );
            frame.render_widget(w, chunks[2]);
        }
        BackupState::Done => render_result(frame, chunks[2], "Backup complete!", Color::Green),
        BackupState::Failed => render_result(
            frame,
            chunks[2],
            app.backup_error.as_deref().unwrap_or("Backup failed!"),
            Color::Red,
        ),
    }

    // Progress
    render_op_progress(frame, chunks[3], &app.backup_progress, app);
}

fn backup_step_order(s: BackupState) -> u8 {
    match s {
        BackupState::SelectSource => 0,
        BackupState::SelectOutput => 1,
        BackupState::Confirm => 2,
        BackupState::Running => 3,
        BackupState::Done | BackupState::Failed => 4,
    }
}

fn render_backup_confirm(frame: &mut Frame, area: Rect, app: &App) {
    let source = app.backup_source.as_deref().unwrap_or("?");
    let output = app
        .backup_output
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "?".into());

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Source: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(source),
        ]),
        Line::from(vec![
            Span::styled("  Output: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&output),
        ]),
        Line::from(""),
        Line::from("  Press Enter to start backup, Esc to go back."),
    ];

    let widget = Paragraph::new(text).block(
        Block::default()
            .title(" Confirm Backup ")
            .borders(Borders::ALL),
    );
    frame.render_widget(widget, area);
}

fn render_output_select(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" Select Output Directory (Enter=use this dir) ")
        .borders(Borders::ALL);

    if let Some(ref fb) = app.file_browser {
        let mut items: Vec<ListItem> = Vec::new();

        // Show current dir
        items.push(
            ListItem::new(format!("  Current: {}", fb.current_dir.display()))
                .style(Style::default().fg(Color::Yellow)),
        );

        for (i, entry) in fb.entries.iter().enumerate() {
            if !entry.is_dir {
                continue;
            }
            let name = entry
                .path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "..".into());

            let style = if i == fb.selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                Style::default().fg(Color::Blue)
            };

            items.push(ListItem::new(format!("  {name}/")).style(style));
        }

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    } else {
        let widget = Paragraph::new("Loading...").block(block);
        frame.render_widget(widget, area);
    }
}

// ── Restore ───────────────────────────────────────────────────────

fn render_restore(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(5),
        Constraint::Min(8),
        Constraint::Length(3),
    ])
    .split(area);

    // Title
    let title = Paragraph::new("Restore Backup")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, chunks[0]);

    // Steps
    let steps = [
        ("1. Input", RestoreState::SelectInput),
        ("2. Header", RestoreState::ShowHeader),
        ("3. Target", RestoreState::SelectTarget),
        ("4. Confirm", RestoreState::Confirm),
    ];
    let spans: Vec<Span> = steps
        .iter()
        .map(|(label, state)| {
            let style = if *state == app.restore_state {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else if restore_step_order(*state) < restore_step_order(app.restore_state) {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            Span::styled(format!("  {label}  "), style)
        })
        .collect();
    let steps_widget =
        Paragraph::new(Line::from(spans)).block(Block::default().borders(Borders::ALL));
    frame.render_widget(steps_widget, chunks[1]);

    // Content
    match app.restore_state {
        RestoreState::SelectInput => render_rfb_select(frame, chunks[2], app),
        RestoreState::ShowHeader => render_header_info(frame, chunks[2], app),
        RestoreState::SelectTarget => {
            render_device_select(frame, chunks[2], app, "Select Target Device")
        }
        RestoreState::Confirm => render_restore_confirm(frame, chunks[2], app),
        RestoreState::Running => {
            let w = Paragraph::new("\n  Restoring backup...").block(
                Block::default()
                    .title(" Restore in Progress ")
                    .borders(Borders::ALL),
            );
            frame.render_widget(w, chunks[2]);
        }
        RestoreState::Done => render_result(frame, chunks[2], "Restore complete!", Color::Green),
        RestoreState::Failed => render_result(
            frame,
            chunks[2],
            app.restore_error.as_deref().unwrap_or("Restore failed!"),
            Color::Red,
        ),
    }

    // Progress
    render_op_progress(frame, chunks[3], &app.restore_progress, app);
}

fn restore_step_order(s: RestoreState) -> u8 {
    match s {
        RestoreState::SelectInput => 0,
        RestoreState::ShowHeader => 1,
        RestoreState::SelectTarget => 2,
        RestoreState::Confirm => 3,
        RestoreState::Running => 4,
        RestoreState::Done | RestoreState::Failed => 5,
    }
}

fn render_rfb_select(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" Select Backup File (.rfb) ")
        .borders(Borders::ALL);

    if let Some(ref fb) = app.file_browser {
        let items: Vec<ListItem> = fb
            .entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let name = entry
                    .path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "..".into());

                let display = if entry.is_dir {
                    format!("  {name}/")
                } else {
                    let size = bytesize::ByteSize(entry.size);
                    format!("  {name}  ({size})")
                };

                let style = if i == fb.selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD | Modifier::REVERSED)
                } else if entry.is_dir {
                    Style::default().fg(Color::Blue)
                } else {
                    Style::default()
                };

                ListItem::new(display).style(style)
            })
            .collect();

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    } else {
        let widget = Paragraph::new("Loading...").block(block);
        frame.render_widget(widget, area);
    }
}

fn render_header_info(frame: &mut Frame, area: Rect, app: &App) {
    let text = if let Some(ref header) = app.restore_header {
        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  Source device: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(header.source_device.as_deref().unwrap_or("unknown")),
            ]),
            Line::from(vec![
                Span::styled(
                    "  Created:       ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(header.created.to_string()),
            ]),
            Line::from(vec![
                Span::styled(
                    "  Source size:    ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(bytesize::ByteSize(header.source_size).to_string()),
            ]),
            Line::from(vec![
                Span::styled(
                    "  Compression:   ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(&header.compression),
            ]),
            Line::from(vec![
                Span::styled(
                    "  Checksum:      ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(
                    "{}:{}...",
                    header.hash_algorithm,
                    &header.checksum[..16.min(header.checksum.len())]
                )),
            ]),
            Line::from(""),
            Line::from("  Press Enter to select target device, Esc to go back."),
        ]
    } else {
        vec![Line::from("  No header loaded.")]
    };

    let widget = Paragraph::new(text).block(
        Block::default()
            .title(" Backup Header ")
            .borders(Borders::ALL),
    );
    frame.render_widget(widget, area);
}

fn render_restore_confirm(frame: &mut Frame, area: Rect, app: &App) {
    let input = app
        .restore_input
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "?".into());
    let target = app.restore_target.as_deref().unwrap_or("?");

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Backup: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&input),
        ]),
        Line::from(vec![
            Span::styled("  Target: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(target),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  WARNING: All data on the target will be destroyed!",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  Press Enter to confirm, Esc to go back."),
    ];

    let widget = Paragraph::new(text).block(
        Block::default()
            .title(" Confirm Restore ")
            .borders(Borders::ALL),
    );
    frame.render_widget(widget, area);
}

// ── Shared helpers ────────────────────────────────────────────────

fn render_device_select(frame: &mut Frame, area: Rect, app: &App, title: &str) {
    let block = Block::default()
        .title(format!(" {title} (Enter=select) "))
        .borders(Borders::ALL);

    if app.devices.is_empty() {
        let widget = Paragraph::new("  No devices detected.\n  Plug in a device.")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        frame.render_widget(widget, area);
    } else {
        let items: Vec<ListItem> = app
            .devices
            .iter()
            .enumerate()
            .map(|(i, dev)| {
                let size = bytesize::ByteSize(dev.size);
                let model = dev.model.as_deref().unwrap_or("Unknown");
                let display = format!("  {} — {} ({})", dev.path, model, size);
                let style = if i == app.selected_index {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD | Modifier::REVERSED)
                } else {
                    Style::default()
                };
                ListItem::new(display).style(style)
            })
            .collect();

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }
}

fn render_result(frame: &mut Frame, area: Rect, msg: &str, color: Color) {
    let widget = Paragraph::new(format!("\n  {msg}\n\n  Press Esc or Enter to return."))
        .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
        .block(Block::default().title(" Result ").borders(Borders::ALL));
    frame.render_widget(widget, area);
}

fn render_op_progress(
    frame: &mut Frame,
    area: Rect,
    progress: &Option<super::super::app::OperationProgress>,
    app: &App,
) {
    if let Some(p) = progress {
        let ratio = if p.total_bytes > 0 {
            p.bytes_written as f64 / p.total_bytes as f64
        } else {
            0.0
        };

        let speed = bytesize::ByteSize(p.speed_bytes_per_sec as u64);
        let label = format!(
            "{:.1}% — {}/s — ETA: {:.0}s",
            ratio * 100.0,
            speed,
            p.eta_seconds,
        );

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Cyan))
            .ratio(ratio.clamp(0.0, 1.0))
            .label(label);
        frame.render_widget(gauge, area);
    } else {
        let msg = app
            .status_message
            .as_deref()
            .unwrap_or("Esc=back  Enter=select");

        let bar = Paragraph::new(msg)
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::TOP));
        frame.render_widget(bar, area);
    }
}
