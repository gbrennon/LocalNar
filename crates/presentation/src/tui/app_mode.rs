use crate::tui::app_tab::AppTab;

/// Application mode enumeration representing the current TUI state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    /// Entering a search query.
    Search,
    /// Browsing the search results table.
    ModelTable,
    /// Watching an install progress.
    InstallProgress,
    /// Managing the installed-model library.
    Library,
    /// Managing operator settings.
    Settings,
    /// Reading the help screen.
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
            Self::Settings => AppTab::Settings,
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
            AppTab::Settings => Self::Settings,
            AppTab::Help => Self::Help,
        }
    }
}
