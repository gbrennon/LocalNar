use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::tui::widgets::themes::Theme;

/// Renders the search screen's instructions.
pub struct SearchScreen;

impl SearchScreen {
    /// Draws the search instructions in the supplied content area.
    pub fn draw(frame: &mut Frame, area: Rect, theme: &dyn Theme) {
        let help = Paragraph::new(
            "Enter search query and press Enter to search models.\nTab / Shift+Tab move between tabs; Alt+1..Alt+4 jump straight to one.\nEsc opens the help tab.",
        )
        .style(theme.content())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Search")
                .title_style(theme.title())
                .border_style(theme.border())
                .style(theme.content()),
        )
        .wrap(Wrap { trim: true });
        frame.render_widget(help, area);
    }
}
