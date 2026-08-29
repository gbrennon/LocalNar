use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::tui::components::help_line::HelpLine;

/// A section of help content with a title and lines.
#[derive(Debug, Clone, Copy)]
pub struct HelpSection {
    title: &'static str,
    lines: &'static [HelpLine],
}

impl HelpSection {
    const SEARCH_MODE_LINES: &[HelpLine] = &[
        HelpLine::Plain("Type query, press Enter to search"),
        HelpLine::KeyBinding { key: "Tab", description: "Switch to model table" },
        HelpLine::KeyBinding { key: "Esc", description: "Open help" },
    ];

    const MODEL_TABLE_MODE_LINES: &[HelpLine] = &[
        HelpLine::KeyBinding { key: "↑/↓", description: "Navigate models" },
        HelpLine::KeyBinding { key: "Enter", description: "Install selected model" },
        HelpLine::KeyBinding { key: "Esc", description: "Return to search" },
        HelpLine::KeyBinding { key: "h", description: "Open help" },
    ];

    const INSTALL_PROGRESS_MODE_LINES: &[HelpLine] = &[HelpLine::KeyBinding {
        key: "Esc",
        description: "Return to model table (install continues)",
    }];

    const GENERAL_LINES: &[HelpLine] = &[
        HelpLine::KeyBinding { key: "h / ?", description: "Toggle help" },
        HelpLine::KeyBinding { key: "Ctrl+Q / Ctrl+C", description: "Quit application" },
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

    const GENERAL: Self = Self {
        title: "General",
        lines: Self::GENERAL_LINES,
    };

    /// All help sections in display order.
    pub const ALL: [&'static Self; 4] = [
        &Self::SEARCH_MODE,
        &Self::MODEL_TABLE_MODE,
        &Self::INSTALL_PROGRESS_MODE,
        &Self::GENERAL,
    ];

    /// Convert section to renderable lines.
    pub fn to_lines(&self) -> Vec<Line<'static>> {
        let mut lines = vec![Line::from(vec![Span::styled(
            self.title,
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )])];

        for line in self.lines {
            match line {
                HelpLine::Plain(text) => lines.push(Line::from(*text)),
                HelpLine::KeyBinding { key, description } => {
                    lines.push(Line::from(vec![
                        Span::styled(format!("  {key:<16}"), Style::default().fg(Color::Cyan)),
                        Span::styled(format!(" - {description}"), Style::default().fg(Color::White)),
                    ]));
                }
                HelpLine::Header(text) => {
                    lines.push(Line::from(vec![Span::styled(
                        *text,
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    )]));
                }
            }
        }
        lines.push(Line::from(""));
        lines
    }
}