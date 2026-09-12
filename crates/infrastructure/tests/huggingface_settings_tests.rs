use localnar_domain::SettingValue;
use localnar_infrastructure::HuggingFaceSettings;

#[test]
fn present_fields_map_to_settings_and_absent_fields_are_omitted() {
    let settings =
        HuggingFaceSettings::new(Some("hf_abc".to_owned()), None, Some("/tmp/hf".to_owned()))
            .into_settings();

    assert_eq!(
        settings.get(&HuggingFaceSettings::api_token_key()),
        Some(&SettingValue::new("hf_abc"))
    );
    assert_eq!(settings.get(&HuggingFaceSettings::endpoint_key()), None);
    assert_eq!(
        settings.get(&HuggingFaceSettings::cache_directory_key()),
        Some(&SettingValue::new("/tmp/hf"))
    );
}

#[test]
fn an_entirely_absent_configuration_yields_no_settings() {
    let settings = HuggingFaceSettings::new(None, None, None).into_settings();

    assert!(settings.is_empty());
}

#[test]
fn from_settings_reads_back_the_values_into_settings_wrote() {
    let written = HuggingFaceSettings::new(
        Some("hf_abc".to_owned()),
        Some("https://example.test".to_owned()),
        None,
    )
    .into_settings();

    let read = HuggingFaceSettings::from_settings(&written);

    assert_eq!(read.api_token(), Some("hf_abc"));
    assert_eq!(read.endpoint(), Some("https://example.test"));
    assert_eq!(read.cache_directory(), None);
}
