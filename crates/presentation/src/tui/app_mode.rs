use crate::tui::app_tab::AppTab;

/// Application mode enumeration representing the current TUI state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    /// Search mode - user enters search query
    Search,
    /// Model table mode - displays one row per model found
    ModelTable,
    /// Install progress mode - shows download/install progress
    InstallProgress,
    /// Library mode - manages the models this machine already holds
    Library,
    /// Help mode - displays key bindings and usage
    Help,
}

impl AppMode {
    /// Names the tab whose strip entry stands for this mode.
    ///
    /// The model table and install progress screens appear as results of search,
    /// so they keep the search tab highlighted.
    pub fn tab(self) -> AppTab {
        match self {
            Self::Search | Self::ModelTable | Self::InstallProgress => AppTab::Search,
            Self::Library => AppTab::Library,
            Self::Help => AppTab::Help,
        }
    }
}

impl From<AppTab> for AppMode {
    /// The mode a strip selection lands on.
    fn from(tab: AppTab) -> Self {
        match tab {
            AppTab::Search => Self::Search,
            AppTab::Library => Self::Library,
            AppTab::Help => Self::Help,
        }
    }
}
