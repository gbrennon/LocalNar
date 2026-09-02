use std::{path::Path, sync::Arc};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use localnar_application::services::SearchModelsService;
use localnar_infrastructure::{
    DiskModelLibrary, HfApiRegistry, HfHubDownloader, ReqwestHubTransport,
};
use localnar_presentation::tui::{AppEvent, AppMode, AppTab, GBadwolf, Theme, TuiApp};
use ratatui::{
    Terminal,
    backend::TestBackend,
    style::{Color, Modifier, Style},
};
use tempfile::TempDir;

const TERMINAL_WIDTH: u16 = 100;
const TERMINAL_HEIGHT: u16 = 24;
const STRIP_ROW: u16 = 1;
const SEARCH_PROMPT_ROW: u16 = 4;
const UNREACHABLE_ENDPOINT: &str = "http://127.0.0.1:9";

fn app(models_root: &Path) -> TuiApp {
    let transport = ReqwestHubTransport::new(UNREACHABLE_ENDPOINT, None).expect("a transport");
    let registry = HfApiRegistry::new(transport);
    let search_service = Arc::new(SearchModelsService::new(registry.clone()));

    TuiApp::new(
        search_service,
        registry,
        HfHubDownloader::default(),
        DiskModelLibrary::new(models_root),
        Arc::new(GBadwolf),
    )
}

fn pressed(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn shortcut(digit: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(digit), KeyModifiers::ALT)
}

fn rendered_row(app: &mut TuiApp, row: u16) -> (String, Vec<Style>) {
    let backend = TestBackend::new(TERMINAL_WIDTH, TERMINAL_HEIGHT);
    let mut terminal = Terminal::new(backend).expect("a test terminal");

    terminal
        .draw(|frame| app.draw(frame))
        .expect("a rendered frame");

    let buffer = terminal.backend().buffer().clone();

    let text = (0..buffer.area.width)
        .map(|column| buffer[(column, row)].symbol())
        .collect::<String>();
    let styles = (0..buffer.area.width)
        .map(|column| buffer[(column, row)].style())
        .collect();

    (text, styles)
}

fn rendered_strip(app: &mut TuiApp) -> (String, Vec<Style>) {
    rendered_row(app, STRIP_ROW)
}

fn highlighted_columns_of(strip: &str, styles: &[Style], title: &str) -> Vec<Style> {
    let columns: Vec<char> = strip.chars().collect();
    let wanted: Vec<char> = title.chars().collect();
    let start = columns
        .windows(wanted.len())
        .position(|window| window == wanted.as_slice())
        .expect("a tab title on the strip");

    styles[start..start + wanted.len()].to_vec()
}

fn carries_the_highlight(styles: &[Style]) -> bool {
    styles.iter().all(|style| {
        style.bg == Some(GBadwolf::ACCENT_ORANGE)
            && style.fg == Some(GBadwolf::ROOT_BACKGROUND)
            && style.add_modifier.contains(Modifier::BOLD)
    })
}

#[test]
fn the_tabs_follow_the_order_of_the_strip() {
    assert_eq!(AppTab::ALL, [AppTab::Search, AppTab::Library, AppTab::Help]);

    for (position, tab) in AppTab::ALL.into_iter().enumerate() {
        assert_eq!(tab.index(), position);
    }
}

#[test]
fn moving_forward_walks_the_tabs_and_wraps_past_the_last() {
    assert_eq!(AppTab::Search.next(), AppTab::Library);
    assert_eq!(AppTab::Library.next(), AppTab::Help);
    assert_eq!(AppTab::Help.next(), AppTab::Search);
}

#[test]
fn moving_back_undoes_moving_forward() {
    for tab in AppTab::ALL {
        assert_eq!(tab.next().previous(), tab);
    }
}

#[test]
fn a_digit_shortcut_names_the_tab_at_that_position() {
    assert_eq!(AppTab::from_shortcut('1'), Some(AppTab::Search));
    assert_eq!(AppTab::from_shortcut('2'), Some(AppTab::Library));
    assert_eq!(AppTab::from_shortcut('3'), Some(AppTab::Help));
    assert_eq!(AppTab::from_shortcut('0'), None);
    assert_eq!(AppTab::from_shortcut('4'), None);
    assert_eq!(AppTab::from_shortcut('x'), None);
}

#[test]
fn every_mode_reports_the_tab_it_belongs_to() {
    assert_eq!(AppMode::Search.tab(), AppTab::Search);
    assert_eq!(AppMode::ModelTable.tab(), AppTab::Search);
    assert_eq!(AppMode::InstallProgress.tab(), AppTab::Search);
    assert_eq!(AppMode::Library.tab(), AppTab::Library);
    assert_eq!(AppMode::Help.tab(), AppTab::Help);
}

#[test]
fn selecting_a_tab_lands_on_that_tabs_mode() {
    assert_eq!(AppMode::from(AppTab::Search), AppMode::Search);
    assert_eq!(AppMode::from(AppTab::Library), AppMode::Library);
    assert_eq!(AppMode::from(AppTab::Help), AppMode::Help);
}

#[tokio::test]
async fn the_operator_starts_on_the_search_tab() {
    let models_root = TempDir::new().expect("temp dir");
    let app = app(models_root.path());

    assert_eq!(app.active_tab(), AppTab::Search);
}

#[tokio::test]
async fn pressing_tab_moves_to_the_next_tab() {
    let models_root = TempDir::new().expect("temp dir");
    let mut app = app(models_root.path());

    app.handle_key_event(pressed(KeyCode::Tab)).await;
    assert_eq!(app.active_tab(), AppTab::Library);

    app.handle_key_event(pressed(KeyCode::Tab)).await;
    assert_eq!(app.active_tab(), AppTab::Help);
}

#[tokio::test]
async fn pressing_shift_tab_moves_to_the_previous_tab() {
    let models_root = TempDir::new().expect("temp dir");
    let mut app = app(models_root.path());

    app.handle_key_event(pressed(KeyCode::BackTab)).await;

    assert_eq!(app.active_tab(), AppTab::Help);
}

#[tokio::test]
async fn a_digit_shortcut_jumps_straight_to_its_tab() {
    let models_root = TempDir::new().expect("temp dir");
    let mut app = app(models_root.path());

    app.handle_key_event(shortcut('2')).await;
    assert_eq!(app.active_tab(), AppTab::Library);

    app.handle_key_event(shortcut('1')).await;
    app.handle_key_event(shortcut('9')).await;
    assert_eq!(app.active_tab(), AppTab::Search);
}

#[tokio::test]
async fn a_modified_digit_never_reaches_the_search_query() {
    let models_root = TempDir::new().expect("temp dir");
    let mut app = app(models_root.path());

    app.handle_key_event(shortcut('2')).await;
    app.handle_key_event(shortcut('1')).await;

    let (prompt, _) = rendered_row(&mut app, SEARCH_PROMPT_ROW);

    assert_eq!(
        prompt.trim_matches(|character| character == '│' || character == ' '),
        ">"
    );
}

#[tokio::test]
async fn the_strip_names_every_tab() {
    let models_root = TempDir::new().expect("temp dir");
    let mut app = app(models_root.path());

    let (strip, _) = rendered_strip(&mut app);

    assert!(strip.contains("1 Search"));
    assert!(strip.contains("2 Library"));
    assert!(strip.contains("3 Help"));
    assert!(!strip.contains("Models"));
}

#[tokio::test]
async fn the_strip_highlights_the_tab_the_operator_is_on() {
    let models_root = TempDir::new().expect("temp dir");
    let mut app = app(models_root.path());

    let (strip, styles) = rendered_strip(&mut app);
    assert!(carries_the_highlight(&highlighted_columns_of(
        &strip, &styles, "1 Search"
    )));
    assert!(!carries_the_highlight(&highlighted_columns_of(
        &strip,
        &styles,
        "2 Library"
    )));

    app.handle_key_event(pressed(KeyCode::Tab)).await;

    let (strip, styles) = rendered_strip(&mut app);
    assert!(carries_the_highlight(&highlighted_columns_of(
        &strip,
        &styles,
        "2 Library"
    )));
    assert!(!carries_the_highlight(&highlighted_columns_of(
        &strip, &styles, "1 Search"
    )));
}

#[tokio::test]
async fn the_strip_stays_on_the_search_tab_while_a_model_installs() {
    let models_root = TempDir::new().expect("temp dir");
    let mut app = app(models_root.path());

    assert_eq!(AppMode::InstallProgress.tab(), AppTab::Search);

    app.handle_key_event(shortcut('2')).await;
    app.handle_key_event(shortcut('1')).await;

    assert_eq!(app.active_tab(), AppTab::Search);
}

#[tokio::test]
async fn search_state_is_preserved_when_switching_tabs() {
    let models_root = TempDir::new().expect("temp dir");
    let mut app = app(models_root.path());

    app.event_sender()
        .send(AppEvent::SearchCompleted(vec![]))
        .ok();
    app.handle_events().await;
    assert_eq!(app.mode(), AppMode::ModelTable);
    assert_eq!(app.active_tab(), AppTab::Search);

    app.handle_key_event(shortcut('2')).await;
    assert_eq!(app.active_tab(), AppTab::Library);

    app.handle_key_event(shortcut('1')).await;
    assert_eq!(app.active_tab(), AppTab::Search);
    assert_eq!(app.mode(), AppMode::ModelTable);
}

#[tokio::test]
async fn active_install_progress_is_restored_when_returning_to_search_tab() {
    let models_root = TempDir::new().expect("temp dir");
    let mut app = app(models_root.path());

    app.event_sender().send(AppEvent::InstallStarted).ok();
    app.handle_events().await;
    assert_eq!(app.mode(), AppMode::InstallProgress);

    app.handle_key_event(shortcut('2')).await;
    assert_eq!(app.active_tab(), AppTab::Library);

    app.handle_key_event(shortcut('1')).await;
    assert_eq!(app.active_tab(), AppTab::Search);
    assert_eq!(app.mode(), AppMode::InstallProgress);
}

#[tokio::test]
async fn install_progress_can_be_reopened_from_model_table() {
    let models_root = TempDir::new().expect("temp dir");
    let mut app = app(models_root.path());

    app.event_sender().send(AppEvent::InstallStarted).ok();
    app.handle_events().await;
    assert_eq!(app.mode(), AppMode::InstallProgress);

    app.handle_key_event(pressed(KeyCode::Esc)).await;
    assert_eq!(app.mode(), AppMode::ModelTable);

    app.handle_key_event(pressed(KeyCode::Char('p'))).await;
    assert_eq!(app.mode(), AppMode::InstallProgress);
}

#[tokio::test]
async fn a_custom_theme_can_be_injected() {
    struct CustomTestTheme;
    impl Theme for CustomTestTheme {
        fn name(&self) -> &'static str {
            "CustomTest"
        }
        fn tab_active(&self) -> Style {
            Style::default().fg(Color::Yellow).bg(Color::Blue)
        }
        fn tab_inactive(&self) -> Style {
            Style::default().fg(Color::Gray).bg(Color::Black)
        }
        fn border(&self) -> Style {
            Style::default().fg(Color::White)
        }
        fn content(&self) -> Style {
            Style::default().fg(Color::White)
        }
        fn content_emphasis(&self) -> Style {
            Style::default().fg(Color::White)
        }
        fn highlight(&self) -> Style {
            Style::default().fg(Color::Yellow).bg(Color::Blue)
        }
        fn status_success(&self) -> Style {
            Style::default().fg(Color::Green)
        }
        fn status_error(&self) -> Style {
            Style::default().fg(Color::Red)
        }
    }

    let models_root = TempDir::new().expect("temp dir");
    let transport = ReqwestHubTransport::new(UNREACHABLE_ENDPOINT, None).expect("a transport");
    let registry = HfApiRegistry::new(transport);
    let search_service = Arc::new(SearchModelsService::new(registry.clone()));

    let mut custom_app = TuiApp::new(
        search_service,
        registry,
        HfHubDownloader::default(),
        DiskModelLibrary::new(models_root.path()),
        Arc::new(CustomTestTheme),
    );

    let (strip, styles) = rendered_strip(&mut custom_app);
    let highlighted = highlighted_columns_of(&strip, &styles, "1 Search");
    assert!(
        highlighted
            .iter()
            .all(|s| s.fg == Some(Color::Yellow) && s.bg == Some(Color::Blue))
    );
}
