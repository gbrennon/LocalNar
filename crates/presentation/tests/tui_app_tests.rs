use std::sync::Arc;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use localnar_application::services::SearchModelsService;
use localnar_domain::{
    ByteLength, Checksum, InstalledModel, ModelFileName, ModelRepository, ModelRepositoryId,
    ModelSpec,
};
use localnar_infrastructure::{
    DiskModelLibrary, HfApiRegistry, HfHubDownloader, ReqwestHubTransport,
    remote::huggingface::downloader::HfHubTokioTransport,
};
use localnar_presentation::tui::{AppEvent, AppMode, AppTab, GBadwolf, TuiApp};
use ratatui::{Terminal, backend::TestBackend};
use tempfile::TempDir;

const WIDTH: u16 = 100;
const HEIGHT: u16 = 20;

fn create_app(temp_dir: &TempDir) -> TuiApp {
    let transport = ReqwestHubTransport::new("http://127.0.0.1:0", None).expect("transport");
    let registry = HfApiRegistry::new(transport);
    let search_service = Arc::new(SearchModelsService::new(registry.clone()));
    let downloader_transport =
        HfHubTokioTransport::new(temp_dir.path(), "http://127.0.0.1:0", None);
    let downloader = HfHubDownloader::new(downloader_transport);
    let library = DiskModelLibrary::new(temp_dir.path());
    let theme = Arc::new(GBadwolf);

    TuiApp::new(search_service, registry, downloader, library, theme)
}

fn render_screen(app: &mut TuiApp) -> String {
    let backend = TestBackend::new(WIDTH, HEIGHT);
    let mut terminal = Terminal::new(backend).expect("terminal");
    terminal.draw(|frame| app.draw(frame)).expect("draw");

    let buffer = terminal.backend().buffer().clone();
    (0..HEIGHT)
        .map(|y| {
            (0..WIDTH)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn test_spec() -> ModelSpec {
    let identifier = ModelRepositoryId::parse("unsloth/Qwen3-8B-GGUF").expect("valid id");
    ModelSpec::new(
        ModelRepository::at_default_revision(identifier),
        ModelFileName::new("Qwen3-8B-Q4_K_M.gguf").expect("valid file name"),
        vec![],
    )
}

#[tokio::test]
async fn app_tracks_download_progress_and_displays_it_in_library_table() {
    let temp_dir = TempDir::new().expect("temp dir");
    let mut app = create_app(&temp_dir);
    let sender = app.event_sender();
    let spec = test_spec();

    sender.send(AppEvent::InstallStarted).expect("send started");
    sender
        .send(AppEvent::InstallProgress(
            0.55,
            "Downloading 55%".to_owned(),
        ))
        .expect("send progress");

    app.handle_events().await;

    let progress_screen = render_screen(&mut app);
    assert!(progress_screen.contains("55.0%"), "{progress_screen}");
    assert!(
        progress_screen.contains("Downloading 55%"),
        "{progress_screen}"
    );

    app.handle_key_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE))
        .await;
    let installed_replica = InstalledModel::new(
        spec,
        temp_dir.path().join("model.gguf"),
        ByteLength::new(4_000_000_000),
        Some(Checksum::from_bytes([0xaa; 32])),
    );
    sender
        .send(AppEvent::InstallCompleted(installed_replica))
        .expect("send completed");
    app.handle_events().await;

    assert_eq!(app.active_tab(), AppTab::Library);
}

#[tokio::test]
async fn app_reloads_library_when_switching_to_library_tab() {
    let temp_dir = TempDir::new().expect("temp dir");
    let mut app = create_app(&temp_dir);

    app.handle_key_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE))
        .await;
    app.handle_events().await;

    assert_eq!(app.active_tab(), AppTab::Library);
    assert_eq!(app.mode(), AppMode::Library);

    let screen = render_screen(&mut app);
    assert!(screen.contains("Repository"), "{screen}");
    assert!(screen.contains("File"), "{screen}");
    assert!(screen.contains("State"), "{screen}");
}
