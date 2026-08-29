use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::tui::components::help_section::HelpSection;

/// Help widget displaying key bindings and usage information.
#[derive(Debug, Default)]
pub struct HelpWidget {
    title: &'static str,
}

impl HelpWidget {
    pub const TITLE: &'static str = "LocalNar TUI";

    /// Create a new help widget.
    pub fn new() -> Self {
        Self { title: Self::TITLE }
    }

    /// Render the help widget.
    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let mut help_lines = vec![Line::from(vec![Span::styled(
            self.title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )])];
        help_lines.push(Line::from(""));

        for section in HelpSection::ALL {
            help_lines.extend(section.to_lines());
        }

        let paragraph = Paragraph::new(help_lines)
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::ALL).title("Help"))
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }
}
