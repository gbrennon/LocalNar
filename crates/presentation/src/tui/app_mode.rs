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
    /// The mode reached by cycling forward from this one.
    ///
    /// The cycle follows the operator's own order of work: find a model, choose
    /// one, manage what the machine ended up holding, then read the help.
    pub fn next(self) -> Self {
        match self {
            Self::Search => Self::ModelTable,
            Self::ModelTable => Self::Library,
            Self::Library => Self::Help,
            Self::Help => Self::Search,
            Self::InstallProgress => Self::Library,
        }
    }

    /// The mode reached by cycling backward from this one.
    pub fn previous(self) -> Self {
        match self {
            Self::Search => Self::Help,
            Self::ModelTable => Self::Search,
            Self::Library => Self::ModelTable,
            Self::Help => Self::Library,
            Self::InstallProgress => Self::ModelTable,
        }
    }
}

#[cfg(test)]
mod app_mode_tests {
    use super::AppMode;

    #[test]
    fn cycling_forward_visits_every_mode_and_returns_to_the_first() {
        let mut mode = AppMode::Search;
        let mut visited = vec![mode];

        for _ in 0..3 {
            mode = mode.next();
            visited.push(mode);
        }

        assert_eq!(
            visited,
            vec![
                AppMode::Search,
                AppMode::ModelTable,
                AppMode::Library,
                AppMode::Help
            ]
        );
        assert_eq!(mode.next(), AppMode::Search);
    }

    #[test]
    fn cycling_backward_undoes_cycling_forward() {
        for mode in [
            AppMode::Search,
            AppMode::ModelTable,
            AppMode::Library,
            AppMode::Help,
        ] {
            assert_eq!(mode.next().previous(), mode);
        }
    }
}
