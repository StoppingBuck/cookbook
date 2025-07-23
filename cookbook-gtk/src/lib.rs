use gtk::prelude::*;
// lib.rs for cookbook-gtk: exposes modules for integration tests
pub mod user_settings;
pub mod types;
pub mod ui_constants;
pub mod utils;
pub mod i18n;
pub mod dialogs;
pub mod kb;
pub mod pantry;
pub mod recipes;
pub mod settings;
pub mod sidebar;
pub mod tabs;

use types::{AppModel, AppWidgets, Tab};
use relm4::gtk;
use relm4::{ComponentSender, SimpleComponent};
use std::cell::Cell;
use std::rc::Rc;
use std::path::PathBuf;
use crate::user_settings::UserSettings;
use cookbook_engine::DataManager;
use ui_constants::*;

impl SimpleComponent for AppModel {
    type Init = ();
    type Input = crate::types::AppMsg;
    type Output = ();
    type Root = gtk::ApplicationWindow;
    type Widgets = AppWidgets;

    fn init_root() -> Self::Root {
        gtk::ApplicationWindow::builder()
            .title("Cookbook")
            .default_width(1024)
            .default_height(768)
            .build()
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        build_app_model_and_widgets(root, Some(sender))
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        // ...existing code from main.rs...
    }

    fn update_view(&self, widgets: &mut Self::Widgets, sender: ComponentSender<Self>) {
        // ...existing code from main.rs...
    }
}

/// Helper for tests: builds the AppModel and AppWidgets for UI testing
pub fn build_app_model_and_widgets(root: gtk::ApplicationWindow, sender: Option<ComponentSender<AppModel>>) -> relm4::ComponentParts<AppModel> {
    // Copy the logic from main.rs's AppModel::init here
    // ...existing code...
// == Begin AppModel::init logic ==
use dirs;
let mut data_dir = match std::env::var("COOKBOOK_DATA_DIR") {
    Ok(path) => PathBuf::from(path),
    Err(_) => {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.pop();
        path.push("example");
        path.push("data");
        path
    }
};
let config_path = dirs::config_dir()
    .unwrap_or_else(|| std::path::PathBuf::from("."))
    .join("cookbook-gtk/user_settings.toml");
let user_settings = UserSettings::load(&config_path);
if let Some(ref user_dir) = user_settings.data_dir {
    data_dir = PathBuf::from(user_dir);
}
let user_settings_rc = std::rc::Rc::new(std::cell::RefCell::new(user_settings));
let mut model = AppModel {
    data_manager: None,
    data_dir: data_dir.clone(),
    current_tab: Tab::Recipes,
    selected_recipe: None,
    selected_ingredient: None,
    selected_kb_entry: None,
    search_text: String::new(),
    show_about_dialog: false,
    show_help_dialog: false,
    selected_pantry_categories: Vec::new(),
    show_in_stock_only: false,
    error_message: None,
    refresh_category_popover: Cell::new(false),
    user_settings: user_settings_rc.clone(),
};
model.data_manager = match DataManager::new(&data_dir) {
    Ok(manager) => Some(Rc::new(manager)),
    Err(_) => None,
};
crate::i18n::set_language(&model.user_settings.borrow().language);
let main_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
let (sidebar, sidebar_buttons) = crate::sidebar::build_sidebar(sender.clone());
let main_stack = gtk::Stack::new();
main_stack.set_margin_top(DEFAULT_MARGIN);
main_stack.set_margin_bottom(DEFAULT_MARGIN);
main_stack.set_margin_end(DEFAULT_MARGIN);
main_stack.set_hexpand(true);
main_stack.set_vexpand(true);
let (recipes_container, recipes_list_box, recipes_details) = crate::recipes::build_recipes_tab(&model, sender.clone());
let (
    pantry_container,
    pantry_list_container,
    pantry_details_box,
    stock_filter_switch,
    pantry_title,
    refresh_categories,
) = crate::pantry::build_pantry_tab(&model, sender.clone());
let (kb_container, kb_list_box, kb_details, kb_label) = crate::kb::build_kb_tab(&model, sender.clone());
let settings_container = crate::settings::build_settings_tab(
    &model.user_settings.borrow().language,
    {
        let user_settings_rc = model.user_settings.clone();
        let sender = sender.clone();
        move |lang: String| {
            let mut user_settings = user_settings_rc.borrow_mut();
            user_settings.language = lang.clone();
            let config_path = dirs::config_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("cookbook-gtk/user_settings.toml");
            user_settings.save(&config_path);
            crate::i18n::set_language(&lang);
            if let Some(sender) = sender.as_ref() {
                sender.input(crate::types::AppMsg::ReloadPantry);
                sender.input(crate::types::AppMsg::ReloadRecipes);
                sender.input(crate::types::AppMsg::SwitchTab(Tab::Settings));
            }
        }
    },
    &model.data_dir.display().to_string(),
    {
        let user_settings_rc = model.user_settings.clone();
        let sender = sender.clone();
        move |new_data_dir: String| {
            let mut user_settings = user_settings_rc.borrow_mut();
            user_settings.data_dir = Some(new_data_dir.clone());
            let config_path = dirs::config_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("cookbook-gtk/user_settings.toml");
            user_settings.save(&config_path);
            crate::utils::validate_and_create_data_dir(&new_data_dir);
            if let Some(sender) = sender.as_ref() {
                sender.input(crate::types::AppMsg::ReloadPantry);
                sender.input(crate::types::AppMsg::ReloadRecipes);
                sender.input(crate::types::AppMsg::SwitchTab(Tab::Settings));
            }
        }
    },
);
main_stack.add_named(&recipes_container, Some("recipes"));
main_stack.add_named(&pantry_container, Some("pantry"));
main_stack.add_named(&kb_container, Some("kb"));
main_stack.add_named(&settings_container, Some("settings"));
main_stack.set_visible_child_name("recipes");
main_box.append(&sidebar);
let vert_separator = gtk::Separator::new(gtk::Orientation::Vertical);
main_box.append(&vert_separator);
main_box.append(&main_stack);
root.set_child(Some(&main_box));
let mut widgets = AppWidgets {
    window: root,
    main_stack,
    recipes_details,
    recipes_list_box,
    pantry_label: pantry_title.clone(),
    pantry_list: pantry_list_container,
    pantry_details: pantry_details_box,
    pantry_in_stock_switch: stock_filter_switch,
    kb_label: kb_label.clone(),
    kb_list_box,
    kb_details,
    sidebar_buttons,
    refresh_categories: None,
};
widgets.refresh_categories = refresh_categories;
relm4::ComponentParts { model, widgets }
}
