use ratatui::{
    style::Modifier,
    text::{Line, Span},
};

use crate::tui::{components::help_line::HelpLine, theme::Theme};

/// A section of help content with a title and lines.
#[derive(Debug, Clone, Copy)]
pub struct HelpSection {
    title: &'static str,
    lines: &'static [HelpLine],
}

impl HelpSection {
    const SEARCH_MODE_LINES: &[HelpLine] = &[
        HelpLine::Plain("Type query, press Enter to search"),
        HelpLine::KeyBinding {
            key: "Tab",
            description: "Move to the library tab",
        },
        HelpLine::KeyBinding {
            key: "Esc",
            description: "Open the help tab",
        },
    ];

    const MODEL_TABLE_MODE_LINES: &[HelpLine] = &[
        HelpLine::KeyBinding {
            key: "↑/↓",
            description: "Navigate models",
        },
        HelpLine::KeyBinding {
            key: "Enter",
            description: "Install selected model",
        },
        HelpLine::KeyBinding {
            key: "l",
            description: "Manage installed models",
        },
        HelpLine::KeyBinding {
            key: "Esc",
            description: "Return to search input",
        },
        HelpLine::KeyBinding {
            key: "h / ?",
            description: "Open help",
        },
    ];

    const INSTALL_PROGRESS_MODE_LINES: &[HelpLine] = &[HelpLine::KeyBinding {
        key: "Esc",
        description: "Return to model table (install continues)",
    }];

    const LIBRARY_MODE_LINES: &[HelpLine] = &[
        HelpLine::Plain("Total control over the models this machine holds"),
        HelpLine::KeyBinding {
            key: "↑/↓",
            description: "Navigate installed models",
        },
        HelpLine::KeyBinding {
            key: "Enter / i",
            description: "Inspect selected model in full",
        },
        HelpLine::KeyBinding {
            key: "v",
            description: "Verify selected model against its digest",
        },
        HelpLine::KeyBinding {
            key: "d",
            description: "Delete selected model (asks to confirm)",
        },
        HelpLine::KeyBinding {
            key: "p",
            description: "Prune leftovers that stand for no model",
        },
        HelpLine::KeyBinding {
            key: "r",
            description: "Re-read the library",
        },
        HelpLine::KeyBinding {
            key: "h / ?",
            description: "Open the help tab",
        },
        HelpLine::KeyBinding {
            key: "Esc",
            description: "Close details, or return to the search tab",
        },
    ];

    const GENERAL_LINES: &[HelpLine] = &[
        HelpLine::KeyBinding {
            key: "Tab / Shift+Tab",
            description: "Move to the next / previous tab",
        },
        HelpLine::KeyBinding {
            key: "Alt+1..Alt+3",
            description: "Jump straight to a tab",
        },
        HelpLine::KeyBinding {
            key: "h / ?",
            description: "Toggle help",
        },
        HelpLine::KeyBinding {
            key: "Ctrl+Q / Ctrl+C",
            description: "Quit application",
        },
    ];

    const SEARCH_MODE: Self = Self {
        title: "Search Mode",
        lines: Self::SEARCH_MODE_LINES,
    };

    const MODEL_TABLE_MODE: Self = Self {
        title: "Model Table Mode",
        lines: Self::MODEL_TABLE_MODE_LINES,
    };

    const INSTALL_PROGRESS_MODE: Self = Self {
        title: "Install Progress Mode",
        lines: Self::INSTALL_PROGRESS_MODE_LINES,
    };

    const LIBRARY_MODE: Self = Self {
        title: "Library Mode",
        lines: Self::LIBRARY_MODE_LINES,
    };

    const GENERAL: Self = Self {
        title: "General",
        lines: Self::GENERAL_LINES,
    };

    /// All help sections in display order.
    pub const ALL: [&'static Self; 5] = [
        &Self::SEARCH_MODE,
        &Self::MODEL_TABLE_MODE,
        &Self::INSTALL_PROGRESS_MODE,
        &Self::LIBRARY_MODE,
        &Self::GENERAL,
    ];

    /// Renders the section title and every line beneath it.
    pub fn to_lines(self) -> Vec<Line<'static>> {
        let mut lines = vec![Line::from(vec![Span::styled(
            self.title,
            Theme::OUTSIDE_TABS.add_modifier(Modifier::BOLD),
        )])];

        for line in self.lines {
            match line {
                HelpLine::Plain(text) => lines.push(Line::from(*text)),
                HelpLine::KeyBinding { key, description } => {
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("  {key:<16}"),
                            Theme::OUTSIDE_TABS.add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(format!(" - {description}"), Theme::OUTSIDE_TABS),
                    ]));
                }
            }
        }
        lines.push(Line::from(""));
        lines
    }
}
