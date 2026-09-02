use std::sync::Arc;

use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::tui::components::{
    help_section::HelpSection,
    themes::{GBadwolf, Theme},
};

/// Help widget displaying key bindings and usage information.
#[derive(Clone)]
pub struct HelpWidget {
    title: &'static str,
    theme: Arc<dyn Theme>,
}

impl HelpWidget {
    pub const TITLE: &'static str = "LocalNar TUI";

    /// Create a new help widget with default theme.
    pub fn new() -> Self {
        Self::with_theme(Arc::new(GBadwolf))
    }

    /// Create a new help widget with an injected theme.
    pub fn with_theme(theme: Arc<dyn Theme>) -> Self {
        Self {
            title: Self::TITLE,
            theme,
        }
    }

    /// Render the help widget.
    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let mut help_lines = vec![Line::from(vec![Span::styled(
            self.title,
            self.theme.content_emphasis(),
        )])];
        help_lines.push(Line::from(""));

        for section in HelpSection::ALL {
            help_lines.extend(section.to_lines(&*self.theme));
        }

        let paragraph = Paragraph::new(help_lines)
            .style(self.theme.content())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Help")
                    .border_style(self.theme.border())
                    .style(self.theme.content()),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }
}

impl Default for HelpWidget {
    fn default() -> Self {
        Self::new()
    }
}
