use std::sync::Arc;

use localnar_domain::{ManagedModel, ModelInventory};
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    widgets::{Block, Borders, Row, Table, TableState},
};

use crate::tui::components::{
    library_row::LibraryRow,
    themes::{GBadwolf, Theme},
};
/// Table of the models this machine holds, one row per installed replica.
///
/// The widget distinguishes a library that has not been read yet from one that
/// was read and found empty: the first is a question nobody asked yet, the
/// second is an answer the operator can act on, and showing them the same way
/// would tell an operator they have no models when nothing has looked.
///
/// The title carries the readings that belong to the library as a whole, so the
/// place, the count, the occupied space, and the number of broken replicas are
/// visible without walking the rows.
#[derive(Clone)]
pub struct LibraryTableWidget {
    inventory: Option<ModelInventory>,
    state: TableState,
    theme: Arc<dyn Theme>,
}

impl LibraryTableWidget {
    const UNREAD_TITLE: &'static str = "Library (not read yet)";
    const EMPTY_TITLE: &'static str = "Library (no models installed)";
    const BROKEN_TITLE_SUFFIX: &'static str = " BROKEN";
    const HIGHLIGHT_SYMBOL: &'static str = "> ";
    const REPOSITORY_MIN_WIDTH: u16 = 22;
    const FILE_MIN_WIDTH: u16 = 20;
    const STATE_WIDTH: u16 = 8;
    const SIZE_WIDTH: u16 = 10;
    const DIGEST_WIDTH: u16 = 12;
    const COLUMN_SPACING: u16 = 1;

    /// Builds an empty table with default theme.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds an empty table with an injected theme.
    pub fn with_theme(theme: Arc<dyn Theme>) -> Self {
        Self {
            inventory: None,
            state: TableState::default(),
            theme,
        }
    }

    /// Replaces the listing, selecting the first row of it.
    pub fn show(&mut self, inventory: ModelInventory) {
        self.state
            .select((!inventory.is_empty()).then_some(Self::FIRST_ROW));
        self.inventory = Some(inventory);
    }

    /// Forgets the listing, leaving the library unread again.
    pub fn clear(&mut self) {
        self.inventory = None;
        self.state.select(None);
    }

    /// The listing the table shows, once the library has been read.
    pub fn inventory(&self) -> Option<&ModelInventory> {
        self.inventory.as_ref()
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

    /// The installed model the selected row stands for.
    pub fn selected_entry(&self) -> Option<&ManagedModel> {
        let inventory = self.inventory.as_ref()?;
        inventory.entries().get(self.state.selected()?)
    }

    /// The number of rows the table holds.
    pub fn row_count(&self) -> usize {
        self.inventory
            .as_ref()
            .map(ModelInventory::count)
            .unwrap_or_default()
    }

    /// The heading the table renders above its rows.
    pub fn title(&self) -> String {
        let Some(inventory) = self.inventory.as_ref() else {
            return Self::UNREAD_TITLE.to_owned();
        };

        if inventory.is_empty() {
            return format!("{} {}", Self::EMPTY_TITLE, inventory.root().display());
        }

        let broken = inventory.broken_count();
        let alarm = if broken == 0 {
            String::new()
        } else {
            format!(" - {broken}{}", Self::BROKEN_TITLE_SUFFIX)
        };

        format!(
            "Library {} - {} models, {} used{alarm}",
            inventory.root().display(),
            inventory.count(),
            inventory.total_size(),
        )
    }

    /// Renders the table into `area`.
    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let title = self.title();
        let rows = self
            .inventory
            .as_ref()
            .map(ModelInventory::entries)
            .unwrap_or_default()
            .iter()
            .map(|entry| {
                let row = LibraryRow::describing(entry);
                let style = if row.is_broken() {
                    self.theme.status_error()
                } else if entry.is_verified() {
                    self.theme.status_success()
                } else {
                    self.theme.content()
                };
                Row::new(row.into_cells()).style(style)
            });

        let table = Table::new(rows, Self::COLUMN_WIDTHS)
            .header(Row::new(LibraryRow::HEADINGS).style(self.theme.content_emphasis()))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .title_style(self.theme.title())
                    .border_style(self.theme.border())
                    .style(self.theme.content()),
            )
            .column_spacing(Self::COLUMN_SPACING)
            .highlight_style(self.theme.highlight())
            .highlight_symbol(Self::HIGHLIGHT_SYMBOL);
        frame.render_stateful_widget(table, area, &mut self.state);
    }

    fn select_by(&mut self, next_row: impl Fn(usize, usize) -> usize) {
        let Some(last_row) = self.row_count().checked_sub(1) else {
            return;
        };

        let selected = match self.state.selected() {
            Some(selected) => next_row(selected.min(last_row), last_row),
            None => Self::FIRST_ROW,
        };

        self.state.select(Some(selected));
    }
}

impl Default for LibraryTableWidget {
    fn default() -> Self {
        Self::with_theme(Arc::new(GBadwolf))
    }
}

impl LibraryTableWidget {
    const FIRST_ROW: usize = 0;

    const COLUMN_WIDTHS: [Constraint; 5] = [
        Constraint::Min(Self::REPOSITORY_MIN_WIDTH),
        Constraint::Min(Self::FILE_MIN_WIDTH),
        Constraint::Length(Self::STATE_WIDTH),
        Constraint::Length(Self::SIZE_WIDTH),
        Constraint::Length(Self::DIGEST_WIDTH),
    ];
}
