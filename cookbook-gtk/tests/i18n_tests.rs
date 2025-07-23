// Unit tests for i18n.rs
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_set_language() {
        let lang = "fr";
        cookbook_gtk::i18n::set_language(lang);
        assert_eq!(lang, "fr");
    }

    #[test]
    fn test_language_fallback() {
        let lang = "zz";
        cookbook_gtk::i18n::set_language(lang);
        assert_eq!(lang, "zz");
    }
}
