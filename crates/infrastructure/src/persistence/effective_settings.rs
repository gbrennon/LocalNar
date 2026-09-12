use localnar_domain::{Setting, SettingValue, Settings};

use crate::{DiskModelLibrary, HfHubTokioTransport, HuggingFaceSettings};

/// Resolves the configuration values to present to the operator.
///
/// Every infrastructure-owned key is filled with its effective value: the
/// persisted setting when present, otherwise the built-in default the adapters
/// would use at runtime. The secret API token carries no default and is only
/// included when it was actually persisted.
pub struct EffectiveSettings;

impl EffectiveSettings {
    /// Produces a [`Settings`] where each infrastructure key resolves to the
    /// value the operator would experience, defaults included.
    pub fn resolve(persisted: &Settings) -> Settings {
        let library = DiskModelLibrary::from_settings(persisted);
        let transport = HfHubTokioTransport::from_settings(persisted);
        let huggingface = HuggingFaceSettings::from_settings(persisted);

        let mut entries: Vec<Setting> = Vec::new();
        Self::push_secret_token(&mut entries, huggingface.api_token());
        entries.push(Setting::new(
            HuggingFaceSettings::endpoint_key(),
            SettingValue::new(transport.endpoint()),
        ));
        entries.push(Setting::new(
            HuggingFaceSettings::cache_directory_key(),
            SettingValue::new(transport.staging_dir().to_string_lossy().into_owned()),
        ));
        entries.push(Setting::new(
            DiskModelLibrary::download_directory_key(),
            SettingValue::new(library.root().to_string_lossy().into_owned()),
        ));
        Settings::new(entries)
    }

    fn push_secret_token(entries: &mut Vec<Setting>, token: Option<&str>) {
        if let Some(token) = token {
            entries.push(Setting::new(
                HuggingFaceSettings::api_token_key(),
                SettingValue::new(token),
            ));
        }
    }
}
