use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

/// Status widget displaying status messages with error/success coloring.
#[derive(Debug, Default)]
pub struct StatusWidget {
    message: String,
    is_error: bool,
}

impl StatusWidget {
    /// Create a new status widget.
    pub fn new() -> Self {
        Self::default()
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
            Style::default().fg(Color::Red).bg(Color::Black)
        } else {
            Style::default().fg(Color::Green).bg(Color::Black)
        };

        let paragraph = Paragraph::new(self.message.as_str())
            .style(style)
            .block(Block::default().borders(Borders::ALL).title("Status"));

        frame.render_widget(paragraph, area);
    }
}
