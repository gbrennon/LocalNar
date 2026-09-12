use localnar_domain::{Setting, SettingValue, Settings};
use localnar_infrastructure::{DiskModelLibrary, EffectiveSettings, HuggingFaceSettings};

#[test]
fn absent_paths_resolve_to_non_blank_defaults() {
    let effective = EffectiveSettings::resolve(&Settings::default());

    assert!(
        effective
            .get(&DiskModelLibrary::download_directory_key())
            .is_some_and(|value| !value.as_str().is_empty())
    );
    assert!(
        effective
            .get(&HuggingFaceSettings::cache_directory_key())
            .is_some_and(|value| !value.as_str().is_empty())
    );
    assert!(
        effective
            .get(&HuggingFaceSettings::endpoint_key())
            .is_some_and(|value| !value.as_str().is_empty())
    );
}

#[test]
fn a_persisted_download_directory_overrides_the_default() {
    let persisted = Settings::new([Setting::new(
        DiskModelLibrary::download_directory_key(),
        SettingValue::new("/custom/models"),
    )]);

    let effective = EffectiveSettings::resolve(&persisted);

    assert_eq!(
        effective.get(&DiskModelLibrary::download_directory_key()),
        Some(&SettingValue::new("/custom/models"))
    );
}

#[test]
fn a_persisted_api_token_is_surfaced() {
    let persisted = Settings::new([Setting::new(
        HuggingFaceSettings::api_token_key(),
        SettingValue::new("hf_secret"),
    )]);

    let effective = EffectiveSettings::resolve(&persisted);

    assert_eq!(
        effective.get(&HuggingFaceSettings::api_token_key()),
        Some(&SettingValue::new("hf_secret"))
    );
}
