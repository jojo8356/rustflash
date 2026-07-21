/// Module public `backup`
pub mod backup;
/// Module public `clone`
pub mod clone;
/// Module public `dialog`
pub mod dialog;
/// Module public `file_browser`
pub mod file_browser;
/// Module public `flash`
pub mod flash;
/// Module public `home`
pub mod home;
/// Module public `partition`
pub mod partition;
/// Module public `progress`
pub mod progress;
/// Module public `settings`
pub mod settings;

use ratatui::Frame;

use super::app::{App, Screen};

/// Fonction publique `render`
pub fn render(frame: &mut Frame, app: &App) {
    match app.screen {
        Screen::Home => home::render(frame, app),
        Screen::Flash => flash::render(frame, app),
        Screen::Clone => clone::render(frame, app),
        Screen::Backup => backup::render(frame, app),
        Screen::Restore => backup::render(frame, app), // shared view
        Screen::Partition => partition::render(frame, app),
        Screen::Settings => settings::render(frame, app),
    }
}
