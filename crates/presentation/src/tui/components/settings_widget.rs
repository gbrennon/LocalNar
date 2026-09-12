use std::sync::Arc;

use localnar_domain::{Setting, SettingKey, SettingValue, Settings};
use localnar_infrastructure::{DiskModelLibrary, HuggingFaceSettings};
use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::tui::components::themes::{GBadwolf, Theme};

struct SettingsField {
    label: &'static str,
    key: SettingKey,
    value: String,
    secret: bool,
}

/// Editable form over the operator's persisted settings.
///
/// Each field is bound to a domain [`SettingKey`]; the widget maps the whole
/// form to and from a domain [`Settings`] so persistence stays the caller's
/// concern. Secret fields are masked unless the field is being edited.
pub struct SettingsWidget {
    fields: Vec<SettingsField>,
    selected: usize,
    editing: bool,
    edit_backup: Option<String>,
    theme: Arc<dyn Theme>,
}

impl SettingsWidget {
    /// Builds the widget with the default theme.
    pub fn new() -> Self {
        Self::with_theme(Arc::new(GBadwolf))
    }

    /// Builds the widget over an injected theme.
    pub fn with_theme(theme: Arc<dyn Theme>) -> Self {
        Self {
            fields: Self::default_fields(),
            selected: 0,
            editing: false,
            edit_backup: None,
            theme,
        }
    }

    /// Replaces every field value with the one stored under its key.
    pub fn load_from(&mut self, settings: &Settings) {
        self.editing = false;
        self.edit_backup = None;
        for field in &mut self.fields {
            field.value = settings
                .get(&field.key)
                .map(|value| value.as_str().to_owned())
                .unwrap_or_default();
        }
    }

    /// Collects the non-blank fields into a domain [`Settings`].
    pub fn to_settings(&self) -> Settings {
        let entries = self.fields.iter().filter_map(|field| {
            let value = field.value.trim();
            if value.is_empty() {
                None
            } else {
                Some(Setting::new(field.key.clone(), SettingValue::new(value)))
            }
        });
        Settings::new(entries)
    }

    /// Reports whether a field is currently being edited.
    pub fn is_editing(&self) -> bool {
        self.editing
    }

    /// Reports the index of the highlighted field.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Moves the highlight to the previous field, wrapping at the top.
    pub fn previous(&mut self) {
        if self.editing {
            return;
        }
        let count = self.fields.len();
        self.selected = (self.selected + count - 1) % count;
    }

    /// Moves the highlight to the next field, wrapping at the bottom.
    pub fn next(&mut self) {
        if self.editing {
            return;
        }
        self.selected = (self.selected + 1) % self.fields.len();
    }

    /// Begins editing the highlighted field, remembering its prior value.
    pub fn begin_edit(&mut self) {
        self.edit_backup = Some(self.fields[self.selected].value.clone());
        self.editing = true;
    }

    /// Stops editing, keeping the typed value.
    pub fn commit_edit(&mut self) {
        self.editing = false;
        self.edit_backup = None;
    }

    /// Stops editing, restoring the value the field held before the edit.
    pub fn cancel_edit(&mut self) {
        if let Some(previous) = self.edit_backup.take() {
            self.fields[self.selected].value = previous;
        }
        self.editing = false;
    }

    /// Appends a character to the highlighted field while editing.
    pub fn input_char(&mut self, character: char) {
        if self.editing {
            self.fields[self.selected].value.push(character);
        }
    }

    /// Removes the last character of the highlighted field while editing.
    pub fn input_backspace(&mut self) {
        if self.editing {
            self.fields[self.selected].value.pop();
        }
    }

    /// Renders the settings form into `area`.
    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let mut lines: Vec<Line> = self
            .fields
            .iter()
            .enumerate()
            .map(|(index, field)| self.field_line(index, field))
            .collect();
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(Self::HINT, self.theme.content())));

        let paragraph = Paragraph::new(lines)
            .style(self.theme.content())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Self::TITLE)
                    .title_style(self.theme.title())
                    .border_style(self.theme.border())
                    .style(self.theme.content()),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    fn field_line(&self, index: usize, field: &SettingsField) -> Line<'static> {
        let is_selected = index == self.selected;
        let editing_this = is_selected && self.editing;
        let caret = if editing_this { "_" } else { "" };
        let text = format!(
            "{}: {}{}",
            field.label,
            Self::display_value(field, editing_this),
            caret
        );
        let style = if is_selected {
            self.theme.highlight()
        } else {
            self.theme.content()
        };
        Line::from(Span::styled(text, style))
    }

    fn display_value(field: &SettingsField, editing_this: bool) -> String {
        if field.secret && !editing_this {
            "*".repeat(field.value.chars().count())
        } else {
            field.value.clone()
        }
    }

    fn default_fields() -> Vec<SettingsField> {
        vec![
            SettingsField {
                label: "Hugging Face API Token",
                key: HuggingFaceSettings::api_token_key(),
                value: String::new(),
                secret: true,
            },
            SettingsField {
                label: "Hugging Face Endpoint",
                key: HuggingFaceSettings::endpoint_key(),
                value: String::new(),
                secret: false,
            },
            SettingsField {
                label: "Hugging Face Cache Directory",
                key: HuggingFaceSettings::cache_directory_key(),
                value: String::new(),
                secret: false,
            },
            SettingsField {
                label: "Model Download Path",
                key: DiskModelLibrary::download_directory_key(),
                value: String::new(),
                secret: false,
            },
        ]
    }
}

impl Default for SettingsWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsWidget {
    const TITLE: &'static str = "Settings";
    const HINT: &'static str = "↑/↓ select, Enter edit, Esc cancel edit, s save; Tab changes tab.";
}

#[cfg(test)]
mod settings_widget_tests {
    use localnar_infrastructure::{DiskModelLibrary, HuggingFaceSettings};

    use super::SettingsWidget;

    fn seeded() -> SettingsWidget {
        let mut widget = SettingsWidget::new();
        let mut store = localnar_domain::Settings::default();
        store = store.with(localnar_domain::Setting::new(
            HuggingFaceSettings::api_token_key(),
            localnar_domain::SettingValue::new("hf_token"),
        ));
        store = store.with(localnar_domain::Setting::new(
            DiskModelLibrary::download_directory_key(),
            localnar_domain::SettingValue::new("/models"),
        ));
        widget.load_from(&store);
        widget
    }

    #[test]
    fn load_from_populates_fields_and_to_settings_reads_them_back() {
        let widget = seeded();
        let settings = widget.to_settings();

        assert_eq!(
            settings
                .get(&HuggingFaceSettings::api_token_key())
                .map(|value| value.as_str()),
            Some("hf_token")
        );
        assert_eq!(
            settings
                .get(&DiskModelLibrary::download_directory_key())
                .map(|value| value.as_str()),
            Some("/models")
        );
    }

    #[test]
    fn blank_fields_are_omitted_from_the_produced_settings() {
        let widget = SettingsWidget::new();

        assert!(widget.to_settings().is_empty());
    }

    #[test]
    fn editing_a_field_updates_the_value_it_produces() {
        let mut widget = SettingsWidget::new();
        widget.begin_edit();
        for character in "hf_new".chars() {
            widget.input_char(character);
        }
        widget.commit_edit();

        let settings = widget.to_settings();
        assert_eq!(
            settings
                .get(&HuggingFaceSettings::api_token_key())
                .map(|value| value.as_str()),
            Some("hf_new")
        );
    }

    #[test]
    fn cancelling_an_edit_restores_the_prior_value() {
        let mut widget = seeded();
        widget.begin_edit();
        widget.input_backspace();
        widget.input_char('X');
        widget.cancel_edit();

        let settings = widget.to_settings();
        assert_eq!(
            settings
                .get(&HuggingFaceSettings::api_token_key())
                .map(|value| value.as_str()),
            Some("hf_token")
        );
    }

    #[test]
    fn navigation_moves_the_selection_and_wraps() {
        let mut widget = SettingsWidget::new();
        assert_eq!(widget.selected(), 0);

        widget.next();
        assert_eq!(widget.selected(), 1);

        widget.previous();
        widget.previous();
        assert_eq!(widget.selected(), 3);
    }
}
