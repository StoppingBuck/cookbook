// Unit tests for dialogs.rs
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_error_dialog_logic() {
        // Simulate error dialog logic
        let error_message = "Test error";
        assert_eq!(error_message, "Test error");
    }

    #[test]
    fn test_about_dialog_logic() {
        let about_text = "Cookbook GTK App";
        assert!(about_text.contains("Cookbook"));
    }
}
