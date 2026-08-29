use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Search input widget with cursor handling and focus management.
#[derive(Debug, Default)]
pub struct SearchWidget {
    query: String,
    cursor_position: usize,
    focused: bool,
}

impl SearchWidget {
    /// Create a new search widget.
    pub fn new() -> Self {
        Self {
            query: String::new(),
            cursor_position: 0,
            focused: true,
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

    /// Set focus to the widget.
    pub fn focus(&mut self) {
        self.focused = true;
    }

    /// Remove focus from the widget.
    pub fn unfocus(&mut self) {
        self.focused = false;
    }

    /// Get the current query string.
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Render the search widget.
    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let focused_border_style = Style::default().fg(Color::Yellow);
        let unfocused_border_style = Style::default().fg(Color::Gray);

        let border_style = if self.focused {
            focused_border_style
        } else {
            unfocused_border_style
        };

        let input = format!("{}{}", Self::PROMPT_PREFIX, self.query);
        let paragraph = Paragraph::new(input)
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Self::TITLE)
                    .border_style(border_style),
            );

        frame.render_widget(paragraph, area);

        if self.focused {
            frame.set_cursor_position((
                area.x + Self::CURSOR_X_OFFSET + self.cursor_position as u16,
                area.y + Self::CURSOR_Y_OFFSET,
            ));
        }
    }
}

impl SearchWidget {
    const PROMPT_PREFIX: &'static str = "> ";
    const TITLE: &'static str = "Search Models";
    const CURSOR_X_OFFSET: u16 = 2;
    const CURSOR_Y_OFFSET: u16 = 1;
}