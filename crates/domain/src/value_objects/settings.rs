use std::collections::HashMap;

use crate::value_objects::{Setting, SettingKey, SettingValue};

/// An immutable keyed collection of configuration settings.
///
/// Later settings with the same key overwrite earlier ones (last-wins).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Settings(HashMap<SettingKey, SettingValue>);

impl Settings {
    /// Builds a collection from a sequence of settings; duplicate keys resolve last-wins.
    pub fn new(settings: impl IntoIterator<Item = Setting>) -> Self {
        let entries = settings
            .into_iter()
            .map(|setting| (setting.key().clone(), setting.value().clone()))
            .collect();
        Self(entries)
    }

    /// Returns the value stored under `key`, if any.
    pub fn get(&self, key: &SettingKey) -> Option<&SettingValue> {
        self.0.get(key)
    }

    /// Returns a new collection that also contains `setting`, overwriting any existing key.
    pub fn with(self, setting: Setting) -> Self {
        let mut entries = self.0;
        entries.insert(setting.key().clone(), setting.value().clone());
        Self(entries)
    }

    /// Reports whether the collection holds no settings.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Reports how many settings the collection holds.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Iterates over the contained key/value pairs in unspecified order.
    pub fn iter(&self) -> impl Iterator<Item = (&SettingKey, &SettingValue)> {
        self.0.iter()
    }
}

#[cfg(test)]
mod settings_tests {
    use crate::value_objects::{Setting, SettingKey, SettingValue, Settings};

    fn setting(key: &str, value: &str) -> Setting {
        Setting::new(
            SettingKey::new(key).expect("valid key"),
            SettingValue::new(value),
        )
    }

    #[test]
    fn an_empty_collection_reports_empty_and_finds_nothing() {
        let settings = Settings::default();
        let key = SettingKey::new("huggingface.api_token").expect("valid key");

        assert!(settings.is_empty());
        assert_eq!(settings.len(), 0);
        assert_eq!(settings.get(&key), None);
    }

    #[test]
    fn a_built_collection_resolves_known_keys_only() {
        let settings = setting("huggingface.api_token", "hf_abc");
        let settings = Settings::new(vec![settings]);
        let known = SettingKey::new("huggingface.api_token").expect("valid key");
        let unknown = SettingKey::new("huggingface.endpoint").expect("valid key");

        assert_eq!(settings.get(&known), Some(&SettingValue::new("hf_abc")));
        assert_eq!(settings.get(&unknown), None);
    }

    #[test]
    fn a_duplicate_key_resolves_last_wins() {
        let settings = Settings::new(vec![
            setting("huggingface.endpoint", "first"),
            setting("huggingface.endpoint", "second"),
        ]);
        let key = SettingKey::new("huggingface.endpoint").expect("valid key");

        assert_eq!(settings.get(&key), Some(&SettingValue::new("second")));
        assert_eq!(settings.len(), 1);
    }

    #[test]
    fn with_adds_an_entry_retrievable_by_get() {
        let key = SettingKey::new("huggingface.cache_directory").expect("valid key");
        let settings = Settings::default().with(setting("huggingface.cache_directory", "/tmp/hf"));

        assert_eq!(settings.get(&key), Some(&SettingValue::new("/tmp/hf")));
    }
}
