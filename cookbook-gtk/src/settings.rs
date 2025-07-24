use crate::i18n::tr;
use crate::ui_constants::*;
use gtk::prelude::*;
use relm4::RelmWidgetExt;

pub fn build_settings_tab(
    current_language: &str,
    on_language_change: impl Fn(String) + 'static,
    current_data_dir: &str,
    on_data_dir_change: impl Fn(String) + 'static,
    current_theme: &str,
    on_theme_change: impl Fn(String) + 'static,
) -> gtk::Box {
    let settings_container = gtk::Box::new(gtk::Orientation::Vertical, SECTION_SPACING);

    let settings_title = gtk::Label::new(Some(&tr("Settings")));
    settings_title.set_markup(&format!(
        "<span size='x-large' weight='bold'>{}</span>",
        tr("Settings")
    ));
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

    // Theme selector
    let theme_label = gtk::Label::new(Some(&tr("Theme:")));
    theme_label.set_halign(gtk::Align::Start);
    theme_label.set_margin_start(DEFAULT_MARGIN);

    let theme_combo = gtk::ComboBoxText::new();
    theme_combo.append(Some("System"), "System");
    theme_combo.append(Some("Light"), "Light");
    theme_combo.append(Some("Dark"), "Dark");
    theme_combo.set_active_id(Some(current_theme));
    let on_theme_change = std::rc::Rc::new(on_theme_change);
    let on_theme_change_clone = on_theme_change.clone();
    theme_combo.connect_changed(move |combo| {
        if let Some(theme) = combo.active_id() {
            on_theme_change_clone(theme.to_string());
        }
    });

    let theme_box = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
    theme_box.append(&theme_label);
    theme_box.append(&theme_combo);

    settings_container.append(&settings_title);
    settings_container.append(&lang_box);
    settings_container.append(&theme_box);

    // Data directory selector
    let data_dir_label = gtk::Label::new(Some(&tr("Data directory:")));
    data_dir_label.set_halign(gtk::Align::Start);
    data_dir_label.set_margin_start(DEFAULT_MARGIN);

    let data_dir_value = gtk::Label::new(Some(current_data_dir));
    data_dir_value.set_halign(gtk::Align::Start);
    data_dir_value.set_margin_start(DEFAULT_MARGIN);
    data_dir_value.set_selectable(true);

    let data_dir_button = gtk::Button::with_label(&tr("Change..."));
    let on_data_dir_change = std::rc::Rc::new(on_data_dir_change);
    let data_dir_value_clone = data_dir_value.clone();
    let on_data_dir_change_clone = on_data_dir_change.clone();
    data_dir_button.connect_clicked(move |_| {
        let dialog = gtk::FileChooserDialog::new(
            Some(&tr("Select Data Directory")),
            Some(&gtk::Window::default()),
            gtk::FileChooserAction::SelectFolder,
            &[
                (&tr("Cancel"), gtk::ResponseType::Cancel),
                (&tr("Select"), gtk::ResponseType::Accept),
            ],
        );
        let data_dir_value_clone2 = data_dir_value_clone.clone();
        let on_data_dir_change_clone2 = on_data_dir_change_clone.clone();
        dialog.connect_response(move |dialog, response| {
            if response == gtk::ResponseType::Accept {
                if let Some(folder) = dialog.file() {
                    if let Some(path) = folder.path() {
                        let path_str = path.to_string_lossy().to_string();
                        data_dir_value_clone2.set_text(&path_str);
                        on_data_dir_change_clone2(path_str);
                    }
                }
            }
            dialog.close();
        });
        dialog.show();
    });

    let data_dir_box = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
    data_dir_box.append(&data_dir_label);
    data_dir_box.append(&data_dir_value);
    data_dir_box.append(&data_dir_button);

    settings_container.append(&data_dir_box);

    settings_container
}
