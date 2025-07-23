// Unit tests for ui_constants.rs
#[cfg(test)]
mod tests {
    use super::*;
    use cookbook_gtk::ui_constants::DEFAULT_MARGIN;
    #[test]
    fn test_default_margin() {
        assert!(DEFAULT_MARGIN > 0);
    }
}
