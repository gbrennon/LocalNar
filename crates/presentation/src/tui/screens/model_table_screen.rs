use ratatui::{Frame, layout::Rect};

use crate::tui::widgets::ModelTableWidget;

/// Renders the model search results screen.
pub struct ModelTableScreen;

impl ModelTableScreen {
    /// Draws the model table in the supplied content area.
    pub fn draw(frame: &mut Frame, area: Rect, widget: &mut ModelTableWidget) {
        widget.draw(frame, area);
    }
}
