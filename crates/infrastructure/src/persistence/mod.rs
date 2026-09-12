//! Adapters that persist model artifacts to a local store.
//!
//! Each sub-module corresponds to a concrete persistence technology. The
//! current implementation stores models on the local filesystem.

pub mod disk;
pub mod effective_settings;
pub mod toml_settings_store;

pub use disk::model_library::DiskModelLibrary;
pub use effective_settings::EffectiveSettings;
pub use toml_settings_store::TomlSettingsStore;
