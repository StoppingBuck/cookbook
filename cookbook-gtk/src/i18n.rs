//! i18n.rs - Internationalization utilities for Cookbook GTK
use gettextrs::{gettext, ngettext};
use once_cell::sync::Lazy;
use std::env;
use std::sync::Mutex;

static LOCALE: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new("en".to_string()));

/// Initialize i18n with the given language code (e.g. "en", "de")
pub fn set_language(lang: &str) {
    let locale = LOCALE.lock();
    match locale {
        Ok(mut locale) => {
            *locale = lang.to_string();
            // Set environment variable for gettext
            env::set_var("LANGUAGE", lang);
            gettextrs::setlocale(gettextrs::LocaleCategory::LcAll, lang);
            // Note: .mo files must be installed in the correct location for gettextrs to find them
        }
        Err(_) => {
            // Handle poisoned lock gracefully
            eprintln!("Failed to acquire LOCALE lock");
        }
    }
}

/// Get the current language code
pub fn get_language() -> String {
    match LOCALE.lock() {
        Ok(locale) => locale.clone(),
        Err(_) => "en".to_string(),
    }
}

/// Translate a string using the current catalog
pub fn tr(msgid: &str) -> String {
    gettext(msgid)
}

/// Translate with plural support
pub fn trn(msgid: &str, msgid_plural: &str, n: u64) -> String {
    ngettext(msgid, msgid_plural, n as u32)
}
