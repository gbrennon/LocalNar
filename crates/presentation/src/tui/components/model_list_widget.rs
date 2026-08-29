use domain::{ByteFormatter, RemoteModelFile};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

/// Model list widget displaying search results with navigation.
#[derive(Debug, Default)]
pub struct ModelListWidget {
    models: Vec<RemoteModelFile>,
    state: ListState,
}

impl ModelListWidget {
    /// Create a new model list widget.
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            models: Vec::new(),
            state,
        }
    }

    /// Set the list of models to display.
    pub fn set_models(&mut self, models: Vec<RemoteModelFile>) {
        self.models = models;
        if !self.models.is_empty() {
            self.state.select(Some(0));
        } else {
            self.state.select(None);
        }
    }

    /// Clear the model list.
    pub fn clear(&mut self) {
        self.models.clear();
        self.state.select(None);
    }

    /// Select the next model in the list.
    pub fn next(&mut self) {
        if self.models.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.models.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// Select the previous model in the list.
    pub fn previous(&mut self) {
        if self.models.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.models.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// Get the currently selected model.
    pub fn selected_model(&self) -> Option<&RemoteModelFile> {
        self.state.selected().and_then(|i| self.models.get(i))
    }

    /// Render the model list widget.
    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .models
            .iter()
            .enumerate()
            .map(|(i, model)| {
                let repo = model.repository().to_string();
                let file = model.file().to_string();
                let size = ByteFormatter::format(model.size().bytes());
                let checksum = model
                    .checksum()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                let content = Line::from(vec![
                    Span::styled(
                        format!("{:3}. ", i + 1),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(repo, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled(" / ", Style::default().fg(Color::DarkGray)),
                    Span::styled(file, Style::default().fg(Color::White)),
                    Span::styled("  [", Style::default().fg(Color::DarkGray)),
                    Span::styled(size, Style::default().fg(Color::Green)),
                    Span::styled("] [", Style::default().fg(Color::DarkGray)),
                    Span::styled(checksum, Style::default().fg(Color::Yellow)),
                    Span::styled("]", Style::default().fg(Color::DarkGray)),
                ]);
                ListItem::new(content)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(Self::TITLE))
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(Self::HIGHLIGHT_SYMBOL);

        frame.render_stateful_widget(list, area, &mut self.state);
    }
}

impl ModelListWidget {
    const TITLE: &'static str = "Models";
    const HIGHLIGHT_SYMBOL: &'static str = "▶ ";
}