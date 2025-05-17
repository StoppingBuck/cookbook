// filepath: /home/mpr/code/cookbook/cookbook-gtk/src/main.rs
// main.rs is the entry point for the GTK application
// It initializes the GTK application, sets up the main window, and handles user interactions
// The application is built using the relm4 library, which provides a way to create GTK applications in Rust

mod dialogs;
mod kb;
mod pantry;
mod recipes;
mod sidebar;
mod tabs;
mod types;

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
use cookbook_engine::DataManager; // Import the DataManager from the cookbook_engine module
use gtk::prelude::*; // Import GTK traits for easier usage
use relm4::gtk; // Import GTK bindings from relm4
use relm4::gtk::glib; // Import glib for async operations
use relm4::ComponentParts; // Import to create component parts with model and widgets
use relm4::ComponentSender; // Import to send messages between components
use relm4::RelmApp; // Import application runner for relm4
use relm4::RelmWidgetExt;
use relm4::SimpleComponent; // Import trait for implementing UI components
use std::env; // Import env for accessing environment variables
use std::path::PathBuf; // Import PathBuf for handling file paths
use std::rc::Rc; // Import Rc for reference counting // Import extension traits for widgets
use types::{AppModel, AppMsg, AppWidgets, Tab};

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
        // Get default data directory
        let data_dir = match env::var("COOKBOOK_DATA_DIR") {
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

        // Create initial model with Recipes tab selected by default
        let mut model = AppModel {
            data_manager: None,                     // Data manager will be initialized below
            data_dir: data_dir.clone(),             // Store the data directory
            current_tab: Tab::Recipes,              // Default tab is Recipes
            selected_recipe: None,                  // No recipe selected initially
            selected_ingredient: None,              // No ingredient selected initially
            selected_kb_entry: None,                // No KB entry selected initially
            search_text: String::new(),             // Search bar is empty initially
            show_about_dialog: false,               // About dialog is not shown by default
            show_help_dialog: false,                // Help dialog is not shown by default
            selected_pantry_categories: Vec::new(), // No category filters selected initially
            show_in_stock_only: false,              // Don't filter by stock status initially
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

        // Here comes all the UI code

        // Create the main layout (sidebar + main content area)
        let main_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        // Create sidebar
        let (sidebar, sidebar_buttons) = sidebar::build_sidebar(&sender);

        // Create the main stack for switching between tab content
        let main_stack = gtk::Stack::new();
        main_stack.set_margin_top(10);
        main_stack.set_margin_bottom(10);
        main_stack.set_margin_end(10);
        main_stack.set_hexpand(true);
        main_stack.set_vexpand(true);

        // Create tab content containers

        // Recipes tab content
        let recipes_container = gtk::Box::new(gtk::Orientation::Vertical, 10);

        let recipes_header = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        recipes_header.set_margin_top(10);
        recipes_header.set_margin_bottom(10);
        recipes_header.set_margin_start(10);
        recipes_header.set_margin_end(10);

        let recipes_title = gtk::Label::new(Some("Recipes"));
        recipes_title.set_markup("<span size='x-large' weight='bold'>Recipes</span>");
        recipes_title.set_halign(gtk::Align::Start);
        recipes_title.set_hexpand(true);

        let search_entry = gtk::SearchEntry::new();
        search_entry.set_placeholder_text(Some("Search recipes..."));

        let sender_clone = sender.clone();
        search_entry.connect_search_changed(move |entry| {
            sender_clone.input(AppMsg::SearchTextChanged(entry.text().to_string()));
        });

        recipes_header.append(&recipes_title);
        recipes_header.append(&search_entry);

        recipes_container.append(&recipes_header);

        // Split view for recipes (list on left, details on right)
        let recipes_content = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        recipes_content.set_hexpand(true);
        recipes_content.set_vexpand(true);

        // Recipe list
        let recipes_list_scroll = gtk::ScrolledWindow::new();
        recipes_list_scroll.set_hexpand(false);
        recipes_list_scroll.set_vexpand(true);
        recipes_list_scroll.set_min_content_width(250);

        let recipes_list_box = gtk::ListBox::new();
        recipes_list_box.set_selection_mode(gtk::SelectionMode::Single);

        // Add recipes to list if available
        if let Some(ref dm) = model.data_manager {
            // Use empty search text initially to get all recipes
            let recipes = dm.search_recipes("");
            if !recipes.is_empty() {
                for recipe in recipes {
                    let row = gtk::ListBoxRow::new();
                    let title_label = gtk::Label::new(Some(&recipe.title));
                    title_label.set_halign(gtk::Align::Start);
                    title_label.set_margin_start(5);
                    title_label.set_margin_end(5);
                    title_label.set_margin_top(5);
                    title_label.set_margin_bottom(5);
                    row.set_child(Some(&title_label));

                    recipes_list_box.append(&row);
                }
            } else {
                let no_recipes_row = gtk::ListBoxRow::new();
                let no_recipes_label = gtk::Label::new(Some("No recipes available"));
                no_recipes_label.set_margin_all(10);
                no_recipes_row.set_child(Some(&no_recipes_label));
                recipes_list_box.append(&no_recipes_row);
            }
        } else {
            let no_data_row = gtk::ListBoxRow::new();
            let no_data_label = gtk::Label::new(Some("Failed to load recipe data"));
            no_data_label.set_margin_all(10);
            no_data_row.set_child(Some(&no_data_label));
            recipes_list_box.append(&no_data_row);
        }

        // Recipe selection handler - callback for recipes_list_box.connect_row_selected
        // - Triggered whenever a row in the recipe list is selected
        // - Gets the recipe title from the row label directly
        // - Sends an AppMsg::SelectRecipe message with the recipe title
        let sender_clone = sender.clone();
        recipes_list_box.connect_row_selected(move |_list, row_opt| {
            if let Some(row) = row_opt {
                if let Some(child) = row.child() {
                    // With the checkmark addition, the child is now a Box, not directly a Label
                    if let Some(box_container) = child.downcast_ref::<gtk::Box>() {
                        // Find the Label within the Box (it should be the first child)
                        if let Some(label_widget) = box_container.first_child() {
                            if let Some(label) = label_widget.downcast_ref::<gtk::Label>() {
                                // Extract recipe title, removing the checkmark if present
                                let raw_text = label.text().to_string();
                                // Remove checkmark if present
                                let recipe_title = raw_text.trim_end_matches(" ✅").to_string();
                                sender_clone.input(AppMsg::SelectRecipe(recipe_title));
                            }
                        }
                    } else if let Some(label) = child.downcast_ref::<gtk::Label>() {
                        // Fallback for old-style rows without Box
                        let recipe_title = label.text().to_string();
                        sender_clone.input(AppMsg::SelectRecipe(recipe_title));
                    }
                }
            }
        });

        recipes_list_scroll.set_child(Some(&recipes_list_box));

        // Recipe details view
        let recipes_details = gtk::Box::new(gtk::Orientation::Vertical, 10);
        recipes_details.set_hexpand(true);
        recipes_details.set_vexpand(true);

        let recipes_label = gtk::Label::new(Some("Select a recipe to view details"));
        recipes_label.set_halign(gtk::Align::Center);
        recipes_label.set_valign(gtk::Align::Center);
        recipes_label.set_hexpand(true);
        recipes_label.set_vexpand(true);

        recipes_details.append(&recipes_label);

        recipes_content.append(&recipes_list_scroll);
        recipes_content.append(&recipes_details);

        recipes_container.append(&recipes_content);

        // Pantry tab content
        let pantry_container = gtk::Box::new(gtk::Orientation::Vertical, 10);

        // Pantry header with title and search
        let pantry_header = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        pantry_header.set_margin_top(10);
        pantry_header.set_margin_bottom(10);
        pantry_header.set_margin_start(10);
        pantry_header.set_margin_end(10);

        let pantry_title = gtk::Label::new(Some("Pantry"));
        pantry_title.set_markup("<span size='x-large' weight='bold'>Pantry</span>");
        pantry_title.set_halign(gtk::Align::Start);
        pantry_title.set_hexpand(true);

        let pantry_search_entry = gtk::SearchEntry::new();
        pantry_search_entry.set_placeholder_text(Some("Search ingredients..."));

        let sender_clone = sender.clone();
        pantry_search_entry.connect_search_changed(move |entry| {
            sender_clone.input(AppMsg::SearchTextChanged(entry.text().to_string()));
        });

        pantry_header.append(&pantry_title);
        pantry_header.append(&pantry_search_entry);

        pantry_container.append(&pantry_header);

        // Filters section
        let filters_frame = gtk::Frame::new(Some("Filters"));
        filters_frame.set_margin_start(10);
        filters_frame.set_margin_end(10);

        let filters_container = gtk::Box::new(gtk::Orientation::Vertical, 5);
        filters_container.set_margin_all(10);

        // Category filters
        let category_filters_label = gtk::Label::new(Some("Categories:"));
        category_filters_label.set_halign(gtk::Align::Start);
        category_filters_label.set_margin_bottom(5);

        let category_filters_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        category_filters_box.set_margin_bottom(10);

        // Get unique categories from ingredients using the engine's method
        let mut categories = Vec::new();
        if let Some(ref dm) = model.data_manager {
            categories = dm.get_all_ingredient_categories();
        }

        // Create filter checkboxes
        for category in &categories {
            let check_button = gtk::CheckButton::with_label(category);
            let sender_clone = sender.clone();
            let category_clone = category.clone();

            check_button.connect_toggled(move |check| {
                sender_clone.input(AppMsg::ToggleCategoryFilter(
                    category_clone.clone(),
                    check.is_active(),
                ));
            });

            category_filters_box.append(&check_button);
        }

        // In-stock only filter
        let stock_filter_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let stock_filter_label = gtk::Label::new(Some("Show in-stock items only:"));
        stock_filter_label.set_halign(gtk::Align::Start);

        let stock_filter_switch = gtk::Switch::new();
        let sender_clone = sender.clone();
        stock_filter_switch.connect_state_notify(move |switch| {
            sender_clone.input(AppMsg::ToggleInStockFilter(switch.state()));
        });

        stock_filter_box.append(&stock_filter_label);
        stock_filter_box.append(&stock_filter_switch);

        filters_container.append(&category_filters_label);
        filters_container.append(&category_filters_box);
        filters_container.append(&stock_filter_box);

        filters_frame.set_child(Some(&filters_container));
        pantry_container.append(&filters_frame);

        // Split view for pantry (list on left, details on right)
        let pantry_content = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        pantry_content.set_hexpand(true);
        pantry_content.set_vexpand(true);
        pantry_content.set_margin_top(10);
        pantry_content.set_margin_start(10);
        pantry_content.set_margin_end(10);
        pantry_content.set_margin_bottom(10);

        // Pantry list
        let pantry_list_scroll = gtk::ScrolledWindow::new();
        pantry_list_scroll.set_hexpand(false);
        pantry_list_scroll.set_vexpand(true);
        pantry_list_scroll.set_min_content_width(300);

        let pantry_list_container = gtk::Box::new(gtk::Orientation::Vertical, 0);

        // Pantry details (right side)
        let pantry_details_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
        pantry_details_box.set_hexpand(true);
        pantry_details_box.set_vexpand(true);

        // Initial content for the details
        let select_label = gtk::Label::new(Some("Select an ingredient to view details"));
        select_label.set_halign(gtk::Align::Center);
        select_label.set_valign(gtk::Align::Center);
        select_label.set_hexpand(true);
        select_label.set_vexpand(true);
        pantry_details_box.append(&select_label);

        // Group by categories using the engine method
        let mut pantry_items_by_category: std::collections::HashMap<
            String,
            Vec<(String, Option<String>, Option<String>, bool)>,
        > = std::collections::HashMap::new();

        if let Some(ref dm) = model.data_manager {
            // Use the engine's method to get pantry items grouped by category
            let items_by_category = dm.get_pantry_items_by_category();

            // Convert to the format expected by the UI
            for (category, items) in items_by_category {
                let mut category_items = Vec::new();

                for (ingredient, pantry_item) in items {
                    let is_in_stock = pantry_item.is_some();
                    let (quantity, quantity_type) = if let Some(item) = pantry_item {
                        (
                            item.quantity.map(|q| q.to_string()),
                            Some(item.quantity_type.clone()),
                        )
                    } else {
                        (None, Some(String::new()))
                    };

                    category_items.push((
                        ingredient.name.clone(),
                        quantity,
                        quantity_type,
                        is_in_stock,
                    ));
                }

                pantry_items_by_category.insert(category, category_items);
            }

            // Sort categories and ingredients
            let mut sorted_categories: Vec<String> =
                pantry_items_by_category.keys().cloned().collect();
            sorted_categories.sort();

            for category in sorted_categories {
                // Create category header
                let category_frame = gtk::Frame::new(Some(&category));
                category_frame.set_margin_bottom(10);

                let category_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
                category_frame.set_child(Some(&category_box));

                if let Some(items) = pantry_items_by_category.get_mut(&category) {
                    // Sort ingredients alphabetically within category
                    items.sort_by(|a, b| a.0.cmp(&b.0));

                    for (name, quantity, quantity_type, is_in_stock) in items.iter() {
                        let item_row = gtk::Box::new(gtk::Orientation::Horizontal, 5);
                        item_row.set_margin_all(5);

                        // Create the item label with quantity if available
                        let mut label_text = name.clone();
                        if let Some(q) = quantity {
                            if let Some(t) = quantity_type.as_ref() {
                                if t.is_empty() {
                                    label_text = format!("{} ({})", name, q);
                                } else {
                                    label_text = format!("{} ({} {})", name, q, t);
                                }
                            } else {
                                label_text = format!("{} ({})", name, q);
                            }
                        }

                        // Add checkmark for in-stock items
                        if *is_in_stock {
                            label_text = format!("{} ✅", label_text);
                        }

                        let item_label = gtk::Label::new(Some(&label_text));
                        item_label.set_halign(gtk::Align::Start);
                        item_label.set_hexpand(true);
                        item_row.append(&item_label);

                        // Make the row selectable
                        let click_gesture = gtk::GestureClick::new();
                        item_row.add_css_class("pantry-item");
                        item_row.add_controller(click_gesture.clone());

                        // Highlight will be added in update_view function
                        // Can't access self.selected_ingredient here

                        let sender_clone = sender.clone();
                        let name_clone = name.clone();
                        click_gesture.connect_pressed(move |_, _, _, _| {
                            sender_clone.input(AppMsg::SelectIngredient(name_clone.clone()));
                        });

                        category_box.append(&item_row);
                    }
                }

                pantry_list_container.append(&category_frame);
            }
        } else {
            // No data available
            let no_data_label = gtk::Label::new(Some("No ingredient data available"));
            no_data_label.set_margin_all(10);
            pantry_list_container.append(&no_data_label);
        }

        pantry_list_scroll.set_child(Some(&pantry_list_container));

        // Add both parts to the pantry content
        pantry_content.append(&pantry_list_scroll);
        pantry_content.append(&pantry_details_box);

        // Add pantry content to container
        pantry_container.append(&pantry_content);

        // Knowledge base tab content
        let kb_container = gtk::Box::new(gtk::Orientation::Vertical, 10);

        let kb_title = gtk::Label::new(Some("Knowledge Base"));
        kb_title.set_markup("<span size='x-large' weight='bold'>Knowledge Base</span>");
        kb_title.set_halign(gtk::Align::Start);
        kb_title.set_margin_all(10);

        kb_container.append(&kb_title);

        // Split view for Knowledge Base (list on left, details on right)
        let kb_content = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        kb_content.set_hexpand(true);
        kb_content.set_vexpand(true);

        // Knowledge Base list (left side)
        let kb_list_scroll = gtk::ScrolledWindow::new();
        kb_list_scroll.set_hexpand(false);
        kb_list_scroll.set_vexpand(true);
        kb_list_scroll.set_min_content_width(250);

        let kb_list_box = gtk::ListBox::new();
        kb_list_box.set_selection_mode(gtk::SelectionMode::Single);

        // Add KB entries to list if available
        if let Some(ref dm) = model.data_manager {
            let entries = dm.get_all_kb_entries();
            if !entries.is_empty() {
                // Sort entries by title for better usability
                let mut sorted_entries = entries.clone();
                sorted_entries.sort_by(|a, b| a.title.cmp(&b.title));

                for entry in sorted_entries {
                    let row = gtk::ListBoxRow::new();
                    let title_label = gtk::Label::new(Some(&entry.title));
                    title_label.set_halign(gtk::Align::Start);
                    title_label.set_margin_start(5);
                    title_label.set_margin_end(5);
                    title_label.set_margin_top(5);
                    title_label.set_margin_bottom(5);
                    row.set_child(Some(&title_label));

                    kb_list_box.append(&row);
                }
            } else {
                let no_entries_row = gtk::ListBoxRow::new();
                let no_entries_label = gtk::Label::new(Some("No KB entries available"));
                no_entries_label.set_margin_all(10);
                no_entries_row.set_child(Some(&no_entries_label));
                kb_list_box.append(&no_entries_row);
            }
        } else {
            let no_data_row = gtk::ListBoxRow::new();
            let no_data_label = gtk::Label::new(Some("Failed to load KB data"));
            no_data_label.set_margin_all(10);
            no_data_row.set_child(Some(&no_data_label));
            kb_list_box.append(&no_data_row);
        }

        // KB entry selection handler
        let sender_clone = sender.clone();
        let dm_clone = model.data_manager.clone();
        kb_list_box.connect_row_selected(move |_list, row_opt| {
            if let Some(row) = row_opt {
                if let Some(ref dm) = dm_clone {
                    let entries = dm.get_all_kb_entries();
                    if row.index() >= 0 && (row.index() as usize) < entries.len() {
                        let entry = &entries[row.index() as usize];
                        sender_clone.input(AppMsg::SelectKnowledgeBaseEntry(entry.slug.clone()));
                    }
                }
            }
        });

        kb_list_scroll.set_child(Some(&kb_list_box));

        // KB details view (right side)
        let kb_details = gtk::Box::new(gtk::Orientation::Vertical, 10);
        kb_details.set_hexpand(true);
        kb_details.set_vexpand(true);

        let kb_label = gtk::Label::new(Some("Select an item to view details"));
        kb_label.set_halign(gtk::Align::Center);
        kb_label.set_valign(gtk::Align::Center);
        kb_label.set_hexpand(true);
        kb_label.set_vexpand(true);

        kb_details.append(&kb_label);

        kb_content.append(&kb_list_scroll);
        kb_content.append(&kb_details);

        kb_container.append(&kb_content);

        // Settings tab content
        let settings_container = gtk::Box::new(gtk::Orientation::Vertical, 10);

        let settings_title = gtk::Label::new(Some("Settings"));
        settings_title.set_markup("<span size='x-large' weight='bold'>Settings</span>");
        settings_title.set_halign(gtk::Align::Start);
        settings_title.set_margin_all(10);

        let settings_label =
            gtk::Label::new(Some("Settings will be implemented in a future version."));
        settings_label.set_halign(gtk::Align::Start);
        settings_label.set_margin_start(10);

        settings_container.append(&settings_title);
        settings_container.append(&settings_label);

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
            recipes_label: recipes_label.clone(),
            recipes_details: recipes_details, // Store the recipes_details container
            recipes_list_box: recipes_list_box, // Store the recipes list box
            pantry_label: pantry_title.clone(), // Use pantry_title instead of pantry_label
            pantry_list: pantry_list_container, // Store the pantry list container
            pantry_details: pantry_details_box, // Use pantry_details_box instead of pantry_details
            pantry_category_filters: category_filters_box, // Store category filter checkboxes
            pantry_in_stock_switch: stock_filter_switch, // Store in-stock filter switch
            kb_label: kb_label.clone(),
            kb_list_box: kb_list_box, // Store the KB list box
            kb_details: kb_details,   // Store the KB details container
            settings_label: settings_label.clone(),
            sidebar_buttons,
        };

        // We call update_view after initializing AppModel and AppWidgets so the entire UI is rendered correctly at app start - for example, the sidebar buttons are styled correctly (Recipes tab is highlighted)
        // This is important because the update_view function is responsible for updating the UI based on the current model state
        // It ensures that the initial state of the application is reflected in the UI, such as which tab is currently active and which recipe is selected.
        // The update_view function is called after the widgets are created and added to the main window, ensuring that the UI is in sync with the model state.
        // This is a common pattern in GUI applications, where the initial state of the UI is set up based on the model data.

        // Apply initial view updates based on model state
        AppModel::update_view(&model, &mut widgets, sender.clone());

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
            // Message: User toggles a category filter
            // - If the category is selected, add it to the selected categories
            AppMsg::ToggleCategoryFilter(category, is_selected) => {
                if is_selected && !self.selected_pantry_categories.contains(&category) {
                    self.selected_pantry_categories.push(category);
                } else if !is_selected {
                    self.selected_pantry_categories.retain(|c| c != &category);
                }
            }
            // Message: User toggles the in-stock filter
            AppMsg::ToggleInStockFilter(show_in_stock_only) => {
                self.show_in_stock_only = show_in_stock_only;
            }
            // Message: User types in the search bar
            AppMsg::SearchTextChanged(text) => {
                self.search_text = text;
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
            // Message: User updates an ingredient with pantry data
            AppMsg::UpdateIngredientWithPantry(
                original_name,
                new_ingredient,
                quantity,
                quantity_type,
            ) => {
                if let Some((updated_manager, new_selected_name)) =
                    pantry::handle_update_ingredient_with_pantry(
                        &self.data_manager,
                        &original_name,
                        new_ingredient,
                        quantity,
                        quantity_type,
                        self.current_tab.clone(),
                        &sender,
                    )
                {
                    self.data_manager = Some(updated_manager);
                    self.selected_ingredient = Some(new_selected_name);
                }
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

                            // Show error dialog to the user in a safe way
                            let error_message = format!("Failed to update recipe: {}", err);

                            glib::spawn_future_local(async move {
                                let dialog = gtk::MessageDialog::builder()
                                    .modal(true)
                                    .buttons(gtk::ButtonsType::Ok)
                                    .message_type(gtk::MessageType::Error)
                                    .text(&error_message)
                                    .build();

                                dialog.connect_response(|dialog, _| dialog.destroy());
                                dialog.show();
                            });
                        }
                    }
                }
            }
        }
    }
    // == UPDATE ENDS HERE ==

    // == UPDATE_VIEW STARTS HERE ==
    // update_view updates the UI based on the current model state
    fn update_view(&self, widgets: &mut Self::Widgets, sender: ComponentSender<Self>) {
        // Update the main stack to show the current tab
        tabs::update_tab_view(&self.current_tab, &widgets.main_stack);

        // Update sidebar button styles based on the current tab
        sidebar::update_sidebar_buttons(&self.current_tab, &widgets.sidebar_buttons);

        // Select the correct recipe in the list box when a recipe is selected
        if self.current_tab == Tab::Recipes && self.selected_recipe.is_some() {
            let recipe_name = self.selected_recipe.as_ref().unwrap();

            // Find the row with the matching recipe title by iterating through the list box
            let mut found = false;
            let mut i = 0;
            while let Some(row) = widgets.recipes_list_box.row_at_index(i) {
                i += 1; // Move to next index
                if let Some(child) = row.child() {
                    if let Some(label) = child.downcast_ref::<gtk::Label>() {
                        if label.text() == *recipe_name {
                            // Select the row (this will highlight it in the UI)
                            widgets.recipes_list_box.select_row(Some(&row));
                            found = true;
                            break;
                        }
                    }
                }
            }

            // If the recipe is not in the current filtered list, clear the filter
            if !found && !self.search_text.is_empty() {
                // Reset search text next time update runs
                // This is a bit of a hack, but it prevents recursion issues
                let sender_clone = sender.clone();
                glib::spawn_future_local(async move {
                    sender_clone.input(AppMsg::SearchTextChanged(String::new()));
                });
            }
        }

        // Update pantry details if an ingredient is selected
        if self.current_tab == Tab::Pantry && self.selected_ingredient.is_some() {
            let ingredient_name = self.selected_ingredient.as_ref().unwrap();

            // Clear previous content
            while let Some(child) = widgets.pantry_details.first_child() {
                widgets.pantry_details.remove(&child);
            }

            if let Some(ref dm) = self.data_manager {
                let details_view = pantry::build_ingredient_detail_view(
                    dm,
                    ingredient_name,
                    &sender,
                    AppMsg::SwitchTab,
                    AppMsg::SelectKnowledgeBaseEntry,
                    AppMsg::SelectRecipe,
                    AppMsg::EditIngredient,
                );
                widgets.pantry_details.append(&details_view);
            } else {
                // Data manager not available
                let error_label = gtk::Label::new(Some(
                    "Unable to load ingredient: data manager not available",
                ));
                error_label.set_halign(gtk::Align::Center);
                error_label.set_valign(gtk::Align::Center);
                widgets.pantry_details.append(&error_label);
            }
        } else if self.current_tab == Tab::Pantry && self.selected_ingredient.is_none() {
            // No ingredient selected
            // Clear previous content
            while let Some(child) = widgets.pantry_details.first_child() {
                widgets.pantry_details.remove(&child);
            }

            let select_label = gtk::Label::new(Some("Select an ingredient to view details"));
            select_label.set_halign(gtk::Align::Center);
            select_label.set_valign(gtk::Align::Center);
            widgets.pantry_details.append(&select_label);
        }

        // Update recipe details if a recipe is selected

        // Update recipe details if a recipe is selected
        if self.current_tab == Tab::Recipes {
            recipes::update_recipe_details(
                self.selected_recipe.as_deref(),
                &widgets.recipes_details,
                &self.data_manager,
                &sender,
                AppMsg::EditRecipe,
            );
        }

        // Update KB entry details if a KB entry is selected
        if self.current_tab == Tab::KnowledgeBase {
            // Select the correct KB entry in the list box
            if let Some(kb_slug) = &self.selected_kb_entry {
                // Update the selection in the list
                kb::select_kb_entry_in_list(&widgets.kb_list_box, kb_slug);

                // Update the details view
                kb::update_kb_details::<Self>(
                    &widgets.kb_details,
                    &self.data_manager,
                    kb_slug,
                    &self.data_dir,
                );
            } else {
                // No KB entry selected, show placeholder
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

        // Rebuild pantry list when filters change or search text changes
        if self.current_tab == Tab::Pantry {
            pantry::rebuild_pantry_list(
                &widgets.pantry_list,
                &self.data_manager,
                &self.search_text,
                &self.selected_pantry_categories,
                self.show_in_stock_only,
                &sender,
                AppMsg::SelectIngredient,
            );
        }

        // Update recipes list when search text changes
        recipes::update_recipes_list(
            &widgets.recipes_list_box,
            &self.data_manager,
            &self.search_text,
            &sender,
            AppMsg::SelectRecipe,
        );
    } // == UPDATE_VIEW ENDS HERE ==
}
//
// The main function initializes the GTK application and runs the app
fn main() {
    let app = RelmApp::new("org.cookbook.CookbookGtk");
    app.run::<AppModel>(());
}
