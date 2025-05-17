use gtk::prelude::*;
use relm4::gtk;
use relm4::ComponentSender;

use crate::types::{AppModel, AppMsg, Tab};

/// Builds the sidebar UI for the application
pub fn build_sidebar(sender: &ComponentSender<AppModel>) -> (gtk::Box, Vec<gtk::Button>) {
    // Create the sidebar container
    let sidebar = gtk::Box::new(gtk::Orientation::Vertical, 0);
    sidebar.set_margin_top(10);
    sidebar.set_margin_bottom(10);
    sidebar.set_margin_start(10);
    sidebar.set_margin_end(10);
    sidebar.set_width_request(200);

    // Sidebar heading
    let sidebar_heading = gtk::Label::new(Some("Navigation"));
    sidebar_heading.set_halign(gtk::Align::Start);
    sidebar_heading.set_margin_bottom(10);
    sidebar.append(&sidebar_heading);

    // Create sidebar buttons and store them for styling
    let mut sidebar_buttons = vec![];

    // Create navigation buttons
    let recipes_button = gtk::Button::with_label("Recipes");
    let pantry_button = gtk::Button::with_label("Pantry");
    let kb_button = gtk::Button::with_label("Knowledge Base");
    let settings_button = gtk::Button::with_label("Settings");
    let about_button = gtk::Button::with_label("About");
    let help_button = gtk::Button::with_label("Help");

    // Connect button signals
    let sender_clone = sender.clone();
    recipes_button.connect_clicked(move |_| {
        sender_clone.input(AppMsg::SwitchTab(Tab::Recipes));
    });

    let sender_clone = sender.clone();
    pantry_button.connect_clicked(move |_| {
        sender_clone.input(AppMsg::SwitchTab(Tab::Pantry));
    });

    let sender_clone = sender.clone();
    kb_button.connect_clicked(move |_| {
        sender_clone.input(AppMsg::SwitchTab(Tab::KnowledgeBase));
    });

    let sender_clone = sender.clone();
    settings_button.connect_clicked(move |_| {
        sender_clone.input(AppMsg::SwitchTab(Tab::Settings));
    });

    let sender_clone = sender.clone();
    about_button.connect_clicked(move |_| {
        sender_clone.input(AppMsg::ShowAbout);
    });

    let sender_clone = sender.clone();
    help_button.connect_clicked(move |_| {
        sender_clone.input(AppMsg::ShowHelp);
    });

    // Add buttons to the sidebar
    sidebar.append(&recipes_button);
    sidebar.append(&pantry_button);
    sidebar.append(&kb_button);

    // Add a separator between main tabs and settings/help
    let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
    separator.set_margin_top(10);
    separator.set_margin_bottom(10);
    sidebar.append(&separator);

    sidebar.append(&settings_button);
    sidebar.append(&about_button);
    sidebar.append(&help_button);

    // Store the main navigation buttons for styling
    sidebar_buttons.push(recipes_button);
    sidebar_buttons.push(pantry_button);
    sidebar_buttons.push(kb_button);
    sidebar_buttons.push(settings_button);

    (sidebar, sidebar_buttons)
}

/// Updates sidebar button styles based on the current tab
pub fn update_sidebar_buttons(current_tab: &Tab, sidebar_buttons: &[gtk::Button]) {
    for (i, button) in sidebar_buttons.iter().enumerate() {
        let tab = match i {
            0 => Tab::Recipes,
            1 => Tab::Pantry,
            2 => Tab::KnowledgeBase,
            3 => Tab::Settings,
            _ => continue,
        };

        // Add or remove the "suggested-action" class to show the active tab
        if tab == *current_tab {
            button.add_css_class("suggested-action");
        } else {
            button.remove_css_class("suggested-action");
        }
    }
}