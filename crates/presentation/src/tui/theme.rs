use ratatui::style::{Color, Modifier, Style};

/// Visual theme defining the color palette and styles for the TUI application.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Theme;

impl Theme {
    /// Gruvbox orange used for tab background and foreground text outside tabs.
    pub const PRIMARY: Color = Color::Rgb(214, 93, 14);

    /// Dark background used for tab font and background outside tabs.
    pub const BACKGROUND: Color = Color::Rgb(28, 27, 26);

    /// Style for the active tab: background #d65d0e and font #1c1b1a.
    pub const TAB_ACTIVE: Style = Style::new()
        .fg(Self::BACKGROUND)
        .bg(Self::PRIMARY)
        .add_modifier(Modifier::BOLD);

    /// Style for inactive tabs: background #1c1b1a and font #d65d0e.
    pub const TAB_INACTIVE: Style = Style::new().fg(Self::PRIMARY).bg(Self::BACKGROUND);

    /// Style for content and widgets outside tabs: background #1c1b1a and font #d65d0e.
    pub const OUTSIDE_TABS: Style = Style::new().fg(Self::PRIMARY).bg(Self::BACKGROUND);

    /// Style for borders outside tabs: foreground #d65d0e and background #1c1b1a.
    pub const BORDER: Style = Style::new().fg(Self::PRIMARY).bg(Self::BACKGROUND);

    /// Style for highlighted selections in tables.
    pub const HIGHLIGHT: Style = Style::new()
        .fg(Self::BACKGROUND)
        .bg(Self::PRIMARY)
        .add_modifier(Modifier::BOLD);

    /// Creates a new theme instance.
    pub fn new() -> Self {
        Self
    }

    /// Returns the active tab style.
    pub fn tab_active(&self) -> Style {
        Self::TAB_ACTIVE
    }

    /// Returns the inactive tab style.
    pub fn tab_inactive(&self) -> Style {
        Self::TAB_INACTIVE
    }

    /// Returns the style for elements outside tabs.
    pub fn outside_tabs(&self) -> Style {
        Self::OUTSIDE_TABS
    }

    /// Returns the border style outside tabs.
    pub fn border(&self) -> Style {
        Self::BORDER
    }

    /// Returns the selection highlight style.
    pub fn highlight(&self) -> Style {
        Self::HIGHLIGHT
    }
}
