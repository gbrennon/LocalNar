use ratatui::{Frame, layout::Rect};

use crate::tui::widgets::HelpWidget;

/// Renders the help screen.
pub struct HelpScreen;

impl HelpScreen {
    /// Draws help content in the supplied content area.
    pub fn draw(frame: &mut Frame, area: Rect, widget: &mut HelpWidget) {
        widget.draw(frame, area);
    }
}
