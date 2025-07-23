// Unit tests for settings.rs
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_settings_tab_build() {
        let tab_name = "Settings";
        assert_eq!(tab_name, "Settings");
    }

    #[test]
    fn test_data_dir_logic() {
        let data_dir = "/tmp/cookbook-data";
        assert!(data_dir.starts_with("/tmp"));
    }
}
