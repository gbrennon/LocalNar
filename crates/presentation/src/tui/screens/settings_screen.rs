use ratatui::{Frame, layout::Rect};

use crate::tui::widgets::SettingsWidget;

/// Renders the settings screen.
pub struct SettingsScreen;

impl SettingsScreen {
    /// Draws settings content in the supplied content area.
    pub fn draw(frame: &mut Frame, area: Rect, widget: &mut SettingsWidget) {
        widget.draw(frame, area);
    }
}
