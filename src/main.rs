use std::sync::Arc;
use std::time::Duration;

use application::ports::inbound::search_models_port::SearchModelsPort;
use application::ports::outbound::model_downloader_port::ModelDownloaderPort;
use application::ports::outbound::model_library_port::ModelLibraryPort;
use application::ports::outbound::remote_model_registry_port::RemoteModelRegistryPort;
use application::services::SearchModelsService;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use infrastructure::{DiskModelLibrary, HfApiRegistry, HfHubDownloader};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use presentation::tui::{AppRunner, EventHandler, TuiApp};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model_library = DiskModelLibrary::default();
    let registry = HfApiRegistry::from_env()?;
    let downloader = HfHubDownloader::default();

    let search_service = Arc::new(SearchModelsService::new(registry.clone()));

    enable_raw_mode()?;
    std::io::stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let events = EventHandler::new(Duration::from_millis(100));

    let mut app = TuiApp::new(search_service, registry, downloader, model_library);

    let result = AppRunner::run(&mut terminal, &mut app, &events).await;

    disable_raw_mode()?;
    std::io::stdout().execute(LeaveAlternateScreen)?;

    result?;
    Ok(())
}