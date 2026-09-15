use std::sync::Arc;

use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::tui::widgets::themes::{GBadwolf, Theme};

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
        self.query.push(c);
        self.cursor_position = self.query.chars().count();
    }

    /// Delete the character before the cursor.
    pub fn input_backspace(&mut self) {
        if self.query.pop().is_some() {
            self.cursor_position = self.query.chars().count();
        }
    }

    /// Returns the trimmed query and clears the input field.
    pub fn take_query(&mut self) -> String {
        let query = self.query.trim().to_string();
        self.query.clear();
        self.cursor_position = 0;
        query
    }

    /// Answers whether the placeholder hint is shown in place of a typed query.
    pub fn is_showing_placeholder(&self) -> bool {
        self.query.is_empty()
    }

    /// Answers whether the operator has entered a non-empty query.
    pub fn has_query(&self) -> bool {
        !self.query.is_empty()
    }

    /// Render the search widget.
    ///
    /// Shows a dimmed Hugging Face style placeholder while the query is empty so
    /// the operator sees an example query without having to type first, then
    /// swaps to the emphasized typed text once input begins.
    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let body = if self.is_showing_placeholder() {
            Span::styled(Self::PLACEHOLDER, self.theme.content())
        } else {
            Span::styled(self.query.clone(), self.theme.content_emphasis())
        };
        let line = Line::from(vec![
            Span::styled(Self::PROMPT_PREFIX, self.theme.content_emphasis()),
            body,
        ]);
        let paragraph = Paragraph::new(line).block(
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
    const PLACEHOLDER: &'static str = "Search models, datasets, users...";
    const CURSOR_X_OFFSET: u16 = 3;
    const CURSOR_Y_OFFSET: u16 = 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_widget_shows_placeholder() {
        let widget = SearchWidget::new();

        assert!(widget.is_showing_placeholder());
    }

    #[test]
    fn typing_a_character_hides_the_placeholder() {
        let mut widget = SearchWidget::new();

        widget.input_char('b');

        assert!(!widget.is_showing_placeholder());
    }

    #[test]
    fn deleting_the_last_character_restores_the_placeholder() {
        let mut widget = SearchWidget::new();
        widget.input_char('b');

        widget.input_backspace();

        assert!(widget.is_showing_placeholder());
    }

    #[test]
    fn take_query_returns_trimmed_input_and_restores_placeholder() {
        let mut widget = SearchWidget::new();
        "  bert  ".chars().for_each(|c| widget.input_char(c));

        let query = widget.take_query();

        assert_eq!(query, "bert");
        assert!(widget.is_showing_placeholder());
    }

    #[test]
    fn cursor_stays_after_multibyte_input() {
        let mut widget = SearchWidget::new();

        widget.input_char('é');
        widget.input_char('a');

        assert_eq!(widget.query, "éa");
        assert_eq!(widget.cursor_position, "éa".chars().count());
    }

    #[test]
    fn cursor_offset_leaves_cursor_after_prompt_and_border() {
        assert_eq!(SearchWidget::CURSOR_X_OFFSET, 3);
    }
}
