use gtk::prelude::*;
// lib.rs for cookbook-gtk: exposes modules for integration tests
pub mod dialogs;
pub mod i18n;
pub mod kb;
pub mod pantry;
pub mod recipes;
pub mod settings;
pub mod sidebar;
pub mod tabs;
pub mod types;
pub mod ui_constants;
pub mod user_settings;
pub mod utils;

use crate::user_settings::UserSettings;
use cookbook_engine::DataManager;
use relm4::gtk;
use relm4::RelmWidgetExt;
use relm4::{ComponentSender, SimpleComponent};
use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;
use types::{AppModel, AppWidgets, Tab};
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
        // --- DEBUG: update_view entry ---
        println!(
            "DEBUG: update_view - Ingredient selection logic entered. selected_ingredient={:?}",
            self.selected_ingredient
        );
        // Pantry Details Pane update logic
        if let Some(selected_slug) = &self.selected_ingredient {
            println!(
                "DEBUG: update_view - Rebuilding details pane for slug={:?}",
                selected_slug
            );
            while let Some(child) = widgets.pantry_details.first_child() {
                widgets.pantry_details.remove(&child);
            }
            if let Some(ref dm) = self.data_manager {
                println!(
                    "DEBUG: update_view - Calling build_ingredient_detail_view for slug={:?}",
                    selected_slug
                );
                let detail_view = crate::pantry::build_ingredient_detail_view(
                    dm,
                    selected_slug,
                    &sender,
                    |_| crate::types::AppMsg::SwitchTab(crate::types::Tab::Pantry),
                    |_| crate::types::AppMsg::SelectKnowledgeBaseEntry(String::new()),
                    |_| crate::types::AppMsg::SelectRecipe(String::new()),
                    |_| crate::types::AppMsg::EditIngredient(String::new()),
                    |_| crate::types::AppMsg::DeleteIngredient(String::new()),
                );
                widgets.pantry_details.append(&detail_view);
                println!(
                    "DEBUG: update_view - Detail view appended for slug={:?}",
                    selected_slug
                );
            } else {
                println!("DEBUG: update_view - No DataManager available");
                let not_found_label = gtk::Label::new(Some("No DataManager available"));
                not_found_label.set_halign(gtk::Align::Center);
                not_found_label.set_valign(gtk::Align::Center);
                widgets.pantry_details.append(&not_found_label);
            }
        } else {
            // No ingredient selected, show placeholder
            println!("DEBUG: update_view - No ingredient selected, clearing selection");
            while let Some(child) = widgets.pantry_details.first_child() {
                widgets.pantry_details.remove(&child);
            }
            let select_label = gtk::Label::new(Some("Select an ingredient to view details"));
            select_label.set_halign(gtk::Align::Center);
            select_label.set_valign(gtk::Align::Center);
            select_label.set_hexpand(true);
            select_label.set_vexpand(true);
            widgets.pantry_details.append(&select_label);
        }
        // ...existing code...
    }
}

/// Helper for tests: builds the AppModel and AppWidgets for UI testing
pub fn build_app_model_and_widgets(
    root: gtk::ApplicationWindow,
    sender: Option<ComponentSender<AppModel>>,
) -> relm4::ComponentParts<AppModel> {
    // Copy the logic from main.rs's AppModel::init here
    // ...existing code...
    // == Begin AppModel::init logic ==
    let config_path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("cookbook-gtk/user_settings.toml");
    let user_settings = UserSettings::load(&config_path);
    // Set GTK theme BEFORE any widgets are created
    {
        use gtk::prelude::*;
        let gtk_settings = gtk::Settings::default().unwrap();
        match &user_settings.theme {
            crate::user_settings::Theme::Light => {
                #[allow(deprecated)]
                let _ = gtk_settings.set_property("gtk-application-prefer-dark-theme", &false);
                let _ = gtk_settings.set_property("gtk-theme-name", &"Adwaita");
            }
            crate::user_settings::Theme::Dark => {
                #[allow(deprecated)]
                let _ = gtk_settings.set_property("gtk-application-prefer-dark-theme", &true);
                let _ = gtk_settings.set_property("gtk-theme-name", &"Adwaita-dark");
            }
            crate::user_settings::Theme::System => {
                #[allow(deprecated)]
                let _ = gtk_settings.set_property("gtk-application-prefer-dark-theme", &false);
                let _ = gtk_settings.set_property("gtk-theme-name", &"Adwaita");
            }
        }
    }
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
        pantry_list_needs_rebuild: Cell::new(true),
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
    let (recipes_container, recipes_list_box, recipes_details) =
        crate::recipes::build_recipes_tab(&model, sender.clone());
    let (
        pantry_container,
        pantry_list_container,
        pantry_details_box,
        stock_filter_switch,
        pantry_title,
        refresh_categories,
    ) = crate::pantry::build_pantry_tab(&model, sender.clone());
    let (kb_container, kb_list_box, kb_details, kb_label) =
        crate::kb::build_kb_tab(&model, sender.clone());
    // ...existing code...
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
        match &model.user_settings.borrow().theme {
            crate::user_settings::Theme::System => "System",
            crate::user_settings::Theme::Light => "Light",
            crate::user_settings::Theme::Dark => "Dark",
        },
        {
            let user_settings_rc = model.user_settings.clone();
            let sender = sender.clone();
            move |theme: String| {
                // Set GTK theme immediately when changed
                use gtk::prelude::*;
                let gtk_settings = gtk::Settings::default().unwrap();
                match theme.as_str() {
                    "Light" => {
                        let _ =
                            gtk_settings.set_property("gtk-application-prefer-dark-theme", &false);
                    }
                    "Dark" => {
                        let _ =
                            gtk_settings.set_property("gtk-application-prefer-dark-theme", &true);
                    }
                    _ => {
                        let _ =
                            gtk_settings.set_property("gtk-application-prefer-dark-theme", &false);
                    }
                }
                // Try to force style refresh (if available)
                #[cfg(feature = "v4_6")] // Only available in gtk4 v4_6+
                {
                    if let Some(context) = gtk_settings.style_context() {
                        context.invalidate();
                    }
                }
                let mut user_settings = user_settings_rc.borrow_mut();
                user_settings.theme = match theme.as_str() {
                    "Light" => crate::user_settings::Theme::Light,
                    "Dark" => crate::user_settings::Theme::Dark,
                    _ => crate::user_settings::Theme::System,
                };
                let config_path = dirs::config_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from("."))
                    .join("cookbook-gtk/user_settings.toml");
                user_settings.save(&config_path);
                println!("[DEBUG] Theme changed to: {}", theme);
                if let Some(sender) = sender.as_ref() {
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
        pantry_row_map: std::collections::HashMap::new(), // slug → ListBoxRow
    };
    widgets.refresh_categories = refresh_categories;
    relm4::ComponentParts { model, widgets }
}

// Add debug breadcrumbs to build_ingredient_detail_view
#[allow(dead_code)]
pub fn build_ingredient_detail_view<C>(
    data_manager: &Rc<DataManager>,
    ingredient_id: &str, // now this is always a slug
    sender: &ComponentSender<C>,
    switch_tab_msg: impl Fn(crate::Tab) -> C::Input + Clone + 'static,
    select_kb_entry_msg: impl Fn(String) -> C::Input + Clone + 'static,
    select_recipe_msg: impl Fn(String) -> C::Input + Clone + 'static,
    edit_ingredient_msg: impl Fn(String) -> C::Input + Clone + 'static,
) -> gtk::Box
where
    C: relm4::Component,
{
    println!(
        "DEBUG: build_ingredient_detail_view - called with ingredient_id={:?}",
        ingredient_id
    );
    // Create a small details view
    let details_container = gtk::Box::new(gtk::Orientation::Vertical, SECTION_SPACING);
    details_container.set_margin_all(DEFAULT_MARGIN);
    let lang = data_manager
        .get_all_ingredients()
        .first()
        .and_then(|_| Some("en")) // fallback if needed
        .unwrap_or("en");
    // Try to resolve by slug or translation
    let ingredient = data_manager.find_ingredient_by_name_or_translation(ingredient_id, lang);
    println!("DEBUG: build_ingredient_detail_view - find_ingredient_by_name_or_translation returned: {:?}", ingredient);
    if let Some(ingredient) = ingredient {
        println!(
            "DEBUG: build_ingredient_detail_view - Found ingredient: {:?}",
            ingredient.name
        );
        // ...existing code...
    } else {
        println!(
            "DEBUG: build_ingredient_detail_view - Ingredient not found for id={:?}",
            ingredient_id
        );
        // ...existing code...
    }
    details_container
}
