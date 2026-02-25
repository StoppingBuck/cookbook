// Unit tests for pantry.rs
// Note: The format module (pantry/format.rs) is created in Phase 4's pantry split.
// These tests will pass once Phase 4 creates the format module at
// cookbook_gtk::pantry::format::format_quantity.
#[cfg(test)]
mod tests {
    #[test]
    fn test_format_quantity_with_unit() {
        let result = cookbook_gtk::pantry::format::format_quantity(Some(2.0), "kg");
        assert_eq!(result, "2 kg");
    }

    #[test]
    fn test_format_quantity_without_unit() {
        let result = cookbook_gtk::pantry::format::format_quantity(Some(3.0), "");
        assert_eq!(result, "3");
    }

    #[test]
    fn test_format_quantity_none() {
        let result = cookbook_gtk::pantry::format::format_quantity(None, "kg");
        assert!(result.is_empty());
    }

    #[test]
    fn test_format_quantity_singular() {
        let result = cookbook_gtk::pantry::format::format_quantity(Some(1.0), "cup");
        assert_eq!(result, "1 cup");
    }
}
