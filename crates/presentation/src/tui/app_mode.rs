/// Application mode enumeration representing the current TUI state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    /// Search mode - user enters search query
    Search,
    /// Model list mode - displays search results
    ModelList,
    /// Install progress mode - shows download/install progress
    InstallProgress,
    /// Help mode - displays key bindings and usage
    Help,
}