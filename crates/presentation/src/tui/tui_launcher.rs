use std::{sync::Arc, time::Duration};

use localnar_application::services::SearchModelsService;
use localnar_infrastructure::{DiskModelLibrary, HfApiRegistry, HfHubDownloader};

use crate::tui::{AppRunner, EventHandler, GBadwolf, TerminalSession, TuiApp, TuiLaunchError};

/// Bootstrap that assembles the application dependencies and opens the TUI.
pub struct TuiLauncher;

impl TuiLauncher {
    const TICK_RATE: Duration = Duration::from_millis(100);

    /// Resolve the remote catalog, build the application state, claim the terminal,
    /// and drive the run loop until the operator quits; the terminal is released when
    /// the session goes out of scope, including on failure.
    pub async fn launch() -> Result<(), TuiLaunchError> {
        let registry = HfApiRegistry::from_env()?;
        let search_service = Arc::new(SearchModelsService::new(registry.clone()));
        let mut app = TuiApp::new(
            search_service,
            registry,
            HfHubDownloader::default(),
            DiskModelLibrary::default(),
            Arc::new(GBadwolf),
        );

        let events = EventHandler::new(Self::TICK_RATE);
        let mut session = TerminalSession::open()?;

        AppRunner::run(session.terminal_mut(), &mut app, &events).await?;

        Ok(())
    }
}
