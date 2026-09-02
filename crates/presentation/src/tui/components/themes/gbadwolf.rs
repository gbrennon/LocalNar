use ratatui::style::{Color, Modifier, Style};

use crate::tui::components::themes::Theme;

/// Badwolf visual theme inspired by the tmux environment and Badwolf color palette.
///
/// Orange (`#d75f00`) is reserved as an active accent, while passive elements, borders,
/// and inactive tabs rest on muted grays and dark surfaces to ensure comfortable contrast.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct GBadwolf;

impl GBadwolf {
    /// Accent orange (`colour166` in tmux / `#d75f00`).
    pub const ACCENT_ORANGE: Color = Color::Rgb(215, 95, 0);

    /// Root background (pure black `#000000` for maximum contrast with accent orange).
    pub const ROOT_BACKGROUND: Color = Color::Rgb(0, 0, 0);

    /// Surface background (`colour234` / `#1c1c1c` / dark surface).
    pub const SURFACE_BACKGROUND: Color = Color::Rgb(24, 24, 24);

    /// Subtle border gray (`colour236` in tmux / `#303030`).
    pub const BORDER_COLOR: Color = Color::Rgb(48, 48, 48);

    /// Muted body text gray (`colour245` in tmux / `#8a8a8a`).
    pub const MUTED_FOREGROUND: Color = Color::Rgb(138, 138, 138);

    /// Bright text for headings and emphasized values.
    pub const BRIGHT_FOREGROUND: Color = Color::Rgb(245, 245, 245);

    /// Active tab style: dark text on accent orange background with bold font.
    pub const TAB_ACTIVE: Style = Style::new()
        .fg(Self::ROOT_BACKGROUND)
        .bg(Self::ACCENT_ORANGE)
        .add_modifier(Modifier::BOLD);

    /// Inactive tab style: muted gray text on dark surface background.
    pub const TAB_INACTIVE: Style = Style::new()
        .fg(Self::MUTED_FOREGROUND)
        .bg(Self::SURFACE_BACKGROUND);

    /// Border style: subtle dark gray lines on root background.
    pub const BORDER: Style = Style::new()
        .fg(Self::BORDER_COLOR)
        .bg(Self::ROOT_BACKGROUND);

    /// Content style: muted gray text on root background.
    pub const CONTENT: Style = Style::new()
        .fg(Self::MUTED_FOREGROUND)
        .bg(Self::ROOT_BACKGROUND);

    /// Emphasized content style: bright bold text on root background.
    pub const CONTENT_EMPHASIS: Style = Style::new()
        .fg(Self::BRIGHT_FOREGROUND)
        .bg(Self::ROOT_BACKGROUND)
        .add_modifier(Modifier::BOLD);

    /// Table row selection highlight: dark text on accent orange background.
    pub const HIGHLIGHT: Style = Style::new()
        .fg(Self::ROOT_BACKGROUND)
        .bg(Self::ACCENT_ORANGE)
        .add_modifier(Modifier::BOLD);

    /// Success status style: green text on root background.
    pub const STATUS_SUCCESS: Style = Style::new().fg(Color::Green).bg(Self::ROOT_BACKGROUND);

    /// Error status style: red text on root background.
    pub const STATUS_ERROR: Style = Style::new().fg(Color::Red).bg(Self::ROOT_BACKGROUND);

    /// Create a new GBadwolf theme instance.
    pub fn new() -> Self {
        Self
    }
}

impl Theme for GBadwolf {
    fn name(&self) -> &'static str {
        "gBadwolf"
    }

    fn tab_active(&self) -> Style {
        Self::TAB_ACTIVE
    }

    fn tab_inactive(&self) -> Style {
        Self::TAB_INACTIVE
    }

    fn border(&self) -> Style {
        Self::BORDER
    }

    fn content(&self) -> Style {
        Self::CONTENT
    }

    fn content_emphasis(&self) -> Style {
        Self::CONTENT_EMPHASIS
    }

    fn highlight(&self) -> Style {
        Self::HIGHLIGHT
    }

    fn status_success(&self) -> Style {
        Self::STATUS_SUCCESS
    }

    fn status_error(&self) -> Style {
        Self::STATUS_ERROR
    }
}
