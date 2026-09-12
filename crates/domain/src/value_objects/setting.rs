use crate::value_objects::{SettingKey, SettingValue};

/// A single configuration setting: a key paired with its value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Setting {
    key: SettingKey,
    value: SettingValue,
}

impl Setting {
    /// Pairs a key with a value.
    pub fn new(key: SettingKey, value: SettingValue) -> Self {
        Self { key, value }
    }

    /// Borrows the setting's key.
    pub fn key(&self) -> &SettingKey {
        &self.key
    }

    /// Borrows the setting's value.
    pub fn value(&self) -> &SettingValue {
        &self.value
    }
}

#[cfg(test)]
mod setting_tests {
    use crate::value_objects::{Setting, SettingKey, SettingValue};

    #[test]
    fn a_setting_exposes_the_key_and_value_it_was_built_from() {
        let key = SettingKey::new("huggingface.endpoint").expect("valid key");
        let value = SettingValue::new("https://huggingface.co");
        let setting = Setting::new(key.clone(), value.clone());

        assert_eq!(setting.key(), &key);
        assert_eq!(setting.value(), &value);
    }
}
