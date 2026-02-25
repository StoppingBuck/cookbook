// Unit tests for utils.rs
#[cfg(test)]
mod tests {
    #[test]
    fn test_validate_and_create_data_dir() {
        let data_dir = "/tmp/cookbook-data-test";
        cookbook_gtk::utils::validate_and_create_data_dir(data_dir);
        assert!(std::path::Path::new(data_dir).exists());
        // Cleanup
        let _ = std::fs::remove_dir_all(data_dir);
    }

    #[test]
    fn test_validate_and_create_data_dir_existing() {
        // Test with a directory that already exists
        let data_dir = "/tmp/cookbook-data-test-existing";
        std::fs::create_dir_all(data_dir).unwrap();
        // Should not panic or fail when dir already exists
        cookbook_gtk::utils::validate_and_create_data_dir(data_dir);
        assert!(std::path::Path::new(data_dir).exists());
        // Cleanup
        let _ = std::fs::remove_dir_all(data_dir);
    }
}
