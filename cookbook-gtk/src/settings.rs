use crate::i18n::tr;
use crate::user_settings::UserSettings;
use gtk::prelude::*;
use relm4::RelmWidgetExt;
use std::path::PathBuf;
use crate::ui_constants::*;

pub fn build_settings_tab(
    current_language: &str,
    on_language_change: impl Fn(String) + 'static,
) -> gtk::Box {
    let settings_container = gtk::Box::new(gtk::Orientation::Vertical, SECTION_SPACING);

    let settings_title = gtk::Label::new(Some(&tr("Settings")));
    settings_title.set_markup(&format!("<span size='x-large' weight='bold'>{}</span>", tr("Settings")));
    settings_title.set_halign(gtk::Align::Start);
    settings_title.set_margin_all(DEFAULT_MARGIN);

    // Language selector
    let lang_label = gtk::Label::new(Some(&tr("Language:")));
    lang_label.set_halign(gtk::Align::Start);
    lang_label.set_margin_start(DEFAULT_MARGIN);

    let lang_combo = gtk::ComboBoxText::new();
    lang_combo.append(Some("en"), "English");
    lang_combo.append(Some("de"), "Deutsch");
    lang_combo.append(Some("fr"), "Français");
    lang_combo.set_active_id(Some(current_language));
    let on_language_change = std::rc::Rc::new(on_language_change);
    let on_language_change_clone = on_language_change.clone();
    lang_combo.connect_changed(move |combo| {
        if let Some(lang) = combo.active_id() {
            on_language_change_clone(lang.to_string());
        }
    });

    let lang_box = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
    lang_box.append(&lang_label);
    lang_box.append(&lang_combo);

    settings_container.append(&settings_title);
    settings_container.append(&lang_box);

    settings_container
}