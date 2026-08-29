use std::sync::Arc;

use application::ports::inbound::search_models_port::SearchModelsPort;
use application::services::{InstallModelService, SearchModelsService};
use crossterm::event::{KeyCode, KeyEvent};
use domain::SearchQuery;
use infrastructure::adapters::ProgressBus;
use infrastructure::remote::huggingface::downloader::HfHubTokioTransport;
use infrastructure::{DiskModelLibrary, HfApiRegistry, HfHubDownloader, ReqwestHubTransport};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use tokio::sync::mpsc;

use crate::tui::app_event::AppEvent;
use crate::tui::app_mode::AppMode;
use crate::tui::components::{
    HelpWidget, ModelTableWidget, ProgressWidget, SearchWidget, StatusWidget,
};
use crate::tui::events::EventHandler;
use crate::tui::layout_helper::LayoutHelper;
use crate::tui::progress_reporter::ProgressReporterBridge;

/// Main TUI application struct managing the model downloader interface.
///
/// Coordinates search, the model table, installation progress, and help modes.
/// Uses concrete infrastructure adapters at the composition root.
pub struct TuiApp {
    search_service: Arc<SearchModelsService<HfApiRegistry<ReqwestHubTransport>>>,
    registry: HfApiRegistry<ReqwestHubTransport>,
    downloader: HfHubDownloader<HfHubTokioTransport>,
    library: DiskModelLibrary,
    progress_bus: ProgressBus,
    mode: AppMode,
    previous_mode: Option<AppMode>,
    search_widget: SearchWidget,
    model_table_widget: ModelTableWidget,
    progress_widget: ProgressWidget,
    status_widget: StatusWidget,
    help_widget: HelpWidget,
    event_sender: mpsc::UnboundedSender<AppEvent>,
    event_receiver: mpsc::UnboundedReceiver<AppEvent>,
    should_quit: bool,
    last_error: Option<String>,
}

impl TuiApp {
    /// Create a new TUI application with the given services.
    pub fn new(
        search_service: Arc<SearchModelsService<HfApiRegistry<ReqwestHubTransport>>>,
        registry: HfApiRegistry<ReqwestHubTransport>,
        downloader: HfHubDownloader<HfHubTokioTransport>,
        library: DiskModelLibrary,
    ) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        let progress_bus = ProgressBus::new(16);
        let _bridge = ProgressReporterBridge::new(&progress_bus, event_sender.clone());

        Self {
            search_service,
            registry,
            downloader,
            library,
            progress_bus,
            mode: AppMode::Search,
            previous_mode: None,
            search_widget: SearchWidget::new(),
            model_table_widget: ModelTableWidget::new(),
            progress_widget: ProgressWidget::new(),
            status_widget: StatusWidget::new(),
            help_widget: HelpWidget::new(),
            event_sender,
            event_receiver,
            should_quit: false,
            last_error: None,
        }
    }

    /// Get a clone of the event sender for spawning async tasks.
    pub fn event_sender(&self) -> mpsc::UnboundedSender<AppEvent> {
        self.event_sender.clone()
    }

    /// Check if the application should quit.
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Create an install service with progress reporting.
    fn make_install_service(
        &self,
    ) -> InstallModelService<
        HfApiRegistry<ReqwestHubTransport>,
        HfHubDownloader<HfHubTokioTransport>,
        DiskModelLibrary,
        infrastructure::adapters::ProgressReporter,
    > {
        let progress_reporter = self.progress_bus.sender();
        InstallModelService::new(
            self.registry.clone(),
            self.downloader.clone(),
            self.library.clone(),
            progress_reporter,
        )
    }

    /// Process pending events from the event channel.
    pub async fn handle_events(&mut self) {
        while let Ok(event) = self.event_receiver.try_recv() {
            match event {
                AppEvent::SearchCompleted(results) => {
                    self.model_table_widget.set_models(results);
                    self.mode = AppMode::ModelTable;
                    self.status_widget.set_message(Self::MSG_SEARCH_COMPLETED);
                }
                AppEvent::SearchFailed(err) => {
                    self.last_error = Some(err.clone());
                    self.status_widget.set_error(err);
                }
                AppEvent::InstallStarted => {
                    self.previous_mode = Some(self.mode);
                    self.mode = AppMode::InstallProgress;
                    self.progress_widget.reset();
                    self.status_widget.set_message(Self::MSG_INSTALL_STARTED);
                }
                AppEvent::InstallProgress(progress, msg) => {
                    self.progress_widget.set_progress(progress, msg);
                }
                AppEvent::InstallCompleted(model) => {
                    self.mode = self.previous_mode.take().unwrap_or(AppMode::ModelTable);
                    self.status_widget.set_message(format!(
                        "{}{}",
                        Self::MSG_INSTALL_COMPLETED,
                        model.spec()
                    ));
                }
                AppEvent::InstallFailed(err) => {
                    self.mode = self.previous_mode.take().unwrap_or(AppMode::ModelTable);
                    self.last_error = Some(err.clone());
                    self.status_widget.set_error(err);
                }
                AppEvent::Quit => {
                    self.should_quit = true;
                }
            }
        }
    }

    /// Handle a key event based on current mode.
    pub async fn handle_key_event(&mut self, key: KeyEvent) {
        // Dismiss error popup on any key press
        if self.last_error.is_some() {
            self.last_error = None;
            return;
        }

        // Global quit handling - works in all modes
        if EventHandler::is_quit_key(&key) {
            self.event_sender.send(AppEvent::Quit).ok();
            return;
        }

        match self.mode {
            AppMode::Search => self.handle_search_keys(key).await,
            AppMode::ModelTable => self.handle_model_table_keys(key).await,
            AppMode::InstallProgress => self.handle_install_progress_keys(key),
            AppMode::Help => self.handle_help_keys(key),
        }
    }

    async fn handle_search_keys(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) => {
                self.search_widget.input_char(c);
            }
            KeyCode::Backspace => {
                self.search_widget.input_backspace();
            }
            KeyCode::Enter => {
                if let Some(query) = self.search_widget.take_query() {
                    self.status_widget.set_message(Self::MSG_SEARCHING);
                    let search_service = self.search_service.clone();
                    let sender = self.event_sender.clone();
                    tokio::spawn(async move {
                        match SearchQuery::new(query) {
                            Ok(search_query) => match search_service.execute(&search_query).await {
                                Ok(results) => {
                                    let _ = sender.send(AppEvent::SearchCompleted(results));
                                }
                                Err(e) => {
                                    let _ = sender.send(AppEvent::SearchFailed(e.to_string()));
                                }
                            },
                            Err(e) => {
                                let _ = sender.send(AppEvent::SearchFailed(e.to_string()));
                            }
                        }
                    });
                }
            }
            KeyCode::Esc => {
                self.mode = AppMode::Help;
            }
            KeyCode::Tab => {
                self.mode = AppMode::ModelTable;
            }
            KeyCode::BackTab => {
                self.mode = AppMode::Help;
            }
            _ => {}
        }
    }

    async fn handle_model_table_keys(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => {
                self.model_table_widget.previous();
            }
            KeyCode::Down => {
                self.model_table_widget.next();
            }
            KeyCode::Enter => {
                if let Some(model) = self.model_table_widget.selected_model() {
                    let spec = model.spec().clone();
                    self.event_sender.send(AppEvent::InstallStarted).ok();

                    let sender = self.event_sender.clone();
                    let install_service = self.make_install_service();

                    tokio::spawn(async move {
                        match application::ports::inbound::InstallModelPort::execute(
                            &install_service,
                            &spec,
                        )
                        .await
                        {
                            Ok(installed) => {
                                let _ = sender.send(AppEvent::InstallCompleted(installed));
                            }
                            Err(e) => {
                                let _ = sender.send(AppEvent::InstallFailed(e.to_string()));
                            }
                        }
                    });
                }
            }
            KeyCode::Esc => {
                self.mode = AppMode::Search;
                self.search_widget.focus();
            }
            KeyCode::Char('h') | KeyCode::Char('H') => {
                self.mode = AppMode::Help;
            }
            KeyCode::Tab => {
                self.mode = AppMode::Help;
            }
            KeyCode::BackTab => {
                self.mode = AppMode::Search;
            }
            _ => {}
        }
    }

    fn handle_install_progress_keys(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Esc {
            self.mode = self.previous_mode.take().unwrap_or(AppMode::ModelTable);
        }
    }

    fn handle_help_keys(&mut self, key: KeyEvent) {
        if matches!(
            key.code,
            KeyCode::Esc | KeyCode::Char('h') | KeyCode::Char('H')
        ) {
            self.mode = self.previous_mode.unwrap_or(AppMode::Search);
        } else if matches!(key.code, KeyCode::Tab) {
            self.mode = AppMode::Search;
        } else if matches!(key.code, KeyCode::BackTab) {
            self.mode = AppMode::ModelTable;
        }
    }
    /// Render the TUI application.
    pub fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(Self::MAIN_LAYOUT_CONSTRAINTS)
            .split(area);

        self.draw_header(frame, chunks[0]);
        self.draw_content(frame, chunks[1]);
        self.status_widget.draw(frame, chunks[2]);

        if let Some(error) = &self.last_error {
            self.draw_error_popup(frame, area, error);
        }
    }

    fn draw_header(&mut self, frame: &mut Frame, area: Rect) {
        match self.mode {
            AppMode::Search => {
                self.search_widget.draw(frame, area);
            }
            AppMode::ModelTable => {
                let title = Paragraph::new(Self::MODEL_TABLE_HEADER)
                    .style(Style::default().fg(Color::Cyan))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(Self::MODEL_TABLE_TITLE),
                    );
                frame.render_widget(title, area);
            }
            AppMode::InstallProgress => {
                let title = Paragraph::new(Self::INSTALL_PROGRESS_HEADER)
                    .style(Style::default().fg(Color::Yellow))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(Self::INSTALL_PROGRESS_TITLE),
                    );
                frame.render_widget(title, area);
            }
            AppMode::Help => {
                let title = Paragraph::new(Self::HELP_HEADER)
                    .style(Style::default().fg(Color::Green))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(Self::HELP_TITLE),
                    );
                frame.render_widget(title, area);
            }
        }
    }

    fn draw_content(&mut self, frame: &mut Frame, area: Rect) {
        match self.mode {
            AppMode::Search => {
                self.draw_search_help(frame, area);
            }
            AppMode::ModelTable => {
                self.model_table_widget.draw(frame, area);
            }
            AppMode::InstallProgress => {
                self.progress_widget.draw(frame, area);
            }
            AppMode::Help => {
                self.help_widget.draw(frame, area);
            }
        }
    }

    fn draw_search_help(&self, frame: &mut Frame, area: Rect) {
        let help = Paragraph::new(Self::SEARCH_HELP_TEXT)
            .style(Style::default().fg(Color::Gray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Self::SEARCH_HELP_TITLE),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(help, area);
    }

    fn draw_error_popup(&self, frame: &mut Frame, area: Rect, error: &str) {
        let popup_area = LayoutHelper::centered_rect(
            Self::ERROR_POPUP_WIDTH_PCT,
            Self::ERROR_POPUP_HEIGHT_PCT,
            area,
        );
        let block = Block::default()
            .title(Self::ERROR_TITLE)
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Red).fg(Color::White));
        let paragraph = Paragraph::new(error)
            .style(Style::default().fg(Color::White))
            .block(block)
            .wrap(Wrap { trim: true });
        frame.render_widget(Clear, popup_area);
        frame.render_widget(paragraph, popup_area);
    }
}

impl TuiApp {
    const MAIN_LAYOUT_CONSTRAINTS: [Constraint; 3] = [
        Constraint::Length(Self::HEADER_HEIGHT),
        Constraint::Min(Self::CONTENT_MIN_HEIGHT),
        Constraint::Length(Self::STATUS_HEIGHT),
    ];

    const HEADER_HEIGHT: u16 = 3;
    const CONTENT_MIN_HEIGHT: u16 = 10;
    const STATUS_HEIGHT: u16 = 3;

    const MODEL_TABLE_HEADER: &'static str =
        "Models (↑/↓ navigate, Enter install, Esc search, h help)";
    const MODEL_TABLE_TITLE: &'static str = "Models";

    const INSTALL_PROGRESS_HEADER: &'static str = "Installing Model... (Esc to return)";
    const INSTALL_PROGRESS_TITLE: &'static str = "Install Progress";

    const HELP_HEADER: &'static str = "Help (Esc/h to close)";
    const HELP_TITLE: &'static str = "Help";

    const SEARCH_HELP_TEXT: &'static str = "Enter search query and press Enter to search models.\nTab to switch to the model table.\nEsc for help.";
    const SEARCH_HELP_TITLE: &'static str = "Search";

    const ERROR_TITLE: &'static str = "Error";
    const ERROR_POPUP_WIDTH_PCT: u16 = 60;
    const ERROR_POPUP_HEIGHT_PCT: u16 = 20;

    const MSG_SEARCH_COMPLETED: &'static str =
        "Search completed. Use ↑/↓ to navigate, Enter to install.";
    const MSG_INSTALL_STARTED: &'static str = "Installing model...";
    const MSG_INSTALL_COMPLETED: &'static str = "Installed: ";
    const MSG_SEARCHING: &'static str = "Searching...";
}
