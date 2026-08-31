use std::{sync::Arc, time::Duration};

use application::services::SearchModelsService;
use crossterm::{
    ExecutableCommand,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use infrastructure::{DiskModelLibrary, HfApiRegistry, HfHubDownloader};
use presentation::tui::{AppRunner, EventHandler, TuiApp};
use ratatui::{Terminal, backend::CrosstermBackend};

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
