// app.rs — real SimpleComponent impl for AppModel, plus build_app_model_and_widgets for tests.

use crate::user_settings::UserSettings;
use cookbook_engine::DataManager;
use gtk::prelude::*;
use relm4::gtk;
use relm4::gtk::glib;
use relm4::ComponentParts;
use relm4::ComponentSender;
use relm4::RelmWidgetExt;
use relm4::SimpleComponent;
use std::cell::Cell;
use std::env;
use std::path::PathBuf;
use std::rc::Rc;
use crate::types::{AppModel, AppMsg, AppWidgets, Tab};
use crate::ui_constants::*;

impl SimpleComponent for AppModel {
    type Init = ();
    type Input = AppMsg;
    type Output = ();
    type Root = gtk::ApplicationWindow;
    type Widgets = AppWidgets;

    // init_root creates the main application window
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
    ) -> ComponentParts<Self> {
        build_app_model_and_widgets(root, Some(sender))
    }

    // == UPDATE STARTS HERE ==
    // update handles incoming messages (e.g. switching tabs, selecting a recipe) and updates the model state
    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            // Message: User switches tabs
            AppMsg::SwitchTab(new_tab) => {
                self.current_tab = new_tab; // Update the current tab

                // Reset selection when changing tabs
                if self.current_tab == Tab::Recipes {
                    self.selected_recipe = None;
                }
                if self.current_tab == Tab::Pantry {
                    self.selected_ingredient = None;
                }
            }
            // Message: User clicks on the About button
            AppMsg::ShowAbout => {
                self.show_about_dialog = true;
            }
            // Message: User clicks on the Help button
            AppMsg::ShowHelp => {
                self.show_help_dialog = true;
            }
            // Message: User closes the About or Help dialog
            AppMsg::ResetDialogs => {
                // Reset all dialog flags
                self.show_about_dialog = false;
                self.show_help_dialog = false;
            }
            // Message: User selects a recipe
            AppMsg::SelectRecipe(recipe_name) => {
                self.selected_recipe = Some(recipe_name);
            }
            // Message: User selects an ingredient
            AppMsg::SelectIngredient(ingredient_name) => {
                self.selected_ingredient = Some(ingredient_name);
            }
            // Message: User selects a Knowledge Base entry
            AppMsg::SelectKnowledgeBaseEntry(slug) => {
                self.selected_kb_entry = Some(slug);
            }
            // Message: User toggles a pantry category filter
            AppMsg::TogglePantryCategory(category, is_selected) => {
                if is_selected && !self.selected_pantry_categories.contains(&category) {
                    self.selected_pantry_categories.push(category);
                    self.pantry_list_needs_rebuild.set(true);
                } else if !is_selected {
                    self.selected_pantry_categories.retain(|c| c != &category);
                    self.pantry_list_needs_rebuild.set(true);
                }
            }
            // Message: User toggles the in-stock filter
            AppMsg::ToggleInStockFilter(show_in_stock_only) => {
                self.show_in_stock_only = show_in_stock_only;
                self.pantry_list_needs_rebuild.set(true);
            }
            // Message: User types in the search bar
            AppMsg::SearchTextChanged(text) => {
                self.search_text = text;
                self.pantry_list_needs_rebuild.set(true);
            }
            // Message: User clicks on the Edit Ingredient button
            AppMsg::EditIngredient(ingredient_name) => {
                if let Some(ref data_manager) = self.data_manager {
                    if let Some(ingredient) = data_manager.get_ingredient(&ingredient_name) {
                        let pantry_item = data_manager.get_pantry_item(&ingredient_name);
                        crate::pantry::show_edit_ingredient_dialog(
                            &ingredient,
                            pantry_item,
                            self.data_manager.clone(),
                            sender.clone(),
                            ingredient_name.clone(),
                        );
                    }
                }
            }
            // Message: User clicks on the Edit Recipe button
            AppMsg::EditRecipe(recipe_title) => {
                if let Some(ref data_manager) = self.data_manager {
                    if let Some(recipe) = data_manager.get_recipe(&recipe_title) {
                        crate::recipes::show_edit_recipe_dialog(
                            &recipe,
                            self.data_manager.clone(),
                            sender.clone(),
                            recipe_title.clone(),
                        );
                    }
                }
            }
            // Message: User clicks the Add Ingredient button
            AppMsg::AddIngredient => {
                // Create a blank ingredient and no pantry item
                let blank_ingredient = cookbook_engine::Ingredient {
                    name: String::new(),
                    slug: String::new(),
                    category: String::new(),
                    kb: None,
                    tags: Some(Vec::new()),
                    translations: None,
                };
                crate::pantry::show_edit_ingredient_dialog(
                    &blank_ingredient,
                    None,
                    self.data_manager.clone(),
                    sender.clone(),
                    String::new(),
                );
            }
            // Message: User updates a recipe
            AppMsg::UpdateRecipe(original_title, new_recipe) => {
                // Use the engine's utility method for handling updates
                if let Some(old_data_manager) = &self.data_manager {
                    // Use the DataManager method that handles the update process
                    match DataManager::create_with_updated_recipe(
                        old_data_manager.get_data_dir(),
                        &original_title,
                        new_recipe.clone(),
                    ) {
                        Ok(updated_manager) => {
                            // Replace the old manager with our updated one
                            self.data_manager = Some(Rc::new(updated_manager));

                            // Update the selected recipe to the new title
                            let new_selected_title = new_recipe.title.clone();
                            self.selected_recipe = Some(new_selected_title);

                            // Force a full UI refresh by triggering a tab switch and back
                            // This ensures the recipe list is updated with any name changes
                            let sender_clone = sender.clone();
                            glib::spawn_future_local(async move {
                                // Switch to another tab and back to force a complete refresh
                                sender_clone.input(AppMsg::SwitchTab(Tab::Pantry));
                                sender_clone.input(AppMsg::SwitchTab(Tab::Recipes));
                            });
                        }
                        Err(err) => {
                            log::error!("Error updating recipe: {:?}", err);
                            let error_message = format!("Failed to update recipe: {}", err);
                            self.error_message = Some(error_message);
                        }
                    }
                }
            }
            // Message: User clicks the Add Recipe button
            AppMsg::AddRecipe => {
                crate::recipes::show_add_recipe_dialog(self.data_manager.clone(), sender.clone());
            }
            // Message: User clicks the Delete Recipe button
            AppMsg::DeleteRecipe(recipe_title) => {
                if let Some(ref data_manager) = self.data_manager {
                    let data_dir = data_manager.get_data_dir();
                    let recipes_dir = data_dir.join("recipes");
                    let file_name = format!("{}.md", recipe_title.replace(" ", "_"));
                    let recipe_path = recipes_dir.join(&file_name);
                    let result = std::fs::remove_file(&recipe_path);
                    match result {
                        Ok(_) => {
                            self.selected_recipe = None;
                            sender.input(AppMsg::ReloadRecipes);
                        }
                        Err(err) => {
                            let error_message = format!("Failed to delete recipe: {}", err);
                            self.error_message = Some(error_message);
                        }
                    }
                }
            }
            // Message: Explicitly reload recipes data and UI
            AppMsg::DeleteIngredient(ingredient_slug) => {
                if let Some(ref data_manager) = self.data_manager {
                    let data_dir = data_manager.get_data_dir();
                    let ingredients_dir = data_dir.join("ingredients");
                    let file_name = format!("{}.yaml", ingredient_slug);
                    let ingredient_path = ingredients_dir.join(&file_name);
                    let result = std::fs::remove_file(&ingredient_path);
                    match result {
                        Ok(_) => {
                            self.selected_ingredient = None;
                            sender.input(AppMsg::ReloadPantry);
                        }
                        Err(err) => {
                            let error_message = format!("Failed to delete ingredient: {}", err);
                            self.error_message = Some(error_message);
                        }
                    }
                }
            }
            AppMsg::ReloadRecipes => {
                if let Some(ref data_manager) = self.data_manager {
                    match cookbook_engine::DataManager::new(data_manager.get_data_dir()) {
                        Ok(updated_manager) => {
                            self.data_manager = Some(Rc::new(updated_manager));
                        }
                        Err(err) => {
                            let error_message = format!("Failed to reload recipes: {}", err);
                            self.error_message = Some(error_message);
                        }
                    }
                }
            }
            // Message: Explicitly reload pantry data and UI
            AppMsg::ReloadPantry => {
                if let Some(ref data_manager) = self.data_manager {
                    match cookbook_engine::DataManager::new(data_manager.get_data_dir()) {
                        Ok(updated_manager) => {
                            self.data_manager = Some(Rc::new(updated_manager));
                            // Do not attempt to rebuild pantry tab UI here; update_view will handle it
                            self.pantry_list_needs_rebuild.set(true);
                        }
                        Err(err) => {
                            let error_message = format!("Failed to reload pantry: {}", err);
                            self.error_message = Some(error_message);
                        }
                    }
                }
            }
            // Message: User creates a new recipe
            AppMsg::CreateRecipe(new_recipe) => {
                if let Some(ref data_manager) = self.data_manager {
                    let data_dir = data_manager.get_data_dir();
                    let recipes_dir = data_dir.join("recipes");
                    let file_name = format!("{}.md", new_recipe.title.replace(" ", "_"));
                    let recipe_path = recipes_dir.join(&file_name);
                    // Write the new recipe to file
                    match new_recipe.to_file(&recipe_path) {
                        Ok(_) => {
                            sender.input(AppMsg::ReloadRecipes);
                            self.selected_recipe = Some(new_recipe.title.clone());
                        }
                        Err(err) => {
                            let error_message = format!("Failed to create recipe: {}", err);
                            self.error_message = Some(error_message);
                        }
                    }
                }
            }
            // Message: User clicks the refresh button for category popover
            AppMsg::RefreshCategoryPopover => {
                log::debug!("Received RefreshCategoryPopover message");
                self.refresh_category_popover.set(true);
                // No model state to update, but force update_view to run
            }
            AppMsg::ClearError => {
                self.error_message = None;
            }
        }
    }
    // == UPDATE ENDS HERE ==

    // == UPDATE_VIEW STARTS HERE ==
    // update_view updates the UI based on the current model state
    fn update_view(&self, widgets: &mut Self::Widgets, sender: ComponentSender<Self>) {
        // Pantry category popover refresh logic
        if self.refresh_category_popover.get() {
            if let Some(ref refresh_fn) = widgets.refresh_categories {
                log::debug!("Calling refresh_categories closure from update_view");
                refresh_fn(self);
            } else {
                log::debug!("refresh_categories closure is None");
            }
            self.refresh_category_popover.set(false);
        }

        // Update the main stack to show the current tab
        crate::tabs::update_tab_view(&self.current_tab, &widgets.main_stack);

        // Update sidebar button styles based on the current tab
        crate::sidebar::update_sidebar_buttons(&self.current_tab, &widgets.sidebar_buttons);

        // Highlight the selected recipe in the Recipes list
        if self.current_tab == Tab::Recipes {
            if let Some(recipe_name) = self.selected_recipe.as_ref() {
                log::debug!(
                    "update_view - Recipe selection logic entered. selected_recipe={:?}",
                    recipe_name
                );
                let mut found = false;
                let mut i = 0;
                while let Some(row) = widgets.recipes_list_box.row_at_index(i) {
                    log::debug!("update_view - Checking recipe row {}", i);
                    i += 1;
                    // Try to find a label with the recipe name
                    let label_text = if let Some(child) = row.child() {
                        if let Some(label) = child.downcast_ref::<gtk::Label>() {
                            label.text().to_string()
                        } else if let Some(box_widget) = child.downcast_ref::<gtk::Box>() {
                            // If the row is a Box, try to get the first child label
                            if let Some(first_child) = box_widget.first_child() {
                                if let Some(label) = first_child.downcast_ref::<gtk::Label>() {
                                    label.text().to_string()
                                } else {
                                    String::new()
                                }
                            } else {
                                String::new()
                            }
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    };
                    log::debug!(
                        "update_view - Recipe row label_text={:?}",
                        label_text
                    );
                    if label_text == *recipe_name {
                        log::debug!(
                            "update_view - Found matching recipe row at index {}",
                            i - 1
                        );
                        let already_selected = widgets
                            .recipes_list_box
                            .selected_row()
                            .map(|selected| selected == row)
                            .unwrap_or(false);
                        if !already_selected {
                            widgets.recipes_list_box.select_row(Some(&row));
                        }
                        found = true;
                        break;
                    }
                }
                log::debug!("update_view - Recipe found={}", found);
                if !found && !self.search_text.is_empty() {
                    log::debug!("update_view - Recipe not found, clearing search_text");
                    let sender_clone = sender.clone();
                    glib::spawn_future_local(async move {
                        sender_clone.input(AppMsg::SearchTextChanged(String::new()));
                    });
                }
            } else {
                log::debug!("update_view - No recipe selected, clearing selection");
                widgets
                    .recipes_list_box
                    .select_row(None::<&gtk::ListBoxRow>);
            }
        }

        // Highlight the selected ingredient in the Pantry list
        if self.current_tab == Tab::Pantry {
            if let Some(selected_slug) = self.selected_ingredient.as_ref() {
                log::debug!("update_view - Ingredient selection logic entered. selected_ingredient(slug)={:?}", selected_slug);
                if let Some(row) = widgets.pantry_row_map.get(selected_slug) {
                    let already_selected = widgets
                        .pantry_list
                        .selected_row()
                        .map(|selected| selected == *row)
                        .unwrap_or(false);
                    if !already_selected {
                        widgets.pantry_list.select_row(Some(row));
                    }
                    log::debug!("update_view - Found matching pantry row for slug in map");
                } else {
                    log::debug!("update_view - Ingredient not found in pantry_row_map");
                    if !self.search_text.is_empty() {
                        let sender_clone = sender.clone();
                        glib::spawn_future_local(async move {
                            sender_clone.input(AppMsg::SearchTextChanged(String::new()));
                        });
                    }
                }
                // Update details pane for selected ingredient
                if let Some(data_manager) = &self.data_manager {
                    while let Some(child) = widgets.pantry_details.first_child() {
                        widgets.pantry_details.remove(&child);
                    }
                    let detail = crate::pantry::build_ingredient_detail_view(
                        data_manager,
                        selected_slug,
                        &sender,
                        move |_| AppMsg::SwitchTab(Tab::Pantry),
                        move |_| AppMsg::SelectKnowledgeBaseEntry(String::new()),
                        move |_| AppMsg::SelectRecipe(String::new()),
                        {
                            let slug = selected_slug.clone();
                            move |_| AppMsg::EditIngredient(slug.clone())
                        },
                        {
                            let slug = selected_slug.clone();
                            move |_| AppMsg::DeleteIngredient(slug.clone())
                        },
                    );
                    widgets.pantry_details.append(&detail);
                }
            } else {
                // Only clear selection if a row is actually selected
                if widgets.pantry_list.selected_row().is_some() {
                    log::debug!("update_view - No ingredient selected, clearing selection");
                    widgets.pantry_list.select_row(None::<&gtk::ListBoxRow>);
                } else {
                    log::debug!("update_view - No ingredient selected, but no row is selected, skipping selection clear");
                }
                // Show placeholder in details pane
                while let Some(child) = widgets.pantry_details.first_child() {
                    widgets.pantry_details.remove(&child);
                }
                let placeholder = gtk::Label::new(Some("Select an ingredient to view details"));
                widgets.pantry_details.append(&placeholder);
            }
        }

        // Update recipe details if a recipe is selected
        if self.current_tab == Tab::Recipes {
            crate::recipes::update_recipe_details(
                self.selected_recipe.as_deref(),
                &widgets.recipes_details,
                &self.data_manager,
                Some(&sender),
                AppMsg::EditRecipe,
            );
        }

        // Update KB entry details if a KB entry is selected
        if self.current_tab == Tab::KnowledgeBase {
            // Select the correct KB entry in the list box
            if let Some(slug) = &self.selected_kb_entry {
                crate::kb::update_kb_details::<AppModel>(&widgets.kb_details, &self.data_manager, slug);
            } else {
                crate::kb::show_kb_details_placeholder(&widgets.kb_details);
            }

            // Update the KB list if needed
            // Only do this when first switching to the tab to avoid unnecessary rebuilds
            if self.current_tab != Tab::KnowledgeBase {
                crate::kb::update_kb_list(
                    &widgets.kb_list_box,
                    &self.data_manager,
                    &sender,
                    AppMsg::SelectKnowledgeBaseEntry,
                );
            }
        }

        // Handle About dialog
        if self.show_about_dialog {
            crate::dialogs::show_about_dialog(&widgets.window, &sender, AppMsg::ResetDialogs);
        }

        // Handle Help dialog
        if self.show_help_dialog {
            crate::dialogs::show_help_dialog(&widgets.window, &sender, AppMsg::ResetDialogs);
        }

        // Only rebuild pantry list if flagged
        if self.current_tab == Tab::Pantry && self.pantry_list_needs_rebuild.get() {
            widgets.pantry_row_map.clear();
            crate::pantry::rebuild_pantry_list(
                &widgets.pantry_list,
                &self.search_text,
                &self.selected_pantry_categories,
                self.show_in_stock_only,
                |slug| AppMsg::SelectIngredient(slug),
                self,
                Some(sender.clone()),
                Some(&widgets.pantry_details),
                Some(&mut widgets.pantry_row_map),
            );
            self.pantry_list_needs_rebuild.set(false);
            self.pantry_list_needs_rebuild.set(false);
        }

        // Update recipes list and details when ReloadRecipes is triggered
        if self.current_tab == Tab::Recipes {
            crate::recipes::refresh_recipes_ui(self, widgets, &sender);
        } else {
            // Only update the recipes list if not already handled by refresh_recipes_ui
            crate::recipes::update_recipes_list(
                &widgets.recipes_list_box,
                &self.data_manager,
                &self.search_text,
                self.selected_recipe.as_ref(),
                Some(&sender),
                AppMsg::SelectRecipe,
            );
        }

        if let Some(ref msg) = self.error_message {
            crate::dialogs::show_error_dialog(&widgets.window, msg);
            // Clear the error after showing
            let sender_clone = sender.clone();
            glib::spawn_future_local(async move {
                sender_clone.input(AppMsg::ClearError);
            });
        }
    } // == UPDATE_VIEW ENDS HERE ==
}

/// Helper for tests: builds the AppModel and AppWidgets for UI testing
pub fn build_app_model_and_widgets(
    root: gtk::ApplicationWindow,
    sender: Option<ComponentSender<AppModel>>,
) -> relm4::ComponentParts<AppModel> {
    let config_path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("cookbook-gtk/user_settings.toml");
    let user_settings = UserSettings::load(&config_path);
    // Set GTK theme BEFORE any widgets are created
    {
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
    let mut data_dir = match env::var("COOKBOOK_DATA_DIR") {
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
                log::debug!("Theme changed to: {}", theme);
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
        pantry_row_map: std::collections::HashMap::new(),
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
    switch_tab_msg: impl Fn(crate::types::Tab) -> C::Input + Clone + 'static,
    select_kb_entry_msg: impl Fn(String) -> C::Input + Clone + 'static,
    select_recipe_msg: impl Fn(String) -> C::Input + Clone + 'static,
    edit_ingredient_msg: impl Fn(String) -> C::Input + Clone + 'static,
) -> gtk::Box
where
    C: relm4::Component,
{
    log::debug!(
        "build_ingredient_detail_view - called with ingredient_id={:?}",
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
    log::debug!("build_ingredient_detail_view - find_ingredient_by_name_or_translation returned: {:?}", ingredient);
    if let Some(ingredient) = ingredient {
        log::debug!(
            "build_ingredient_detail_view - Found ingredient: {:?}",
            ingredient.name
        );
    } else {
        log::debug!(
            "build_ingredient_detail_view - Ingredient not found for id={:?}",
            ingredient_id
        );
    }
    details_container
}
