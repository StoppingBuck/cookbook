// main.rs is the entry point for the GTK application
// It initializes the GTK application, sets up the main window, and handles user interactions
// The application is built using the relm4 library, which provides a way to create GTK applications in Rust

mod dialogs;
mod i18n;
mod kb;
mod pantry;
mod recipes;
mod settings;
mod sidebar;
mod tabs;
mod types;
mod ui_constants;
mod user_settings;
mod utils;

// First, we import the necessary libraries and modules
// The gtk::prelude::* import brings in a collection of traits from the GTK library, which are essential for working with GTK widgets and their associated methods. This simplifies the usage of GTK by allowing you to call methods directly on widgets without needing to explicitly import each trait.
// The cookbook_engine::DataManager import brings in the DataManager type from the cookbook_engine crate. This is the brain of the application, responsible for managing data such as recipes, pantry items, and ingredients. It acts as the bridge between the GUI and the underlying business logic.
// std::path::PathBuf is imported to handle file paths in a platform-independent manner. This is useful for managing the application's data directory or accessing specific files like recipes or pantry data.
// std::env is used to interact with environment variables, which can help in configuring the application (e.g., setting the data directory).
// std::rc::Rc is a reference-counted smart pointer that allows multiple parts of the application to share ownership of data, such as widgets or shared state, without requiring mutable references.
// ComponentParts and ComponentSender are used to define and manage the components of the application, including their models (state) and widgets (UI elements).
// SimpleComponent is a trait that simplifies the implementation of UI components, making it easier to define how the application reacts to user interactions.
// RelmApp is the application runner provided by relm4, which initializes and runs the main event loop of the GUI application.
// RelmWidgetExt provides extension traits for widgets, adding convenience methods that streamline widget manipulation and interaction.
// Together, these imports set up the foundation for building a GTK-based GUI application that leverages the cookbook_engine library for its core functionality and uses relm4 to manage the application's reactive components and event-driven architecture.
use crate::user_settings::UserSettings; // Import UserSettings to resolve undeclared type error // Import extension traits for widgets
use cookbook_engine::DataManager; // Import the DataManager from the cookbook_engine module
use gtk::prelude::*; // Import GTK traits for easier usage
use relm4::gtk; // Import GTK bindings from relm4
use relm4::gtk::glib; // Import glib for async operations
use relm4::ComponentParts; // Import to create component parts with model and widgets
use relm4::ComponentSender; // Import to send messages between components
use relm4::RelmApp; // Import application runner for relm4
use relm4::SimpleComponent;
use std::cell::Cell; // Import Cell for interior mutability
use std::env; // Import env for accessing environment variables
use std::path::PathBuf; // Import PathBuf for handling file paths
use std::rc::Rc; // Import Rc for reference counting
use types::{AppModel, AppMsg, AppWidgets, Tab};
use ui_constants::*; // Import trait for implementing UI components

// Implement the SimpleComponent trait for the AppModel
// This trait defines how the component is initialized, updated, and rendered
// It also defines how to handle messages and update the view
// The SimpleComponent trait is part of the relm4 library, which provides a way to build GTK applications in Rust
impl SimpleComponent for AppModel {
    type Init = ();
    type Input = AppMsg;
    type Output = ();
    type Root = gtk::ApplicationWindow;
    type Widgets = AppWidgets;

    // init_root creates the main application window
    // (if you are using rust-analyzer: the grey 'ApplicationWindowBuilder' is a hint that shows what each method returns)
    fn init_root() -> Self::Root {
        gtk::ApplicationWindow::builder()
            .title("Cookbook")
            .default_width(1024)
            .default_height(768)
            .build()
    }

    // init is a key part of the relm4 framework's component lifecycle. It is responsible for initializing a component in the application. In relm4, a component represents a self-contained part of the user interface, including its state (model) and associated widgets.
    // The function takes three parameters:
    // 1. _: Self::Init: This parameter represents the initialization data for the component. The underscore (_) indicates that this parameter is unused in this implementation, but it could be used in other cases to pass initial state or configuration to the component.
    // 2. root: Self::Root: This is the root widget of the component, typically a GTK container widget (e.g., gtk::Box or gtk::Window). It serves as the entry point for building the component's user interface.
    // 3. sender: ComponentSender<Self>: This is a communication channel used to send messages to the component. Messages are a core part of relm4's reactive architecture, allowing the component to respond to user interactions or other events.
    // The function returns a ComponentParts<Self>. This is a struct provided by relm4 that bundles together the component's model (state) and its widgets. It ensures that the component's state and UI are properly initialized and connected.
    // In summary, this function is the starting point for defining how a relm4 component is initialized, linking its state, UI, and message-handling logic. It plays a crucial role in ensuring that the component is ready to interact with the rest of the application.
    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // Get default data directory, or from user settings if set
        let mut data_dir = match env::var("COOKBOOK_DATA_DIR") {
            // Get the data directory from environment variable
            Ok(path) => PathBuf::from(path), // If the environment variable is set, use it
            Err(_) => {
                // If not set, use the example data folder as default
                let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR")); // Get the current directory
                path.pop(); // Go up one level from cookbook-gtk
                path.push("example"); // Go into the example folder
                path.push("data"); // Go into the data folder
                path // Return the path
            }
        };
        // Load user settings from config file
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

        // Load data using the DataManager
        model.data_manager = match DataManager::new(&data_dir) {
            Ok(manager) => {
                println!("Data loaded successfully from: {}", data_dir.display());
                Some(Rc::new(manager))
            }
            Err(e) => {
                eprintln!("Error loading data: {}", e);
                None
            }
        };

        // Set language for i18n at startup
        crate::i18n::set_language(&model.user_settings.borrow().language);

        // Here comes all the UI code

        // Create the main layout (sidebar + main content area)
        let main_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        // Create sidebar
        let (sidebar, sidebar_buttons) = sidebar::build_sidebar(Some(sender.clone()));

        // Create the main stack for switching between tab content
        let main_stack = gtk::Stack::new();
        main_stack.set_margin_top(DEFAULT_MARGIN);
        main_stack.set_margin_bottom(DEFAULT_MARGIN);
        main_stack.set_margin_end(DEFAULT_MARGIN);
        main_stack.set_hexpand(true);
        main_stack.set_vexpand(true);

        // Create tab content containers

        // Recipes tab content
        let (recipes_container, recipes_list_box, recipes_details) =
            recipes::build_recipes_tab(&model, Some(sender.clone()));

        // Pantry tab content
        let (
            pantry_container,
            pantry_list_container,
            pantry_details_box,
            stock_filter_switch,
            pantry_title,
            refresh_categories,
        ) = pantry::build_pantry_tab(&model, Some(sender.clone()));

        // Knowledge base tab content
        let (kb_container, kb_list_box, kb_details, kb_label) =
            kb::build_kb_tab(&model, Some(sender.clone()));

        // Settings tab content
        let settings_container = settings::build_settings_tab(
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
                    println!(
                        "[DEBUG] UI refresh triggered after language change to: {}",
                        lang
                    );
                    sender.input(AppMsg::ReloadPantry);
                    sender.input(AppMsg::ReloadRecipes);
                    sender.input(AppMsg::SwitchTab(Tab::Settings));
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
                    sender.input(AppMsg::ReloadPantry);
                    sender.input(AppMsg::ReloadRecipes);
                    sender.input(AppMsg::SwitchTab(Tab::Settings));
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
                    sender.input(AppMsg::SwitchTab(Tab::Settings));
                }
            },
        );

        // Add tab content to stack
        main_stack.add_named(&recipes_container, Some("recipes"));
        main_stack.add_named(&pantry_container, Some("pantry"));
        main_stack.add_named(&kb_container, Some("kb"));
        main_stack.add_named(&settings_container, Some("settings"));

        // Set initial page
        main_stack.set_visible_child_name("recipes");

        // Add sidebar and main stack to main box
        main_box.append(&sidebar);

        // Add separator between sidebar and content
        let vert_separator = gtk::Separator::new(gtk::Orientation::Vertical);
        main_box.append(&vert_separator);

        main_box.append(&main_stack);

        // Add main box to window
        root.set_child(Some(&main_box));

        // Initialize widgets struct
        let mut widgets = AppWidgets {
            window: root,
            main_stack,
            //recipes_label: recipes_label.clone(),
            recipes_details: recipes_details, // Store the recipes_details container
            recipes_list_box: recipes_list_box, // Store the recipes list box
            pantry_label: pantry_title.clone(), // Use pantry_title instead of pantry_label
            pantry_list: pantry_list_container, // Store the pantry list as ListBox
            pantry_details: pantry_details_box, // Use pantry_details_box instead of pantry_details
            // pantry_category_filters: category_filters_box; // REMOVED
            pantry_in_stock_switch: stock_filter_switch, // Store in-stock filter switch
            kb_label: kb_label.clone(),
            kb_list_box: kb_list_box, // Store the KB list box
            kb_details: kb_details,   // Store the KB details container
            //settings_label: settings_label.clone(),
            sidebar_buttons,
            refresh_categories: None,
            pantry_row_map: std::collections::HashMap::new(), // slug → ListBoxRow
        };

        // We call update_view after initializing AppModel and AppWidgets so the entire UI is rendered correctly at app start - for example, the sidebar buttons are styled correctly (Recipes tab is highlighted)
        // This is important because the update_view function is responsible for updating the UI based on the current model state
        // It ensures that the initial state of the application is reflected in the UI, such as which tab is currently active and which recipe is selected.
        // The update_view function is called after the widgets are created and added to the main window, ensuring that the UI is in sync with the model state.
        // This is a common pattern in GUI applications, where the initial state of the UI is set up based on the model data.

        // Apply initial view updates based on model state
        AppModel::update_view(&model, &mut widgets, sender.clone());

        // After initializing widgets
        widgets.refresh_categories = refresh_categories;

        ComponentParts { model, widgets }
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
                        pantry::show_edit_ingredient_dialog(
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
                        recipes::show_edit_recipe_dialog(
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
                pantry::show_edit_ingredient_dialog(
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
                            eprintln!("Error updating recipe: {:?}", err);
                            let error_message = format!("Failed to update recipe: {}", err);
                            self.error_message = Some(error_message);
                        }
                    }
                }
            }
            // Message: User clicks the Add Recipe button
            AppMsg::AddRecipe => {
                recipes::show_add_recipe_dialog(self.data_manager.clone(), sender.clone());
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
                println!("DEBUG: Received RefreshCategoryPopover message");
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
                println!("DEBUG: Calling refresh_categories closure from update_view");
                refresh_fn(self);
            } else {
                println!("DEBUG: refresh_categories closure is None");
            }
            self.refresh_category_popover.set(false);
        }

        // Update the main stack to show the current tab
        tabs::update_tab_view(&self.current_tab, &widgets.main_stack);

        // Update sidebar button styles based on the current tab
        sidebar::update_sidebar_buttons(&self.current_tab, &widgets.sidebar_buttons);

        // Highlight the selected recipe in the Recipes list
        if self.current_tab == Tab::Recipes {
            if let Some(recipe_name) = self.selected_recipe.as_ref() {
                println!(
                    "DEBUG: update_view - Recipe selection logic entered. selected_recipe={:?}",
                    recipe_name
                );
                let mut found = false;
                let mut i = 0;
                while let Some(row) = widgets.recipes_list_box.row_at_index(i) {
                    println!("DEBUG: update_view - Checking recipe row {}", i);
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
                    println!(
                        "DEBUG: update_view - Recipe row label_text={:?}",
                        label_text
                    );
                    if label_text == *recipe_name {
                        println!(
                            "DEBUG: update_view - Found matching recipe row at index {}",
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
                println!("DEBUG: update_view - Recipe found={}", found);
                if !found && !self.search_text.is_empty() {
                    println!("DEBUG: update_view - Recipe not found, clearing search_text");
                    let sender_clone = sender.clone();
                    glib::spawn_future_local(async move {
                        sender_clone.input(AppMsg::SearchTextChanged(String::new()));
                    });
                }
            } else {
                println!("DEBUG: update_view - No recipe selected, clearing selection");
                widgets
                    .recipes_list_box
                    .select_row(None::<&gtk::ListBoxRow>);
            }
        }

        // Highlight the selected ingredient in the Pantry list
        if self.current_tab == Tab::Pantry {
            if let Some(selected_slug) = self.selected_ingredient.as_ref() {
                println!("DEBUG: update_view - Ingredient selection logic entered. selected_ingredient(slug)={:?}", selected_slug);
                if let Some(row) = widgets.pantry_row_map.get(selected_slug) {
                    let already_selected = widgets
                        .pantry_list
                        .selected_row()
                        .map(|selected| selected == *row)
                        .unwrap_or(false);
                    if !already_selected {
                        widgets.pantry_list.select_row(Some(row));
                    }
                    println!("DEBUG: update_view - Found matching pantry row for slug in map");
                } else {
                    println!("DEBUG: update_view - Ingredient not found in pantry_row_map");
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
                    let detail = pantry::build_ingredient_detail_view(
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
                    println!("DEBUG: update_view - No ingredient selected, clearing selection");
                    widgets.pantry_list.select_row(None::<&gtk::ListBoxRow>);
                } else {
                    println!("DEBUG: update_view - No ingredient selected, but no row is selected, skipping selection clear");
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
            recipes::update_recipe_details(
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
                kb::update_kb_details::<AppModel>(&widgets.kb_details, &self.data_manager, slug);
            } else {
                kb::show_kb_details_placeholder(&widgets.kb_details);
            }

            // Update the KB list if needed
            // Only do this when first switching to the tab to avoid unnecessary rebuilds
            if self.current_tab != Tab::KnowledgeBase {
                kb::update_kb_list(
                    &widgets.kb_list_box,
                    &self.data_manager,
                    &sender,
                    AppMsg::SelectKnowledgeBaseEntry,
                );
            }
        }

        // Handle About dialog
        if self.show_about_dialog {
            dialogs::show_about_dialog(&widgets.window, &sender, AppMsg::ResetDialogs);
        }

        // Handle Help dialog
        if self.show_help_dialog {
            dialogs::show_help_dialog(&widgets.window, &sender, AppMsg::ResetDialogs);
        }

        // Only rebuild pantry list if flagged
        if self.current_tab == Tab::Pantry && self.pantry_list_needs_rebuild.get() {
            widgets.pantry_row_map.clear();
            pantry::rebuild_pantry_list(
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
            recipes::refresh_recipes_ui(self, widgets, &sender);
        } else {
            // Only update the recipes list if not already handled by refresh_recipes_ui
            recipes::update_recipes_list(
                &widgets.recipes_list_box,
                &self.data_manager,
                &self.search_text,
                self.selected_recipe.as_ref(),
                Some(&sender),
                AppMsg::SelectRecipe,
            );
        }

        if let Some(ref msg) = self.error_message {
            dialogs::show_error_dialog(&widgets.window, msg);
            // Clear the error after showing
            let sender_clone = sender.clone();
            glib::spawn_future_local(async move {
                sender_clone.input(AppMsg::ClearError);
            });
        }
    } // == UPDATE_VIEW ENDS HERE ==
}
//
// The main function initializes the GTK application and runs the app
fn main() {
    let app = RelmApp::new("org.cookbook.CookbookGtk");
    app.run::<AppModel>(());
}
