use std::sync::Arc;

use application::{
    ports::inbound::search_models_port::SearchModelsPort,
    services::{InstallModelService, SearchModelsService},
};
use crossterm::event::{KeyCode, KeyEvent};
use domain::{DiscardedStray, ManagedModel, ModelSpec, SearchQuery};
use infrastructure::{
    DiskModelLibrary, HfApiRegistry, HfHubDownloader, ReqwestHubTransport, adapters::ProgressBus,
    remote::huggingface::downloader::HfHubTokioTransport,
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use tokio::sync::mpsc;

use crate::tui::{
    app_event::AppEvent,
    app_mode::AppMode,
    components::{
        HelpWidget, LibraryTableWidget, ModelDetails, ModelTableWidget, ProgressWidget,
        SearchWidget, StatusWidget,
    },
    events::EventHandler,
    layout_helper::LayoutHelper,
    library_manager::LibraryManager,
    progress_reporter::ProgressReporterBridge,
};

/// Main TUI application struct managing the model downloader interface.
///
/// Coordinates search, the model table, installation progress, the library of
/// models this machine already holds, and help modes. Uses concrete
/// infrastructure adapters at the composition root.
pub struct TuiApp {
    search_service: Arc<SearchModelsService<HfApiRegistry<ReqwestHubTransport>>>,
    registry: HfApiRegistry<ReqwestHubTransport>,
    downloader: HfHubDownloader<HfHubTokioTransport>,
    library: DiskModelLibrary,
    library_manager: LibraryManager,
    progress_bus: ProgressBus,
    mode: AppMode,
    previous_mode: Option<AppMode>,
    search_widget: SearchWidget,
    model_table_widget: ModelTableWidget,
    library_table_widget: LibraryTableWidget,
    progress_widget: ProgressWidget,
    status_widget: StatusWidget,
    help_widget: HelpWidget,
    details: Option<ManagedModel>,
    pending_removal: Option<ModelSpec>,
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
        let library_manager = LibraryManager::new(library.clone(), event_sender.clone());

        Self {
            search_service,
            registry,
            downloader,
            library,
            library_manager,
            progress_bus,
            mode: AppMode::Search,
            previous_mode: None,
            search_widget: SearchWidget::new(),
            model_table_widget: ModelTableWidget::new(),
            library_table_widget: LibraryTableWidget::new(),
            progress_widget: ProgressWidget::new(),
            status_widget: StatusWidget::new(),
            help_widget: HelpWidget::new(),
            details: None,
            pending_removal: None,
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
                    self.model_table_widget.show(results);
                    self.mode = AppMode::ModelTable;
                    self.status_widget.report(Self::MSG_SEARCH_COMPLETED);
                }
                AppEvent::SearchFailed(err) => {
                    self.raise_failure(err);
                }
                AppEvent::InstallStarted => {
                    self.previous_mode = Some(self.mode);
                    self.mode = AppMode::InstallProgress;
                    self.progress_widget.reset();
                    self.status_widget.report(Self::MSG_INSTALL_STARTED);
                }
                AppEvent::InstallProgress(progress, msg) => {
                    self.progress_widget.advance(progress, msg);
                }
                AppEvent::InstallCompleted(model) => {
                    self.mode = self.previous_mode.take().unwrap_or(AppMode::ModelTable);
                    self.status_widget.report(format!(
                        "{}{}",
                        Self::MSG_INSTALL_COMPLETED,
                        model.spec()
                    ));
                    self.library_manager.list();
                }
                AppEvent::InstallFailed(err) => {
                    self.mode = self.previous_mode.take().unwrap_or(AppMode::ModelTable);
                    self.raise_failure(err);
                }
                AppEvent::LibraryListed(inventory) => {
                    self.status_widget.report(format!(
                        "{} models installed, {} used, {} verified, {} broken",
                        inventory.count(),
                        inventory.total_size(),
                        inventory.verified_count(),
                        inventory.broken_count(),
                    ));
                    self.library_table_widget.show(inventory);
                }
                AppEvent::LibraryListingFailed(err) => {
                    self.raise_failure(err);
                }
                AppEvent::ModelInspected(entry) => {
                    self.status_widget.report(Self::MSG_INSPECTED);
                    self.details = Some(entry);
                }
                AppEvent::ModelInspectionFailed(err) => {
                    self.raise_failure(err);
                }
                AppEvent::ModelVerified(entry) => {
                    self.status_widget.report(Self::verdict_of(&entry));
                    self.details = Some(entry);
                    self.library_manager.list();
                }
                AppEvent::ModelVerificationFailed(err) => {
                    self.raise_failure(err);
                }
                AppEvent::ModelRemoved(removed) => {
                    self.details = None;
                    self.status_widget.report(format!(
                        "Removed {}, reclaiming {}",
                        removed.spec(),
                        removed.reclaimed()
                    ));
                    self.library_manager.list();
                }
                AppEvent::ModelRemovalFailed(err) => {
                    self.raise_failure(err);
                }
                AppEvent::LibraryPruned(strays) => {
                    self.status_widget.report(format!(
                        "Pruned {} leftovers, reclaiming {}",
                        strays.len(),
                        DiscardedStray::total_reclaimed(&strays)
                    ));
                    self.library_manager.list();
                }
                AppEvent::LibraryPruningFailed(err) => {
                    self.raise_failure(err);
                }
                AppEvent::Quit => {
                    self.should_quit = true;
                }
            }
        }
    }

    /// Handle a key event based on current mode.
    pub async fn handle_key_event(&mut self, key: KeyEvent) {
        if self.last_error.is_some() {
            self.last_error = None;
            return;
        }

        if EventHandler::is_quit_key(&key) {
            self.event_sender.send(AppEvent::Quit).ok();
            return;
        }

        match key.code {
            KeyCode::Tab => return self.switch_to(self.mode.next()),
            KeyCode::BackTab => return self.switch_to(self.mode.previous()),
            _ => {}
        }

        match self.mode {
            AppMode::Search => self.handle_search_keys(key).await,
            AppMode::ModelTable => self.handle_model_table_keys(key).await,
            AppMode::InstallProgress => self.handle_install_progress_keys(key),
            AppMode::Library => self.handle_library_keys(key),
            AppMode::Help => self.handle_help_keys(key),
        }
    }

    /// Leaves the current mode for `mode`, preparing whatever it needs.
    ///
    /// Entering the library reads it when nothing has read it yet, so the
    /// operator never faces an empty screen they have to know to refresh.
    /// Leaving it abandons an unanswered removal prompt rather than carrying a
    /// pending deletion into a mode that cannot show it.
    fn switch_to(&mut self, mode: AppMode) {
        if self.mode == AppMode::Library && mode != AppMode::Library {
            self.details = None;
            self.pending_removal = None;
        }

        self.mode = mode;

        if mode == AppMode::Library && self.library_table_widget.inventory().is_none() {
            self.status_widget.report(Self::MSG_READING_LIBRARY);
            self.library_manager.list();
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
                    self.status_widget.report(Self::MSG_SEARCHING);
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
            KeyCode::Char('l') | KeyCode::Char('L') => {
                self.switch_to(AppMode::Library);
            }
            KeyCode::Esc => {
                self.switch_to(AppMode::Search);
            }
            KeyCode::Char('h') | KeyCode::Char('H') => {
                self.mode = AppMode::Help;
            }
            _ => {}
        }
    }

    /// Handles a key press while the operator manages installed models.
    ///
    /// An unanswered removal prompt takes every key: a deletion the operator
    /// has not confirmed must not be able to fall through to a shortcut that
    /// navigates away from it, leaving it armed.
    fn handle_library_keys(&mut self, key: KeyEvent) {
        if let Some(spec) = self.pending_removal.take() {
            return self.answer_removal_prompt(key, spec);
        }

        match key.code {
            KeyCode::Up => self.library_table_widget.previous(),
            KeyCode::Down => self.library_table_widget.next(),
            KeyCode::Enter | KeyCode::Char('i') | KeyCode::Char('I') => {
                self.inspect_selected_model();
            }
            KeyCode::Char('v') | KeyCode::Char('V') => self.verify_selected_model(),
            KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Delete => {
                self.propose_removal_of_selected_model();
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                self.status_widget.report(Self::MSG_PRUNING);
                self.library_manager.prune();
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.status_widget.report(Self::MSG_READING_LIBRARY);
                self.library_manager.list();
            }
            KeyCode::Char('h') | KeyCode::Char('H') => {
                self.mode = AppMode::Help;
            }
            KeyCode::Esc => self.close_details_or_leave_library(),
            _ => {}
        }
    }

    /// Closes an open details popup, or leaves the library when none is open.
    ///
    /// Escape means "back out of the innermost thing", so it dismisses the
    /// popup first and only surrenders the mode once there is nothing left
    /// inside it to dismiss.
    fn close_details_or_leave_library(&mut self) {
        if self.details.take().is_none() {
            self.switch_to(AppMode::ModelTable);
        }
    }

    fn inspect_selected_model(&mut self) {
        if let Some(spec) = self.selected_spec() {
            self.library_manager.inspect(spec);
        }
    }

    fn verify_selected_model(&mut self) {
        if let Some(spec) = self.selected_spec() {
            self.status_widget
                .report(format!("{}{spec}", Self::MSG_VERIFYING));
            self.library_manager.verify(spec);
        }
    }

    fn propose_removal_of_selected_model(&mut self) {
        if let Some(spec) = self.selected_spec() {
            self.details = None;
            self.pending_removal = Some(spec);
        }
    }

    fn answer_removal_prompt(&mut self, key: KeyEvent, spec: ModelSpec) {
        if matches!(key.code, KeyCode::Char('y') | KeyCode::Char('Y')) {
            self.status_widget
                .report(format!("{}{spec}", Self::MSG_REMOVING));
            self.library_manager.remove(spec);
        } else {
            self.status_widget.report(Self::MSG_REMOVAL_CANCELLED);
        }
    }

    fn selected_spec(&self) -> Option<ModelSpec> {
        self.library_table_widget
            .selected_entry()
            .map(|entry| entry.spec().clone())
    }

    fn raise_failure(&mut self, failure: String) {
        self.last_error = Some(failure.clone());
        self.status_widget.report_failure(failure);
    }

    fn verdict_of(entry: &ManagedModel) -> String {
        if entry.is_broken() {
            format!("{}{}", Self::MSG_VERDICT_BROKEN, entry.spec())
        } else if entry.is_verified() {
            format!("{}{}", Self::MSG_VERDICT_VERIFIED, entry.spec())
        } else {
            format!("{}{}", Self::MSG_VERDICT_UNPROVEN, entry.spec())
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

        if let Some(entry) = self.details.as_ref() {
            Self::draw_details_popup(frame, area, entry);
        }

        if let Some(spec) = self.pending_removal.as_ref() {
            Self::draw_removal_prompt(frame, area, spec);
        }

        if let Some(error) = self.last_error.as_ref() {
            Self::draw_error_popup(frame, area, error);
        }
    }

    fn draw_header(&mut self, frame: &mut Frame, area: Rect) {
        match self.mode {
            AppMode::Search => {
                self.search_widget.draw(frame, area);
            }
            AppMode::ModelTable => {
                Self::draw_banner(
                    frame,
                    area,
                    Self::MODEL_TABLE_HEADER,
                    Self::MODEL_TABLE_TITLE,
                    Color::Cyan,
                );
            }
            AppMode::InstallProgress => {
                Self::draw_banner(
                    frame,
                    area,
                    Self::INSTALL_PROGRESS_HEADER,
                    Self::INSTALL_PROGRESS_TITLE,
                    Color::Yellow,
                );
            }
            AppMode::Library => {
                Self::draw_banner(
                    frame,
                    area,
                    Self::LIBRARY_HEADER,
                    Self::LIBRARY_TITLE,
                    Color::Magenta,
                );
            }
            AppMode::Help => {
                Self::draw_banner(
                    frame,
                    area,
                    Self::HELP_HEADER,
                    Self::HELP_TITLE,
                    Color::Green,
                );
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
            AppMode::Library => {
                self.library_table_widget.draw(frame, area);
            }
            AppMode::Help => {
                self.help_widget.draw(frame, area);
            }
        }
    }

    fn draw_banner(
        frame: &mut Frame,
        area: Rect,
        header: &'static str,
        title: &'static str,
        color: Color,
    ) {
        let banner = Paragraph::new(header)
            .style(Style::default().fg(color))
            .block(Block::default().borders(Borders::ALL).title(title));
        frame.render_widget(banner, area);
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

    fn draw_details_popup(frame: &mut Frame, area: Rect, entry: &ManagedModel) {
        let popup_area = LayoutHelper::centered_rect(
            Self::DETAILS_POPUP_WIDTH_PCT,
            Self::DETAILS_POPUP_HEIGHT_PCT,
            area,
        );
        let color = if entry.is_broken() {
            Color::Red
        } else {
            Color::Cyan
        };
        let details = Paragraph::new(ModelDetails::describing(entry).to_lines().join("\n"))
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .title(Self::DETAILS_TITLE)
                    .borders(Borders::ALL)
                    .style(Style::default().bg(Color::Black).fg(color)),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(Clear, popup_area);
        frame.render_widget(details, popup_area);
    }

    fn draw_removal_prompt(frame: &mut Frame, area: Rect, spec: &ModelSpec) {
        let popup_area = LayoutHelper::centered_rect(
            Self::PROMPT_POPUP_WIDTH_PCT,
            Self::PROMPT_POPUP_HEIGHT_PCT,
            area,
        );
        let prompt = Paragraph::new(format!("{spec}\n\n{}", Self::PROMPT_ANSWERS))
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .title(Self::PROMPT_TITLE)
                    .borders(Borders::ALL)
                    .style(Style::default().bg(Color::Black).fg(Color::Yellow)),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(Clear, popup_area);
        frame.render_widget(prompt, popup_area);
    }

    fn draw_error_popup(frame: &mut Frame, area: Rect, error: &str) {
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
        "Models (↑/↓ navigate, Enter install, l library, Esc search, h help)";
    const MODEL_TABLE_TITLE: &'static str = "Models";

    const INSTALL_PROGRESS_HEADER: &'static str = "Installing Model... (Esc to return)";
    const INSTALL_PROGRESS_TITLE: &'static str = "Install Progress";

    const LIBRARY_HEADER: &'static str =
        "Library (↑/↓ navigate, i inspect, v verify, d delete, p prune, r reload, Esc back)";
    const LIBRARY_TITLE: &'static str = "Installed Models";

    const HELP_HEADER: &'static str = "Help (Esc/h to close)";
    const HELP_TITLE: &'static str = "Help";

    const SEARCH_HELP_TEXT: &'static str = "Enter search query and press Enter to search models.\nTab cycles search, models, library, help.\nEsc for help.";
    const SEARCH_HELP_TITLE: &'static str = "Search";

    const ERROR_TITLE: &'static str = "Error";
    const ERROR_POPUP_WIDTH_PCT: u16 = 60;
    const ERROR_POPUP_HEIGHT_PCT: u16 = 20;

    const DETAILS_TITLE: &'static str = "Installed Model (Esc to close)";
    const DETAILS_POPUP_WIDTH_PCT: u16 = 76;
    const DETAILS_POPUP_HEIGHT_PCT: u16 = 50;

    const PROMPT_TITLE: &'static str = "Delete this model?";
    const PROMPT_ANSWERS: &'static str = "y to delete, any other key to keep it";
    const PROMPT_POPUP_WIDTH_PCT: u16 = 60;
    const PROMPT_POPUP_HEIGHT_PCT: u16 = 25;

    const MSG_SEARCH_COMPLETED: &'static str =
        "Search completed. Use ↑/↓ to navigate, Enter to install.";
    const MSG_INSTALL_STARTED: &'static str = "Installing model...";
    const MSG_INSTALL_COMPLETED: &'static str = "Installed: ";
    const MSG_SEARCHING: &'static str = "Searching...";
    const MSG_READING_LIBRARY: &'static str = "Reading the installed models...";
    const MSG_INSPECTED: &'static str = "Esc closes the details.";
    const MSG_VERIFYING: &'static str = "Verifying ";
    const MSG_REMOVING: &'static str = "Removing ";
    const MSG_PRUNING: &'static str = "Pruning leftovers...";
    const MSG_REMOVAL_CANCELLED: &'static str = "Kept the model; nothing was deleted.";
    const MSG_VERDICT_VERIFIED: &'static str = "Verified: ";
    const MSG_VERDICT_UNPROVEN: &'static str = "No digest recorded, cannot prove: ";
    const MSG_VERDICT_BROKEN: &'static str = "BROKEN, bytes disagree with the digest: ";
}
