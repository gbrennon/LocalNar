/// A tab of the strip the operator switches screens with.
///
/// The strip is the canonical order of the TUI's screens: every move between
/// screens is a move between tabs, so this type alone decides what comes next
/// and what a digit shortcut stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTab {
    /// Where the operator states what they are looking for.
    Search,
    /// Where found models are listed and chosen for install.
    Models,
    /// Where the models this machine already holds are managed.
    Library,
    /// Where the key bindings are explained.
    Help,
}

impl AppTab {
    /// Every tab, in the order the strip renders them.
    pub const ALL: [Self; 4] = [Self::Search, Self::Models, Self::Library, Self::Help];

    /// The label the strip renders for this tab.
    ///
    /// The leading digit is the affordance for the `Alt+N` shortcut, so it must
    /// stay in step with this tab's position in `ALL`.
    pub fn title(self) -> &'static str {
        match self {
            Self::Search => "1 Search",
            Self::Models => "2 Models",
            Self::Library => "3 Library",
            Self::Help => "4 Help",
        }
    }

    /// This tab's position in `ALL`, which is the strip's selected index.
    pub fn index(self) -> usize {
        match self {
            Self::Search => 0,
            Self::Models => 1,
            Self::Library => 2,
            Self::Help => 3,
        }
    }

    /// The tab to the right, wrapping past the last one back to the first.
    pub fn next(self) -> Self {
        Self::ALL[(self.index() + 1) % Self::ALL.len()]
    }

    /// The tab to the left, wrapping past the first one back to the last.
    pub fn previous(self) -> Self {
        Self::ALL[(self.index() + Self::ALL.len() - 1) % Self::ALL.len()]
    }

    /// Names the tab a digit shortcut stands for, or nothing when the digit
    /// stands for no tab.
    pub fn from_shortcut(digit: char) -> Option<Self> {
        digit
            .to_digit(10)
            .and_then(|position| position.checked_sub(1))
            .and_then(|offset| Self::ALL.get(offset as usize).copied())
    }
}
