use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct UserSettings {
    pub language: String,
}

impl UserSettings {
    pub fn load(path: &PathBuf) -> Self {
        if let Ok(data) = fs::read_to_string(path) {
            toml::from_str(&data).unwrap_or_default()
        } else {
            Self::default()
        }
    }
    pub fn save(&self, path: &PathBuf) {
        if let Ok(data) = toml::to_string(self) {
            let _ = fs::write(path, data);
        }
    }
}
