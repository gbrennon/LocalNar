use ratatui::{
    Frame,
    layout::Rect,
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::tui::{components::help_section::HelpSection, theme::Theme};

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
            Theme::OUTSIDE_TABS.add_modifier(Modifier::BOLD),
        )])];
        help_lines.push(Line::from(""));

        for section in HelpSection::ALL {
            help_lines.extend(section.to_lines());
        }

        let paragraph = Paragraph::new(help_lines)
            .style(Theme::OUTSIDE_TABS)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Help")
                    .border_style(Theme::BORDER)
                    .style(Theme::OUTSIDE_TABS),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }
}
