use std::{sync::Arc, time::Duration};

use localnar_application::{
    ports::inbound::LoadSettingsPort,
    services::{LoadSettingsService, SaveSettingsService, SearchModelsService},
};
use localnar_infrastructure::{
    DiskModelLibrary, HfApiRegistry, HfHubDownloader, TomlSettingsStore,
};

use crate::tui::{AppRunner, EventHandler, GBadwolf, TerminalSession, TuiApp, TuiLaunchError};

/// Composition root that assembles and runs the TUI.
pub struct TuiLauncher;

impl TuiLauncher {
    const TICK_RATE: Duration = Duration::from_millis(100);

    /// Loads persisted settings, builds every adapter from them, and runs the
    /// TUI until the operator quits.
    pub async fn launch() -> Result<(), TuiLaunchError> {
        let store = TomlSettingsStore::default();
        let settings = LoadSettingsService::new(store.clone()).execute()?;

        let registry = HfApiRegistry::from_settings(&settings)?;
        let search_service = Arc::new(SearchModelsService::new(registry.clone()));
        let downloader = HfHubDownloader::from_settings(&settings);
        let library = DiskModelLibrary::from_settings(&settings);
        let save_settings = Arc::new(SaveSettingsService::new(store));

        let mut app = TuiApp::new(
            search_service,
            registry,
            downloader,
            library,
            settings,
            save_settings,
            Arc::new(GBadwolf),
        );

        let events = EventHandler::new(Self::TICK_RATE);
        let mut session = TerminalSession::open()?;

        AppRunner::run(session.terminal_mut(), &mut app, &events).await?;

        Ok(())
    }
}
