use localnar_application::ports::outbound::SettingsStorePort;
use localnar_domain::{Setting, SettingKey, SettingValue, Settings};
use localnar_infrastructure::TomlSettingsStore;
use tempfile::TempDir;

fn setting(key: &str, value: &str) -> Setting {
    Setting::new(
        SettingKey::new(key).expect("valid key"),
        SettingValue::new(value),
    )
}

#[test]
fn loading_an_absent_store_yields_empty_settings() {
    let dir = TempDir::new().expect("temp dir");
    let store = TomlSettingsStore::new(dir.path().join("settings.toml"));

    let settings = store.load().expect("load");

    assert!(settings.is_empty());
}

#[test]
fn saved_settings_round_trip_through_the_store() {
    let dir = TempDir::new().expect("temp dir");
    let store = TomlSettingsStore::new(dir.path().join("nested").join("settings.toml"));
    let saved = Settings::new(vec![
        setting("huggingface.api_token", "hf_secret"),
        setting("huggingface.endpoint", "https://example.test"),
        setting("library.download_directory", "/models"),
    ]);

    store.save(&saved).expect("save");
    let loaded = store.load().expect("load");

    assert_eq!(loaded, saved);
}
