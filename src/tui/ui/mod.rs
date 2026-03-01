pub mod backup;
pub mod clone;
pub mod dialog;
pub mod file_browser;
pub mod flash;
pub mod home;
pub mod partition;
pub mod progress;

use ratatui::Frame;

use super::app::{App, Screen};

pub fn render(frame: &mut Frame, app: &App) {
    match app.screen {
        Screen::Home => home::render(frame, app),
        Screen::Flash => flash::render(frame, app),
        Screen::Clone => clone::render(frame, app),
        Screen::Backup => backup::render(frame, app),
        Screen::Restore => backup::render(frame, app), // shared view
        Screen::Partition => partition::render(frame, app),
        Screen::Settings => home::render(frame, app), // TODO: settings view
    }
}
