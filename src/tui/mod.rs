pub mod app;
pub mod event;
pub mod theme;
pub mod ui;

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;

use self::app::App;
use self::event::EventHandler;

pub async fn run() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state and event handler
    let mut app = App::new();
    let events = EventHandler::new(250);

    // Main loop
    let result = run_loop(&mut terminal, &mut app, &events).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    events: &EventHandler,
) -> Result<()> {
    loop {
        terminal.draw(|frame| ui::render(frame, app))?;

        match events.next()? {
            event::AppEvent::Tick => app.tick(),
            event::AppEvent::Key(key) => {
                if app.handle_key(key) {
                    return Ok(());
                }
            }
            event::AppEvent::Resize(_, _) => {}
        }
    }
}
