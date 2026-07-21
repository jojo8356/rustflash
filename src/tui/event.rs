use anyhow::Result;
use crossterm::event::{self, Event, KeyEvent};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Énumération publique `AppEvent`
pub enum AppEvent {
    /// Variante d'énumération `Tick` du type énuméré.
    Tick,
    /// Élément public `Key` exposé par l'API.
    Key(KeyEvent),
    /// Élément public `Resize` exposé par l'API.
    Resize(u16, u16),
}

/// Structure publique `EventHandler`
pub struct EventHandler {
    rx: mpsc::Receiver<AppEvent>,
    _tx: mpsc::Sender<AppEvent>,
}

impl EventHandler {
    /// Fonction publique `new`
    pub fn new(tick_rate_ms: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate_ms);
        let (tx, rx) = mpsc::channel();
        let event_tx = tx.clone();

        thread::spawn(move || {
            loop {
                if event::poll(tick_rate).unwrap_or(false) {
                    match event::read() {
                        Ok(Event::Key(key)) if event_tx.send(AppEvent::Key(key)).is_err() => {
                            return;
                        }
                        Ok(Event::Resize(w, h))
                            if event_tx.send(AppEvent::Resize(w, h)).is_err() =>
                        {
                            return;
                        }
                        _ => {}
                    }
                } else if event_tx.send(AppEvent::Tick).is_err() {
                    return;
                }
            }
        });

        Self { rx, _tx: tx }
    }

    /// Fonction publique `next`
    pub fn next(&self) -> Result<AppEvent> {
        Ok(self.rx.recv()?)
    }
}
