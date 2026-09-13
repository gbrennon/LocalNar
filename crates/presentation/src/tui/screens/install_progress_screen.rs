use ratatui::{Frame, layout::Rect};

use crate::tui::widgets::ProgressWidget;

/// Renders the active model download screen.
pub struct InstallProgressScreen;

impl InstallProgressScreen {
    /// Draws download progress in the supplied content area.
    pub fn draw(frame: &mut Frame, area: Rect, widget: &mut ProgressWidget) {
        widget.draw(frame, area);
    }
}
