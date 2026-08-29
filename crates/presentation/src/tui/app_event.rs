use domain::RemoteModelFile;

/// Events that can be sent between the TUI components and async tasks.
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Search completed with results
    SearchCompleted(Vec<RemoteModelFile>),
    /// Search failed with error message
    SearchFailed(String),
    /// Install started
    InstallStarted,
    /// Install progress update (0.0-1.0 ratio, message)
    InstallProgress(f64, String),
    /// Install completed with installed model
    InstallCompleted(domain::InstalledModel),
    /// Install failed with error message
    InstallFailed(String),
    /// Quit application
    Quit,
}