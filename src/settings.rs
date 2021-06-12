use crate::config;
use gio::prelude::*;

#[derive(Display, Debug, Clone, EnumString, PartialEq)]
#[strum(serialize_all = "kebab_case")]
pub enum Key {
    WindowWidth,
    WindowHeight,
    IsMaximized,
}

pub struct SettingsManager {}

impl SettingsManager {
    pub fn get_settings() -> gio::Settings {
        let app_id = config::APP_ID.trim_end_matches(".Devel");
        gio::Settings::new(app_id)
    }

    pub fn get_boolean(key: Key) -> bool {
        let settings = Self::get_settings();
        settings.get_boolean(&key.to_string())
    }

    pub fn set_boolean(key: Key, value: bool) {
        let settings = Self::get_settings();
        if let Err(err) = settings.set_boolean(&key.to_string(), value) {
            warn!("Failed to set {} to {} due to {}", key.to_string(), value, err);
        }
    }

    pub fn get_integer(key: Key) -> i32 {
        let settings = Self::get_settings();
        settings.get_int(&key.to_string())
    }

    pub fn set_integer(key: Key, value: i32) {
        let settings = Self::get_settings();
        if let Err(err) = settings.set_int(&key.to_string(), value) {
            warn!("Failed to set {} to {} due to {}", key.to_string(), value, err);
        }
    }
}
