// Unit tests for recipes.rs
#[cfg(test)]
mod tests {
    use cookbook_engine::DataManager;
    use std::path::PathBuf;

    fn fixture_data_dir() -> PathBuf {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest_dir.parent().unwrap().join("example/data")
    }

    #[test]
    fn test_data_manager_loads_recipes() {
        // Verify that the data manager correctly loads recipes
        // This is a pure engine test, not GTK
        let dm = DataManager::new(fixture_data_dir()).unwrap();
        assert_eq!(dm.get_all_recipes().len(), 2);
    }

    #[test]
    fn test_search_recipes_returns_results() {
        let dm = DataManager::new(fixture_data_dir()).unwrap();
        let results = dm.search_recipes("Lasagna");
        assert!(!results.is_empty());
        assert_eq!(results[0].title, "Lasagna");
    }
}
