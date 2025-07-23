// Unit tests for user_settings.rs
#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use cookbook_gtk::user_settings::UserSettings;

    #[test]
    fn test_user_settings_load_default() {
        // Should load default settings if file does not exist
        let tmp_path = PathBuf::from("/tmp/nonexistent_user_settings.toml");
        let settings = UserSettings::load(&tmp_path);
        assert!(settings.language.is_empty() || settings.language == "en");
    }

    #[test]
    fn test_user_settings_load_custom_dir() {
        // Save a config file and load it
        let tmp_path = PathBuf::from("/tmp/test_user_settings.toml");
        let mut settings = UserSettings::default();
        settings.language = "fr".to_string();
        settings.data_dir = Some("/tmp/cookbook-data".to_string());
        settings.save(&tmp_path);
        let loaded = UserSettings::load(&tmp_path);
        assert_eq!(loaded.language, "fr");
        assert_eq!(loaded.data_dir, Some("/tmp/cookbook-data".to_string()));
        // Cleanup
        let _ = fs::remove_file(&tmp_path);
    }
}
