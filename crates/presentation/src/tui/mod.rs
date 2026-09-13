mod application;
mod composition;
mod download_progress;
mod event_loop;
mod integrations;
mod interactions;
mod layout;
mod screens;
mod terminal;
mod widgets;

pub use application::{AppEvent, AppMode, AppTab, TuiApp};
pub use composition::{TuiLaunchError, TuiLauncher};
pub use download_progress::{DownloadSpeedTracker, ProgressReporterBridge};
pub use event_loop::AppRunner;
pub use integrations::LibraryManager;
pub use interactions::EventHandler;
pub use layout::LayoutHelper;
pub use screens::{
    HelpScreen, InstallProgressScreen, LibraryScreen, ModelTableScreen, SearchScreen,
    SettingsScreen,
};
pub use terminal::TerminalSession;
pub use widgets::{
    LibraryRow, LibraryTableWidget, ModelDetails, ModelRow, ModelTableWidget, ProgressWidget,
    SettingsWidget, TabsWidget,
    themes::{self, GBadwolf, Theme},
};
