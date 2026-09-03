use std::sync::Arc;

use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
};

use crate::tui::components::themes::{GBadwolf, Theme};

/// Status widget displaying status messages with error/success coloring.
#[derive(Clone)]
pub struct StatusWidget {
    message: String,
    is_error: bool,
    theme: Arc<dyn Theme>,
}

impl StatusWidget {
    /// Create a new status widget with default theme.
    pub fn new() -> Self {
        Self::with_theme(Arc::new(GBadwolf))
    }

    /// Create a new status widget with an injected theme.
    pub fn with_theme(theme: Arc<dyn Theme>) -> Self {
        Self {
            message: String::new(),
            is_error: false,
            theme,
        }
    }

    /// Set a normal status message (green).
    pub fn report(&mut self, message: impl Into<String>) {
        self.message = message.into();
        self.is_error = false;
    }

    /// Set an error message (red).
    pub fn report_failure(&mut self, error: impl Into<String>) {
        self.message = error.into();
        self.is_error = true;
    }

    /// Render the status widget.
    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let style = if self.is_error {
            self.theme.status_error()
        } else {
            self.theme.content()
        };

        let paragraph = Paragraph::new(self.message.as_str()).style(style).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Status")
                .title_style(self.theme.title())
                .border_style(self.theme.border())
                .style(self.theme.content()),
        );

        frame.render_widget(paragraph, area);
    }
}

impl Default for StatusWidget {
    fn default() -> Self {
        Self::new()
    }
}
