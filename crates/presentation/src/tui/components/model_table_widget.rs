use localnar_domain::ModelInfo;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table, TableState},
};

use crate::tui::components::model_row::ModelRow;

/// Table of the models a search found, one row per model.
///
/// A catalog entry publishes many files, but the operator is choosing a model,
/// so the table holds one described model per entry and never one row per
/// published file.
#[derive(Debug, Default)]
pub struct ModelTableWidget {
    models: Vec<ModelInfo>,
    state: TableState,
}

impl ModelTableWidget {
    const TITLE: &'static str = "Models";
    const HIGHLIGHT_SYMBOL: &'static str = "> ";
    const NAME_MIN_WIDTH: u16 = 24;
    const QUANTIZATION_WIDTH: u16 = 8;
    const SIZE_WIDTH: u16 = 10;
    const PARAMETERS_WIDTH: u16 = 8;
    const CONTEXT_WIDTH: u16 = 8;
    const COLUMN_SPACING: u16 = 1;

    /// Builds an empty table with nothing selected.
    pub fn new() -> Self {
        Self::default()
    }

    /// Replaces the described models, selecting the first of them.
    pub fn show(&mut self, models: Vec<ModelInfo>) {
        self.state
            .select((!models.is_empty()).then_some(Self::FIRST_ROW));
        self.models = models;
    }

    /// Drops every row, leaving nothing selected.
    pub fn clear(&mut self) {
        self.models.clear();
        self.state.select(None);
    }

    /// Moves the selection one row down, wrapping past the last row.
    pub fn next(&mut self) {
        self.select_by(|selected, last| {
            if selected >= last {
                Self::FIRST_ROW
            } else {
                selected + 1
            }
        });
    }

    /// Moves the selection one row up, wrapping past the first row.
    pub fn previous(&mut self) {
        self.select_by(|selected, last| {
            if selected == Self::FIRST_ROW {
                last
            } else {
                selected - 1
            }
        });
    }

    /// The model the selected row stands for.
    pub fn selected_model(&self) -> Option<&ModelInfo> {
        self.state
            .selected()
            .and_then(|selected| self.models.get(selected))
    }

    /// The number of rows the table holds.
    pub fn row_count(&self) -> usize {
        self.models.len()
    }

    /// Renders the table into `area`.
    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let rows = self.models.iter().map(|info| {
            Row::new(ModelRow::describing(info).into_cells())
                .style(Style::default().fg(Color::White))
        });

        let table = Table::new(rows, Self::COLUMN_WIDTHS)
            .header(
                Row::new(ModelRow::HEADINGS).style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            )
            .block(Block::default().borders(Borders::ALL).title(Self::TITLE))
            .column_spacing(Self::COLUMN_SPACING)
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(Self::HIGHLIGHT_SYMBOL);

        frame.render_stateful_widget(table, area, &mut self.state);
    }

    fn select_by(&mut self, next_row: impl Fn(usize, usize) -> usize) {
        let Some(last_row) = self.models.len().checked_sub(1) else {
            return;
        };

        let selected = match self.state.selected() {
            Some(selected) => next_row(selected.min(last_row), last_row),
            None => Self::FIRST_ROW,
        };

        self.state.select(Some(selected));
    }
}

impl ModelTableWidget {
    const FIRST_ROW: usize = 0;

    const COLUMN_WIDTHS: [Constraint; 5] = [
        Constraint::Min(Self::NAME_MIN_WIDTH),
        Constraint::Length(Self::QUANTIZATION_WIDTH),
        Constraint::Length(Self::SIZE_WIDTH),
        Constraint::Length(Self::PARAMETERS_WIDTH),
        Constraint::Length(Self::CONTEXT_WIDTH),
    ];
}
