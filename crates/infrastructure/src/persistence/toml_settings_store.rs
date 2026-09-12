use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use localnar_application::{errors::SettingsStoreError, ports::outbound::SettingsStorePort};
use localnar_domain::{Setting, SettingKey, SettingValue, Settings};

/// Settings store backed by a single TOML file on disk.
///
/// Keys and values are persisted as a flat table of strings. A missing file is
/// treated as an empty settings set so a first run needs no bootstrapping.
#[derive(Debug, Clone)]
pub struct TomlSettingsStore {
    path: PathBuf,
}

impl TomlSettingsStore {
    /// Builds a store backed by the TOML file at `path`.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Resolves the default settings file under the user configuration directory.
    pub fn from_env() -> Self {
        Self::new(Self::default_path())
    }

    /// Returns the path of the backing TOML file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    fn default_path() -> PathBuf {
        Self::config_root().join("localnar").join("settings.toml")
    }

    fn config_root() -> PathBuf {
        std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| Self::home_dir().join(".config"))
    }

    fn home_dir() -> PathBuf {
        std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir())
    }

    fn parse(content: &str) -> Result<Settings, SettingsStoreError> {
        let table: BTreeMap<String, String> = toml::from_str(content)
            .map_err(|err| SettingsStoreError::Malformed(err.to_string()))?;
        let mut entries: Vec<Setting> = Vec::with_capacity(table.len());
        for (key, value) in table {
            let key = SettingKey::new(key)
                .map_err(|err| SettingsStoreError::Malformed(err.to_string()))?;
            entries.push(Setting::new(key, SettingValue::new(value)));
        }
        Ok(Settings::new(entries))
    }

    fn serialize(settings: &Settings) -> Result<String, SettingsStoreError> {
        let table: BTreeMap<String, String> = settings
            .iter()
            .map(|(key, value)| (key.as_str().to_owned(), value.as_str().to_owned()))
            .collect();
        toml::to_string(&table).map_err(|err| SettingsStoreError::Unwritable(err.to_string()))
    }
}

impl Default for TomlSettingsStore {
    fn default() -> Self {
        Self::from_env()
    }
}

impl SettingsStorePort for TomlSettingsStore {
    fn load(&self) -> Result<Settings, SettingsStoreError> {
        if !self.path.exists() {
            return Ok(Settings::default());
        }
        let content = fs::read_to_string(&self.path)
            .map_err(|err| SettingsStoreError::Unreadable(err.to_string()))?;
        Self::parse(&content)
    }

    fn save(&self, settings: &Settings) -> Result<(), SettingsStoreError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| SettingsStoreError::Unwritable(err.to_string()))?;
        }
        let content = Self::serialize(settings)?;
        fs::write(&self.path, content)
            .map_err(|err| SettingsStoreError::Unwritable(err.to_string()))
    }
}
