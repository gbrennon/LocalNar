use localnar_domain::{Setting, SettingKey, SettingValue, Settings};

const API_TOKEN_KEY: &str = "huggingface.api_token";
const ENDPOINT_KEY: &str = "huggingface.endpoint";
const CACHE_DIRECTORY_KEY: &str = "huggingface.cache_directory";

/// The Hugging Face-specific configuration, owning its setting keys.
///
/// Each field is optional; only the fields that are present contribute a setting
/// to the produced domain [`Settings`].
pub struct HuggingFaceSettings {
    api_token: Option<String>,
    endpoint: Option<String>,
    cache_directory: Option<String>,
}

impl HuggingFaceSettings {
    /// Collects the raw Hugging Face configuration values.
    pub fn new(
        api_token: Option<String>,
        endpoint: Option<String>,
        cache_directory: Option<String>,
    ) -> Self {
        Self {
            api_token,
            endpoint,
            cache_directory,
        }
    }

    /// Reads the Hugging Face-owned keys out of a generic domain [`Settings`].
    ///
    /// Values that are absent or blank become `None`.
    pub fn from_settings(settings: &Settings) -> Self {
        Self {
            api_token: Self::read(settings, &Self::api_token_key()),
            endpoint: Self::read(settings, &Self::endpoint_key()),
            cache_directory: Self::read(settings, &Self::cache_directory_key()),
        }
    }

    /// The configured Hugging Face API token, if any.
    pub fn api_token(&self) -> Option<&str> {
        self.api_token.as_deref()
    }

    /// The configured Hugging Face endpoint, if any.
    pub fn endpoint(&self) -> Option<&str> {
        self.endpoint.as_deref()
    }

    /// The configured Hugging Face cache directory, if any.
    pub fn cache_directory(&self) -> Option<&str> {
        self.cache_directory.as_deref()
    }

    /// The domain key under which the Hugging Face API token is stored.
    pub fn api_token_key() -> SettingKey {
        SettingKey::new(API_TOKEN_KEY).expect("a static hugging face setting key is non-blank")
    }

    /// The domain key under which the Hugging Face endpoint is stored.
    pub fn endpoint_key() -> SettingKey {
        SettingKey::new(ENDPOINT_KEY).expect("a static hugging face setting key is non-blank")
    }

    /// The domain key under which the Hugging Face cache directory is stored.
    pub fn cache_directory_key() -> SettingKey {
        SettingKey::new(CACHE_DIRECTORY_KEY)
            .expect("a static hugging face setting key is non-blank")
    }

    /// Maps the present configuration values onto a generic domain [`Settings`].
    ///
    /// Absent fields emit no setting.
    pub fn into_settings(self) -> Settings {
        let mut entries: Vec<Setting> = Vec::new();
        Self::push_optional(&mut entries, Self::api_token_key(), self.api_token);
        Self::push_optional(&mut entries, Self::endpoint_key(), self.endpoint);
        Self::push_optional(
            &mut entries,
            Self::cache_directory_key(),
            self.cache_directory,
        );
        Settings::new(entries)
    }

    fn push_optional(entries: &mut Vec<Setting>, key: SettingKey, value: Option<String>) {
        if let Some(value) = value {
            entries.push(Setting::new(key, SettingValue::new(value)));
        }
    }

    fn read(settings: &Settings, key: &SettingKey) -> Option<String> {
        settings
            .get(key)
            .map(|value| value.as_str().to_owned())
            .filter(|value| !value.trim().is_empty())
    }
}
