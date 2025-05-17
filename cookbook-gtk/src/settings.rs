use gtk::prelude::*;
use relm4::RelmWidgetExt;

use crate::ui_constants::*;

pub fn build_settings_tab() -> gtk::Box {
    let settings_container = gtk::Box::new(gtk::Orientation::Vertical, SECTION_SPACING);

    let settings_title = gtk::Label::new(Some("Settings"));
    settings_title.set_markup("<span size='x-large' weight='bold'>Settings</span>");
    settings_title.set_halign(gtk::Align::Start);
    settings_title.set_margin_all(DEFAULT_MARGIN);

    let settings_label =
        gtk::Label::new(Some("Settings will be implemented in a future version."));
    settings_label.set_halign(gtk::Align::Start);
    settings_label.set_margin_start(DEFAULT_MARGIN);

    settings_container.append(&settings_title);
    settings_container.append(&settings_label);

    settings_container
}