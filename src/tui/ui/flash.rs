use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
};

use crate::tui::app::{App, FlashState};

/// Fonction publique `render`
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::vertical([
        Constraint::Length(3), // title
        Constraint::Length(5), // step indicator
        Constraint::Min(8),    // main content
        Constraint::Length(3), // progress / status
    ])
    .split(area);

    render_flash_title(frame, chunks[0]);
    render_steps(frame, chunks[1], app);
    render_content(frame, chunks[2], app);
    render_progress(frame, chunks[3], app);
}

fn render_flash_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new("Flash Image → Device")
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
        ("1. Image", FlashState::SelectImage),
        ("2. Target", FlashState::SelectTarget),
        ("3. Confirm", FlashState::Confirm),
        ("4. Writing", FlashState::Writing),
        ("5. Verify", FlashState::Verifying),
        ("6. Done", FlashState::Done),
    ];

    let spans: Vec<Span> = steps
        .iter()
        .map(|(label, state)| {
            let style = if *state == app.flash_state {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if is_step_completed(*state, app.flash_state) {
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

fn is_step_completed(step: FlashState, current: FlashState) -> bool {
    let order = |s: FlashState| -> u8 {
        match s {
            FlashState::SelectImage => 0,
            FlashState::SelectTarget => 1,
            FlashState::Confirm => 2,
            FlashState::Writing => 3,
            FlashState::Verifying => 4,
            FlashState::Done | FlashState::Failed => 5,
        }
    };
    order(step) < order(current)
}

fn render_content(frame: &mut Frame, area: Rect, app: &App) {
    match app.flash_state {
        FlashState::SelectImage => render_image_select(frame, area, app),
        FlashState::SelectTarget => render_target_select(frame, area, app),
        FlashState::Confirm => render_confirm(frame, area, app),
        FlashState::Writing => render_writing_status(frame, area, "Writing image to device..."),
        FlashState::Verifying => render_writing_status(frame, area, "Verifying written data..."),
        FlashState::Done => render_result(frame, area, "Flash complete!", Color::Green),
        FlashState::Failed => render_result(
            frame,
            area,
            app.flash_error.as_deref().unwrap_or("Flash failed!"),
            Color::Red,
        ),
    }
}

fn render_image_select(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" Select Image (Enter=select, Up/Down=navigate) ")
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
        let widget = Paragraph::new("Loading file browser...").block(block);
        frame.render_widget(widget, area);
    }
}

fn render_target_select(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" Select Target Device (Enter=select) ")
        .borders(Borders::ALL);

    if app.devices.is_empty() {
        let widget =
            Paragraph::new("  No removable devices detected.\n  Plug in a USB drive or SD card.")
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

fn render_confirm(frame: &mut Frame, area: Rect, app: &App) {
    let image_str = app
        .selected_image
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "?".into());
    let target_str = app.selected_target.as_deref().unwrap_or("?");

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Image:  ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&image_str),
        ]),
        Line::from(vec![
            Span::styled("  Target: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(target_str),
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
            .title(" Confirm Flash ")
            .borders(Borders::ALL),
    );
    frame.render_widget(widget, area);
}

fn render_writing_status(frame: &mut Frame, area: Rect, msg: &str) {
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
    if let Some(ref progress) = app.progress {
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
