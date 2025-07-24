use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSettings {
    pub language: String,
    pub data_dir: Option<String>,
    pub theme: Theme,
}

impl Default for UserSettings {
    fn default() -> Self {
        UserSettings {
            language: "en".to_string(),
            data_dir: None,
            theme: Theme::System,
        }
    }
}

impl UserSettings {
    pub fn load(path: &PathBuf) -> Self {
        println!("[DEBUG] Loading UserSettings from: {}", path.display());
        if let Ok(data) = fs::read_to_string(path) {
            let settings: Self = toml::from_str(&data).unwrap_or_default();
            println!("[DEBUG] Loaded UserSettings: theme={:?}", settings.theme);
            settings
        } else {
            println!(
                "[DEBUG] No settings file found at {}, using default theme.",
                path.display()
            );
            Self::default()
        }
    }
    pub fn save(&self, path: &PathBuf) {
        println!("[DEBUG] Saving UserSettings to: {}", path.display());
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(data) = toml::to_string(self) {
            println!("[DEBUG] Saving UserSettings: theme={:?}", self.theme);
            let _ = fs::write(path, data);
        }
    }
}
