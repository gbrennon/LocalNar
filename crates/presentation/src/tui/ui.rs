use crossterm::event::Event;
use ratatui::Terminal;

use crate::tui::TuiApp;
use crate::tui::events::EventHandler;

/// Application runner managing the main TUI event loop.
pub struct AppRunner;

impl AppRunner {
    /// Run the application main loop.
    pub async fn run<Backend>(
        terminal: &mut Terminal<Backend>,
        app: &mut TuiApp,
        events: &EventHandler,
    ) -> Result<(), std::io::Error>
    where
        Backend: ratatui::backend::Backend,
    {
        loop {
            terminal.draw(|frame| app.draw(frame))?;

            app.handle_events().await;

            if app.should_quit() {
                break;
            }

            if let Ok(Some(event)) = events.try_next() {
                match event {
                    Event::Key(key) => {
                        app.handle_key_event(key).await;
                    }
                    Event::Resize(_, _) => {
                        // Terminal resized - next draw will use new size
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}