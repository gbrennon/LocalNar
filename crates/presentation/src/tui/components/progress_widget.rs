use std::sync::Arc;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Gauge, Paragraph},
};

use crate::tui::components::themes::{GBadwolf, Theme};

/// Progress widget displaying install progress with gauge and status message.
#[derive(Clone)]
pub struct ProgressWidget {
    progress: f64,
    message: String,
    theme: Arc<dyn Theme>,
}

impl ProgressWidget {
    /// Create a new progress widget with default theme.
    pub fn new() -> Self {
        Self::with_theme(Arc::new(GBadwolf))
    }

    /// Create a new progress widget with an injected theme.
    pub fn with_theme(theme: Arc<dyn Theme>) -> Self {
        Self {
            progress: 0.0,
            message: String::new(),
            theme,
        }
    }

    /// Reset the progress to initial state.
    pub fn reset(&mut self) {
        self.progress = 0.0;
        self.message = String::new();
    }

    /// Advances the transfer to `progress`, saying what it is doing.
    pub fn advance(&mut self, progress: f64, message: String) {
        self.progress = progress.clamp(0.0, 1.0);
        self.message = message;
    }

    /// Render the progress widget.
    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(Self::LAYOUT_CONSTRAINTS)
            .split(area);

        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Self::GAUGE_TITLE)
                    .title_style(self.theme.title())
                    .border_style(self.theme.border())
                    .style(self.theme.content()),
            )
            .gauge_style(self.theme.highlight())
            .label(format!("{:.1}%", self.progress * 100.0));

        frame.render_widget(gauge, chunks[0]);

        let message = Paragraph::new(self.message.as_str())
            .style(self.theme.content())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Self::STATUS_TITLE)
                    .title_style(self.theme.title())
                    .border_style(self.theme.border())
                    .style(self.theme.content()),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(message, chunks[1]);

        let help = Paragraph::new(Self::HELP_TEXT)
            .style(self.theme.content())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Self::CONTROLS_TITLE)
                    .title_style(self.theme.title())
                    .border_style(self.theme.border())
                    .style(self.theme.content()),
            );
        frame.render_widget(help, chunks[2]);
    }
}

impl ProgressWidget {
    const LAYOUT_CONSTRAINTS: [Constraint; 3] = [
        Constraint::Length(Self::GAUGE_HEIGHT),
        Constraint::Length(Self::STATUS_HEIGHT),
        Constraint::Min(Self::HELP_MIN_HEIGHT),
    ];

    const GAUGE_HEIGHT: u16 = 3;
    const STATUS_HEIGHT: u16 = 3;
    const HELP_MIN_HEIGHT: u16 = 0;

    const GAUGE_TITLE: &'static str = "Progress";
    const STATUS_TITLE: &'static str = "Status";
    const CONTROLS_TITLE: &'static str = "Controls";
    const HELP_TEXT: &'static str =
        "Press Esc to return to the model table (install continues in background)";
}

impl Default for ProgressWidget {
    fn default() -> Self {
        Self::new()
    }
}
