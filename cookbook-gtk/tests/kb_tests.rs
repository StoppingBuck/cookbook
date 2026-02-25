// Unit tests for kb.rs
#[cfg(test)]
mod tests {
    use cookbook_gtk::kb::markdown_to_pango;
    use gtk::prelude::*;

    fn init_gtk() {
        gtk::init().ok();
    }

    #[test]
    fn test_h1_heading() {
        init_gtk();
        let result = markdown_to_pango("# Title");
        assert!(result.contains("xx-large"), "h1 should use xx-large size");
        assert!(result.contains("Title"), "should contain heading text");
        assert!(
            result.contains("weight='bold'") || result.contains("weight=\"bold\""),
            "heading should be bold"
        );
    }

    #[test]
    fn test_h2_heading() {
        init_gtk();
        let result = markdown_to_pango("## Subtitle");
        assert!(result.contains("x-large"), "h2 should use x-large size");
        assert!(result.contains("Subtitle"));
    }

    #[test]
    fn test_h3_heading() {
        init_gtk();
        let result = markdown_to_pango("### Section");
        assert!(result.contains("large"), "h3 should use large size");
        assert!(result.contains("Section"));
    }

    #[test]
    fn test_bold_text() {
        init_gtk();
        let result = markdown_to_pango("**bold**");
        assert!(
            result.contains("<b>") && result.contains("</b>"),
            "bold text should be wrapped in <b>"
        );
        assert!(result.contains("bold"));
    }

    #[test]
    fn test_italic_text() {
        init_gtk();
        let result = markdown_to_pango("*italic*");
        assert!(
            result.contains("<i>") && result.contains("</i>"),
            "italic text should be wrapped in <i>"
        );
        assert!(result.contains("italic"));
    }

    #[test]
    fn test_list_item_star() {
        init_gtk();
        let result = markdown_to_pango("* item");
        assert!(result.contains("•"), "list items should use bullet point");
        assert!(result.contains("item"));
    }

    #[test]
    fn test_list_item_dash() {
        init_gtk();
        let result = markdown_to_pango("- item");
        assert!(
            result.contains("•"),
            "list items with dash should use bullet point"
        );
    }

    #[test]
    fn test_empty_string() {
        init_gtk();
        let result = markdown_to_pango("");
        assert!(
            result.is_empty() || result.trim().is_empty(),
            "empty input should produce empty output"
        );
    }

    #[test]
    fn test_plain_text() {
        init_gtk();
        let result = markdown_to_pango("Hello world");
        assert!(result.contains("Hello world"), "plain text should pass through");
    }

    #[test]
    fn test_multiline() {
        init_gtk();
        let result = markdown_to_pango("# Title\nPlain text\n**bold**");
        assert!(result.contains("Title"));
        assert!(result.contains("Plain text"));
        assert!(result.contains("<b>"));
    }
}
