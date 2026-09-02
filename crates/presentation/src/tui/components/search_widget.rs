use std::sync::Arc;

use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
};

use crate::tui::components::themes::{GBadwolf, Theme};

/// Search input widget with cursor handling.
#[derive(Clone)]
pub struct SearchWidget {
    query: String,
    cursor_position: usize,
    theme: Arc<dyn Theme>,
}

impl SearchWidget {
    /// Create a new search widget with default theme.
    pub fn new() -> Self {
        Self::with_theme(Arc::new(GBadwolf))
    }

    /// Create a new search widget with an injected theme.
    pub fn with_theme(theme: Arc<dyn Theme>) -> Self {
        Self {
            query: String::new(),
            cursor_position: 0,
            theme,
        }
    }

    /// Insert a character at the cursor position.
    pub fn input_char(&mut self, c: char) {
        self.query.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    /// Delete the character before the cursor.
    pub fn input_backspace(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.query.remove(self.cursor_position);
        }
    }

    /// Take the current query if non-empty, clearing the widget.
    pub fn take_query(&mut self) -> Option<String> {
        let query = self.query.trim().to_string();
        if query.is_empty() {
            None
        } else {
            self.query.clear();
            self.cursor_position = 0;
            Some(query)
        }
    }

    /// Render the search widget.
    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let input = format!("{}{}", Self::PROMPT_PREFIX, self.query);
        let paragraph = Paragraph::new(input)
            .style(self.theme.content_emphasis())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Self::TITLE)
                    .title_style(self.theme.title())
                    .border_style(self.theme.border())
                    .style(self.theme.content()),
            );

        frame.render_widget(paragraph, area);

        frame.set_cursor_position((
            area.x + Self::CURSOR_X_OFFSET + self.cursor_position as u16,
            area.y + Self::CURSOR_Y_OFFSET,
        ));
    }
}

impl Default for SearchWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchWidget {
    const PROMPT_PREFIX: &'static str = "> ";
    const TITLE: &'static str = "Search Models";
    const CURSOR_X_OFFSET: u16 = 2;
    const CURSOR_Y_OFFSET: u16 = 1;
}
