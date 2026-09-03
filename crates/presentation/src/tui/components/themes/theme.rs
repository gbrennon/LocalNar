use ratatui::style::Style;

/// Contract defining visual styling and colors for the TUI application.
///
/// Implementors provide cohesive palettes and style definitions that can be
/// injected across widgets and application components.
pub trait Theme: Send + Sync {
    /// Human-readable identifier for this theme.
    fn name(&self) -> &'static str;

    /// Style for the active tab item.
    fn tab_active(&self) -> Style;

    /// Style for inactive tab items.
    fn tab_inactive(&self) -> Style;

    /// Style for container and divider borders.
    fn border(&self) -> Style;

    /// Style for container and block titles.
    fn title(&self) -> Style;

    /// Style for regular body and descriptive text.
    fn content(&self) -> Style;

    /// Style for headers, titles, and emphasized labels.
    fn content_emphasis(&self) -> Style;

    /// Style for selected rows and highlighted UI elements.
    fn highlight(&self) -> Style;

    /// Style for success or verified status messages.
    fn status_success(&self) -> Style;

    /// Style for error or broken status messages.
    fn status_error(&self) -> Style;
}
