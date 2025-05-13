use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::System
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: Theme,
    pub font_scale: f32,
    pub data_dir: Option<String>,
    pub language: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            theme: Theme::default(),
            font_scale: 1.0,
            data_dir: None,
            language: "en".to_string(),
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        let config_path = Self::get_config_path();
        
        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => {
                    match serde_yaml::from_str(&content) {
                        Ok(config) => return config,
                        Err(e) => {
                            eprintln!("Error parsing config file: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading config file: {}", e);
                }
            }
        }
        
        // Return default if we couldn't load
        let default_config = AppConfig::default();
        default_config
    }
    
    pub fn save(&self) {
        let config_path = Self::get_config_path();
        
        // Ensure directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        
        match serde_yaml::to_string(self) {
            Ok(yaml) => {
                if let Err(e) = fs::write(&config_path, yaml) {
                    eprintln!("Failed to save config: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Failed to serialize config: {}", e);
            }
        }
    }
    
    fn get_config_path() -> PathBuf {
        let mut path = if let Some(config_dir) = dirs::config_dir() {
            config_dir
        } else {
            PathBuf::from(".")
        };
        
        path.push("cookbook");
        path.push("config.yaml");
        path
    }
}
