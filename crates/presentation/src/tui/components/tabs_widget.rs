use std::sync::Arc;

use ratatui::{
    Frame,
    layout::Rect,
    symbols,
    widgets::{Block, Borders, Tabs},
};

use crate::tui::{
    app_tab::AppTab,
    components::themes::{GBadwolf, Theme},
};

/// Renders the tab strip that tells the operator which screen they are on.
///
/// The strip lists every tab on every frame and highlights the active one, so
/// switching screens stops being an invisible mode change.
#[derive(Clone)]
pub struct TabsWidget {
    theme: Arc<dyn Theme>,
}

impl TabsWidget {
    /// Create a new tab strip renderer with default theme.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new tab strip renderer with an injected theme.
    pub fn with_theme(theme: Arc<dyn Theme>) -> Self {
        Self { theme }
    }
    /// Draws every tab into `area`, highlighting `active`.
    pub fn draw(&self, frame: &mut Frame, area: Rect, active: AppTab) {
        let tabs = Tabs::new(AppTab::ALL.map(AppTab::title))
            .select(active.index())
            .style(self.theme.tab_inactive())
            .highlight_style(self.theme.tab_active())
            .divider(symbols::line::VERTICAL)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Self::TITLE)
                    .title_style(self.theme.title())
                    .border_style(self.theme.border())
                    .style(self.theme.content()),
            );

        frame.render_widget(tabs, area);
    }
}

impl Default for TabsWidget {
    fn default() -> Self {
        Self::with_theme(Arc::new(GBadwolf))
    }
}

impl TabsWidget {
    /// Title of the tab strip widget.
    pub const TITLE: &'static str = "LocalNar";
}
