use ratatui::{Frame, layout::Rect};

use crate::tui::widgets::LibraryTableWidget;

/// Renders the installed-model library screen.
pub struct LibraryScreen;

impl LibraryScreen {
    /// Draws the installed-model table in the supplied content area.
    pub fn draw(frame: &mut Frame, area: Rect, widget: &mut LibraryTableWidget) {
        widget.draw(frame, area);
    }
}
