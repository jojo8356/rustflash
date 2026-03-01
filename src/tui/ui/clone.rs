use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
};

use crate::tui::app::{App, CloneState};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(5),
        Constraint::Min(8),
        Constraint::Length(3),
    ])
    .split(area);

    render_title(frame, chunks[0]);
    render_steps(frame, chunks[1], app);
    render_content(frame, chunks[2], app);
    render_progress(frame, chunks[3], app);
}

fn render_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new("Clone Disk")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, area);
}

fn render_steps(frame: &mut Frame, area: Rect, app: &App) {
    let steps = [
        ("1. Source", CloneState::SelectSource),
        ("2. Dest", CloneState::SelectDest),
        ("3. Confirm", CloneState::Confirm),
        ("4. Copying", CloneState::Copying),
        ("5. Verify", CloneState::Verifying),
        ("6. Done", CloneState::Done),
    ];

    let spans: Vec<Span> = steps
        .iter()
        .map(|(label, state)| {
            let style = if *state == app.clone_state {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if is_step_completed(*state, app.clone_state) {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            Span::styled(format!("  {label}  "), style)
        })
        .collect();

    let widget = Paragraph::new(Line::from(spans)).block(Block::default().borders(Borders::ALL));
    frame.render_widget(widget, area);
}

fn is_step_completed(step: CloneState, current: CloneState) -> bool {
    let order = |s: CloneState| -> u8 {
        match s {
            CloneState::SelectSource => 0,
            CloneState::SelectDest => 1,
            CloneState::Confirm => 2,
            CloneState::Copying => 3,
            CloneState::Verifying => 4,
            CloneState::Done | CloneState::Failed => 5,
        }
    };
    order(step) < order(current)
}

fn render_content(frame: &mut Frame, area: Rect, app: &App) {
    match app.clone_state {
        CloneState::SelectSource => render_device_select(frame, area, app, "Select Source Device"),
        CloneState::SelectDest => render_dest_select(frame, area, app),
        CloneState::Confirm => render_confirm(frame, area, app),
        CloneState::Copying => render_status(frame, area, "Cloning disk..."),
        CloneState::Verifying => render_status(frame, area, "Verifying clone..."),
        CloneState::Done => render_result(frame, area, "Clone complete!", Color::Green),
        CloneState::Failed => render_result(
            frame,
            area,
            app.clone_error.as_deref().unwrap_or("Clone failed!"),
            Color::Red,
        ),
    }
}

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

fn render_dest_select(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" Select Destination (Enter=select file, navigate dirs) ")
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

fn render_confirm(frame: &mut Frame, area: Rect, app: &App) {
    let source = app.clone_source.as_deref().unwrap_or("?");
    let dest = app
        .clone_dest
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
            Span::styled("  Dest:   ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&dest),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  WARNING: All data on the destination will be destroyed!",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  Press Enter to confirm, Esc to go back."),
    ];

    let widget = Paragraph::new(text).block(
        Block::default()
            .title(" Confirm Clone ")
            .borders(Borders::ALL),
    );
    frame.render_widget(widget, area);
}

fn render_status(frame: &mut Frame, area: Rect, msg: &str) {
    let widget = Paragraph::new(format!("\n  {msg}")).block(
        Block::default()
            .title(" Operation in Progress ")
            .borders(Borders::ALL),
    );
    frame.render_widget(widget, area);
}

fn render_result(frame: &mut Frame, area: Rect, msg: &str, color: Color) {
    let widget = Paragraph::new(format!("\n  {msg}\n\n  Press Esc or Enter to return."))
        .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
        .block(Block::default().title(" Result ").borders(Borders::ALL));
    frame.render_widget(widget, area);
}

fn render_progress(frame: &mut Frame, area: Rect, app: &App) {
    if let Some(ref progress) = app.clone_progress {
        let ratio = if progress.total_bytes > 0 {
            progress.bytes_written as f64 / progress.total_bytes as f64
        } else {
            0.0
        };

        let speed = bytesize::ByteSize(progress.speed_bytes_per_sec as u64);
        let label = format!(
            "{:.1}% — {}/s — ETA: {:.0}s",
            ratio * 100.0,
            speed,
            progress.eta_seconds,
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
