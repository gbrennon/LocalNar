/// Application mode enumeration representing the current TUI state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    /// Search mode - user enters search query
    Search,
    /// Model table mode - displays one row per model found
    ModelTable,
    /// Install progress mode - shows download/install progress
    InstallProgress,
    /// Help mode - displays key bindings and usage
    Help,
}