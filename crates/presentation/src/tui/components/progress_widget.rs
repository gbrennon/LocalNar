use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge, Paragraph},
};

/// Progress widget displaying install progress with gauge and status message.
#[derive(Debug, Default)]
pub struct ProgressWidget {
    progress: f64,
    message: String,
}

impl ProgressWidget {
    /// Create a new progress widget.
    pub fn new() -> Self {
        Self::default()
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
                    .title(Self::GAUGE_TITLE),
            )
            .gauge_style(Style::default().fg(Color::Green).bg(Color::Black))
            .ratio(self.progress)
            .label(format!("{:.1}%", self.progress * 100.0));

        frame.render_widget(gauge, chunks[0]);

        let message = Paragraph::new(self.message.as_str())
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Self::STATUS_TITLE),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(message, chunks[1]);

        let help = Paragraph::new(Self::HELP_TEXT)
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Self::CONTROLS_TITLE),
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
