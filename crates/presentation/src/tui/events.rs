use crossterm::event::{self, Event, KeyEvent};
use std::time::Duration;

/// Event handler for crossterm events with configurable tick rate.
pub struct EventHandler {
    tick_rate: Duration,
}

impl EventHandler {
    /// Create a new event handler with the given tick rate.
    pub fn new(tick_rate: Duration) -> Self {
        Self { tick_rate }
    }

    /// Wait for the next event up to the tick rate.
    pub fn next(&self) -> Result<Event, std::io::Error> {
        match event::poll(self.tick_rate)? {
            true => event::read(),
            false => Err(std::io::Error::new(std::io::ErrorKind::WouldBlock, "no event")),
        }
    }

    /// Try to get the next event without blocking.
    pub fn try_next(&self) -> Result<Option<Event>, std::io::Error> {
        if event::poll(Duration::from_millis(0))? {
            Ok(Some(event::read()?))
        } else {
            Ok(None)
        }
    }

    /// Check if the key event is a quit key (q/Q or Ctrl+C).
    pub fn is_quit_key(key: &KeyEvent) -> bool {
        matches!(key.code, crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Char('Q'))
            || matches!(
                key.code,
                crossterm::event::KeyCode::Char('c') | crossterm::event::KeyCode::Char('C')
            ) && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
    }
}