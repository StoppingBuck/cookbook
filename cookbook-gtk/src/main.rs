// filepath: /home/mpr/code/cookbook/cookbook-gtk/src/main.rs
// main.rs is the entry point for the GTK application
// It initializes the GTK application, sets up the main window, and handles user interactions
// The application is built using the relm4 library, which provides a way to create GTK applications in Rust

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
use gtk::prelude::*;                // Import GTK traits for easier usage
use cookbook_engine::DataManager;   // Import the DataManager from the cookbook_engine module
use std::path::PathBuf;             // Import PathBuf for handling file paths
use std::env;                       // Import env for accessing environment variables
use std::rc::Rc;                    // Import Rc for reference counting
use relm4::gtk;                     // Import GTK bindings from relm4
use relm4::ComponentParts;          // Import to create component parts with model and widgets
use relm4::ComponentSender;         // Import to send messages between components
use relm4::SimpleComponent;         // Import trait for implementing UI components
use relm4::RelmApp;                 // Import application runner for relm4
use relm4::RelmWidgetExt;           // Import extension traits for widgets

// The main application model, representing the state of the app (e.g., current tab, selected recipe)
#[allow(dead_code)]
struct AppModel {
    data_manager: Option<Rc<DataManager>>,  // Manages data loading and saving (e.g., recipes, pantry items)
    data_dir: PathBuf,                      // Path to the directory containing the data files
    current_tab: Tab,                       // The currently selected tab in the UI (e.g., Recipes, Pantry)
    selected_recipe: Option<String>,        // The name of the currently selected recipe, if any
    selected_ingredient: Option<String>,    // The name of the currently selected ingredient, if any
    search_text: String,                    // The current text in the search bar
    show_about_dialog: bool,                // Flag to indicate if the About dialog should be shown
    show_help_dialog: bool,                 // Flag to indicate if the Help dialog should be shown
    selected_pantry_categories: Vec<String>, // The currently selected categories for filtering the pantry
    show_in_stock_only: bool,              // Flag to indicate if only in-stock ingredients should be shown
}

// Enum representing the different tabs in the application
#[derive(Debug, Clone, PartialEq)]
enum Tab {
    Recipes,       // Recipes tab
    Pantry,        // Pantry tab
    KnowledgeBase, // Knowledge Base tab
    Settings,      // Settings tab
}

// Messages that the app can respond to (e.g., user actions)
#[derive(Debug)]
enum AppMsg {
    SwitchTab(Tab),            // Switch to a different tab
    ShowAbout,                 // Show the About dialog
    ShowHelp,                  // Show the Help dialog
    ResetDialogs,              // Reset dialog flags
    SelectRecipe(String),      // Select a recipe by name
    SelectIngredient(String),  // Select an ingredient by name
    ToggleCategoryFilter(String, bool), // Toggle a category filter (category, is_selected)
    ToggleInStockFilter(bool), // Toggle the in-stock only filter
    SearchTextChanged(String), // Update the search text
}

// References to the GTK widgets used in the app (e.g. buttons, labels, stack)
#[allow(dead_code)]
struct AppWidgets {
    window: gtk::ApplicationWindow,     // The main application window
    main_stack: gtk::Stack,             // The stack for switching between tabs
    recipes_label: gtk::Label,          // Label for displaying recipe details
    recipes_details: gtk::Box,          // Container for recipe details
    pantry_label: gtk::Label,           // Label for displaying pantry info
    pantry_list: gtk::Box,              // Container for the pantry list items
    pantry_details: gtk::Box,           // Container for pantry item details
    pantry_category_filters: gtk::Box,  // Container for category filter checkboxes
    pantry_in_stock_switch: gtk::Switch, // Switch for toggling in-stock only filter
    kb_label: gtk::Label,               // Label for displaying knowledge base info
    settings_label: gtk::Label,         // Label for displaying settings info
    sidebar_buttons: Vec<gtk::Button>,  // Buttons in the sidebar
}

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
        let data_dir = match env::var("COOKBOOK_DATA_DIR") { // Get the data directory from environment variable
            Ok(path) => PathBuf::from(path), // If the environment variable is set, use it
            Err(_) => {                              // If not set, use the example data folder as default
                let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));   // Get the current directory
                path.pop();                                                          // Go up one level from cookbook-gtk
                path.push("example");                                          // Go into the example folder
                path.push("data");                                             // Go into the data folder
                path                                                                // Return the path
            }
        };

        // Create initial model with Recipes tab selected by default
        let mut model = AppModel {
            data_manager: None,         // Data manager will be initialized below
            data_dir: data_dir.clone(), // Store the data directory
            current_tab: Tab::Recipes,  // Default tab is Recipes
            selected_recipe: None,      // No recipe selected initially
            selected_ingredient: None,  // No ingredient selected initially
            search_text: String::new(), // Search bar is empty initially
            show_about_dialog: false,   // About dialog is not shown by default
            show_help_dialog: false,    // Help dialog is not shown by default
            selected_pantry_categories: Vec::new(), // No category filters selected initially
            show_in_stock_only: false,  // Don't filter by stock status initially
        };

        // Load data using the DataManager
        model.data_manager = match DataManager::new(&data_dir) {
            Ok(manager) => {
                println!("Data loaded successfully from: {}", data_dir.display());
                Some(Rc::new(manager))
            },
            Err(e) => {
                eprintln!("Error loading data: {}", e);
                None
            }
        };

        // Here comes all the UI code

        // Create the main layout (sidebar + main content area)
        let main_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        // Create the sidebar (navigation buttons)
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

        // Create sidebar buttons for navigation
        let mut sidebar_buttons = vec![];

        let recipes_button  = gtk::Button::with_label("Recipes");
        let pantry_button   = gtk::Button::with_label("Pantry");
        let kb_button       = gtk::Button::with_label("Knowledge Base");
        let settings_button = gtk::Button::with_label("Settings");
        let about_button    = gtk::Button::with_label("About");
        let help_button     = gtk::Button::with_label("Help");

        // Connect sidebar button signals to handle tab switching
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

        sidebar_buttons.push(recipes_button);
        sidebar_buttons.push(pantry_button);
        sidebar_buttons.push(kb_button);
        sidebar_buttons.push(settings_button);

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
            let recipes = dm.get_all_recipes();
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
        // - Gets the index of the selected row
        // - Fetches the corresponding recipe from the DataManager
        // - Sends an AppMsg::SelectRecipe message with the recipe title
        let sender_clone = sender.clone();
        let dm_clone = model.data_manager.clone();
        recipes_list_box.connect_row_selected(move |_list, row_opt| {
            if let Some(row) = row_opt {
                if let Some(ref dm) = dm_clone {
                    let recipes = dm.get_all_recipes();
                    if row.index() >= 0 && (row.index() as usize) < recipes.len() {
                        let recipe = &recipes[row.index() as usize];
                        sender_clone.input(AppMsg::SelectRecipe(recipe.title.clone()));
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
        
        // Get unique categories from ingredients
        let mut categories = Vec::new();
        if let Some(ref dm) = model.data_manager {
            for ingredient in dm.get_all_ingredients() {
                if !categories.contains(&ingredient.category) {
                    categories.push(ingredient.category.clone());
                }
            }
        }
        categories.sort(); // Sort alphabetically
        
        // Create filter checkboxes
        for category in &categories {
            let check_button = gtk::CheckButton::with_label(category);
            let sender_clone = sender.clone();
            let category_clone = category.clone();
            
            check_button.connect_toggled(move |check| {
                sender_clone.input(AppMsg::ToggleCategoryFilter(category_clone.clone(), check.is_active()));
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
        
        // Group by categories
        let mut pantry_items_by_category: std::collections::HashMap<String, Vec<(String, Option<String>, Option<String>, bool)>> = std::collections::HashMap::new();
        
        if let Some(ref dm) = model.data_manager {
            let pantry = dm.get_pantry();
            
            // Create a map of ingredient name -> is in stock
            let mut in_pantry = std::collections::HashMap::new();
            if let Some(pantry) = pantry {
                for item in &pantry.items {
                    // Convert f64 quantity to String for display
                    let quantity_str = item.quantity.map(|q| q.to_string());
                    in_pantry.insert(item.ingredient.clone(), (quantity_str, item.quantity_type.clone()));
                }
            }
            
            // Group ingredients by category
            for ingredient in dm.get_all_ingredients() {
                let is_in_stock = in_pantry.contains_key(&ingredient.name);
                let (quantity, quantity_type) = if let Some((q, t)) = in_pantry.get(&ingredient.name) {
                    (q.clone(), t.clone())
                } else {
                    (None, None)
                };
                
                pantry_items_by_category
                    .entry(ingredient.category.clone())
                    .or_default()
                    .push((ingredient.name.clone(), quantity, quantity_type, is_in_stock));
            }
            
            // Sort categories and ingredients
            let mut sorted_categories: Vec<String> = pantry_items_by_category.keys().cloned().collect();
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
                        if let (Some(q), Some(t)) = (quantity, quantity_type) {
                            label_text = format!("{} ({} {})", name, q, t);
                        } else if let (Some(q), None) = (quantity, quantity_type) {
                            label_text = format!("{} ({})", name, q);
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
        
        // Ingredient details view (small)
        let pantry_details = gtk::Box::new(gtk::Orientation::Vertical, 10);
        pantry_details.set_hexpand(true);
        pantry_details.set_vexpand(true);
        
        let pantry_label = gtk::Label::new(Some("Select an ingredient to view details"));
        pantry_label.set_halign(gtk::Align::Center);
        pantry_label.set_valign(gtk::Align::Center);
        pantry_label.set_hexpand(true);
        pantry_label.set_vexpand(true);
        
        pantry_details.append(&pantry_label);
        
        pantry_content.append(&pantry_list_scroll);
        pantry_content.append(&pantry_details);
        
        pantry_container.append(&pantry_content);
        
        // Knowledge base tab content
        let kb_container = gtk::Box::new(gtk::Orientation::Vertical, 10);
        
        let kb_title = gtk::Label::new(Some("Knowledge Base"));
        kb_title.set_markup("<span size='x-large' weight='bold'>Knowledge Base</span>");
        kb_title.set_halign(gtk::Align::Start);
        kb_title.set_margin_all(10);
        
        let kb_label = gtk::Label::new(None);
        if let Some(ref dm) = model.data_manager {
            let kb_entries = dm.get_all_kb_entries();
            kb_label.set_text(&format!("The knowledge base contains {} entries.", kb_entries.len()));
        } else {
            kb_label.set_text("No knowledge base data available");
        }
        
        kb_label.set_halign(gtk::Align::Start);
        kb_label.set_margin_start(10);
        
        kb_container.append(&kb_title);
        kb_container.append(&kb_label);
        
        // Settings tab content
        let settings_container = gtk::Box::new(gtk::Orientation::Vertical, 10);
        
        let settings_title = gtk::Label::new(Some("Settings"));
        settings_title.set_markup("<span size='x-large' weight='bold'>Settings</span>");
        settings_title.set_halign(gtk::Align::Start);
        settings_title.set_margin_all(10);
        
        let settings_label = gtk::Label::new(Some("Settings will be implemented in a future version."));
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
            recipes_details: recipes_details,      // Store the recipes_details container
            pantry_label: pantry_label.clone(),
            pantry_list: pantry_list_container,    // Store the pantry list container
            pantry_details: pantry_details,        // Store the pantry details container
            pantry_category_filters: category_filters_box, // Store category filter checkboxes
            pantry_in_stock_switch: stock_filter_switch,  // Store in-stock filter switch
            kb_label: kb_label.clone(),
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
    
    // update handles incoming messages (e.g. switching tabs, selecting a recipe) and updates the model state
    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            AppMsg::SwitchTab(new_tab) => {
                self.current_tab = new_tab;
                // Reset selection when changing tabs
                if self.current_tab == Tab::Recipes {
                    self.selected_recipe = None;
                }
                if self.current_tab == Tab::Pantry {
                    self.selected_ingredient = None;
                }
            }
            AppMsg::ShowAbout => {
                self.show_about_dialog = true;
            }
            AppMsg::ShowHelp => {
                self.show_help_dialog = true;
            }
            AppMsg::ResetDialogs => {
                // Reset all dialog flags
                self.show_about_dialog = false;
                self.show_help_dialog = false;
            }
            AppMsg::SelectRecipe(recipe_name) => {
                self.selected_recipe = Some(recipe_name);
            }
            AppMsg::SelectIngredient(ingredient_name) => {
                self.selected_ingredient = Some(ingredient_name);
            }
            AppMsg::ToggleCategoryFilter(category, is_selected) => {
                if is_selected && !self.selected_pantry_categories.contains(&category) {
                    self.selected_pantry_categories.push(category);
                } else if !is_selected {
                    self.selected_pantry_categories.retain(|c| c != &category);
                }
            }
            AppMsg::ToggleInStockFilter(show_in_stock_only) => {
                self.show_in_stock_only = show_in_stock_only;
            }
            AppMsg::SearchTextChanged(text) => {
                self.search_text = text;
            }
        }
    }

    // update_view updates the UI based on the current model state
    fn update_view(&self, widgets: &mut Self::Widgets, sender: ComponentSender<Self>) {
        // Update stack page based on current tab
        match self.current_tab {
            Tab::Recipes => widgets.main_stack.set_visible_child_name("recipes"),
            Tab::Pantry => widgets.main_stack.set_visible_child_name("pantry"),
            Tab::KnowledgeBase => widgets.main_stack.set_visible_child_name("kb"),
            Tab::Settings => widgets.main_stack.set_visible_child_name("settings"),
        }

        // Update sidebar button styling
        for (i, button) in widgets.sidebar_buttons.iter().enumerate() {
            let tab = match i {
                0 => Tab::Recipes,
                1 => Tab::Pantry,
                2 => Tab::KnowledgeBase,
                3 => Tab::Settings,
                _ => continue,
            };

            if tab == self.current_tab {
                button.add_css_class("suggested-action");
            } else {
                button.remove_css_class("suggested-action");
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
                if let Some(ingredient) = dm.get_ingredient(ingredient_name) {
                    // Create a small details view
                    let details_container = gtk::Box::new(gtk::Orientation::Vertical, 10);
                    details_container.set_margin_all(10);
                    
                    // Title with ingredient name
                    let title = gtk::Label::new(None);
                    title.set_markup(&format!("<span size='x-large' weight='bold'>{}</span>", ingredient.name));
                    title.set_halign(gtk::Align::Start);
                    title.set_margin_bottom(10);
                    details_container.append(&title);
                    
                    // Category
                    let category = gtk::Label::new(None);
                    category.set_markup(&format!("<b>Category:</b> {}", ingredient.category));
                    category.set_halign(gtk::Align::Start);
                    details_container.append(&category);
                    
                    // Tags
                    if !ingredient.tags.is_empty() {
                        let tags_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
                        tags_box.set_margin_top(5);
                        
                        let tags_label = gtk::Label::new(Some("Tags:"));
                        tags_label.set_halign(gtk::Align::Start);
                        tags_box.append(&tags_label);
                        
                        for tag in &ingredient.tags {
                            let tag_button = gtk::Button::with_label(tag);
                            tag_button.add_css_class("tag");
                            tags_box.append(&tag_button);
                        }
                        
                        details_container.append(&tags_box);
                    }
                    
                    // Pantry information (quantity, etc.)
                    if let Some(pantry) = dm.get_pantry() {
                        if let Some(pantry_item) = pantry.items.iter().find(|item| item.ingredient == ingredient.name) {
                            let stock_label = gtk::Label::new(None);
                            stock_label.set_margin_top(10);
                            
                            if let (Some(quantity), Some(quantity_type)) = (pantry_item.quantity, &pantry_item.quantity_type) {
                                stock_label.set_markup(&format!("<b>In stock:</b> {} {}", quantity, quantity_type));
                            } else if let (Some(quantity), None) = (pantry_item.quantity, &pantry_item.quantity_type) {
                                stock_label.set_markup(&format!("<b>In stock:</b> {}", quantity));
                            } else {
                                stock_label.set_markup("<b>In stock:</b> Yes (unknown quantity)");
                            }
                            
                            stock_label.set_halign(gtk::Align::Start);
                            details_container.append(&stock_label);
                            
                            let updated_label = gtk::Label::new(None);
                            updated_label.set_markup(&format!("<b>Last updated:</b> {}", pantry_item.last_updated));
                            updated_label.set_halign(gtk::Align::Start);
                            details_container.append(&updated_label);
                        } else {
                            let stock_label = gtk::Label::new(Some("Not in stock"));
                            stock_label.set_halign(gtk::Align::Start);
                            stock_label.set_margin_top(10);
                            details_container.append(&stock_label);
                        }
                    }
                    
                    // Knowledge base info if available
                    if let Some(kb_slug) = &ingredient.kb {
                        if let Some(kb_entry) = dm.get_kb_entry(kb_slug) {
                            let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
                            separator.set_margin_top(10);
                            separator.set_margin_bottom(10);
                            details_container.append(&separator);
                            
                            let kb_label = gtk::Label::new(None);
                            kb_label.set_markup("<b>Knowledge Base:</b>");
                            kb_label.set_halign(gtk::Align::Start);
                            details_container.append(&kb_label);
                            
                            let kb_title = gtk::Label::new(None);
                            kb_title.set_markup(&format!("<i>{}</i>", kb_entry.title));
                            kb_title.set_halign(gtk::Align::Start);
                            details_container.append(&kb_title);
                            
                            // We could add a preview of the KB content here
                        }
                    }
                    
                    widgets.pantry_details.append(&details_container);
                } else {
                    // Ingredient not found
                    let not_found_label = gtk::Label::new(Some(&format!("Ingredient '{}' not found", ingredient_name)));
                    not_found_label.set_halign(gtk::Align::Center);
                    not_found_label.set_valign(gtk::Align::Center);
                    widgets.pantry_details.append(&not_found_label);
                }
            } else {
                // Data manager not available
                let error_label = gtk::Label::new(Some("Unable to load ingredient: data manager not available"));
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
        if self.current_tab == Tab::Recipes {
            if let Some(recipe_name) = &self.selected_recipe {
                // Clear previous content
                while let Some(child) = widgets.recipes_details.first_child() {
                    widgets.recipes_details.remove(&child);
                }

                // Find the selected recipe in the data manager
                if let Some(ref dm) = self.data_manager {
                    if let Some(recipe) = dm.get_recipe(recipe_name) {
                        // Create new widgets to display recipe details
                        let recipe_details_scroll = gtk::ScrolledWindow::new();
                        recipe_details_scroll.set_hexpand(true);
                        recipe_details_scroll.set_vexpand(true);

                        let details_container = gtk::Box::new(gtk::Orientation::Vertical, 10);
                        details_container.set_margin_all(10);

                        // Title
                        let title = gtk::Label::new(None);
                        title.set_markup(&format!("<span size='x-large' weight='bold'>{}</span>", recipe.title));
                        title.set_halign(gtk::Align::Start);
                        title.set_margin_bottom(10);
                        details_container.append(&title);

                        // Metadata section
                        let metadata_box = gtk::Box::new(gtk::Orientation::Horizontal, 15);
                        metadata_box.set_margin_bottom(10);

                        // Preparation time
                        let prep_time = gtk::Label::new(None);
                        prep_time.set_markup(&format!("<b>Prep:</b> {} min", recipe.prep_time.unwrap_or(0)));
                        prep_time.set_halign(gtk::Align::Start);

                        // Cooking/downtime
                        let cook_time = gtk::Label::new(None);
                        cook_time.set_markup(&format!("<b>Cook:</b> {} min", recipe.downtime.unwrap_or(0)));
                        cook_time.set_halign(gtk::Align::Start);

                        // Servings
                        let servings = gtk::Label::new(None);
                        servings.set_markup(&format!("<b>Servings:</b> {}", recipe.servings.unwrap_or(0)));
                        servings.set_halign(gtk::Align::Start);

                        metadata_box.append(&prep_time);
                        metadata_box.append(&cook_time);
                        metadata_box.append(&servings);

                        details_container.append(&metadata_box);

                        // Tags
                        if let Some(tags) = &recipe.tags {
                            if !tags.is_empty() {
                                let tags_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
                                tags_box.set_margin_bottom(10);

                                let tags_label = gtk::Label::new(Some("Tags:"));
                                tags_label.set_halign(gtk::Align::Start);
                                tags_box.append(&tags_label);

                                for tag in tags {
                                    let tag_button = gtk::Button::with_label(tag);
                                    tag_button.add_css_class("tag");
                                    tags_box.append(&tag_button);
                                }

                                details_container.append(&tags_box);
                            }
                        }

                        // Separator
                        let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
                        separator.set_margin_top(5);
                        separator.set_margin_bottom(15);
                        details_container.append(&separator);

                        // Ingredients section
                        let ingredients_label = gtk::Label::new(None);
                        ingredients_label.set_markup("<span size='large' weight='bold'>Ingredients</span>");
                        ingredients_label.set_halign(gtk::Align::Start);
                        ingredients_label.set_margin_bottom(5);
                        details_container.append(&ingredients_label);

                        let ingredients_list = gtk::Box::new(gtk::Orientation::Vertical, 5);
                        ingredients_list.set_margin_start(10);
                        ingredients_list.set_margin_bottom(15);

                        for ingredient in &recipe.ingredients {
                            let ingredient_label = gtk::Label::new(None);
                                                
                            let quantity_text = match (&ingredient.quantity, &ingredient.quantity_type) {
                                (Some(q), Some(t)) => format!("{} {} ", q, t),
                                (Some(q), None) => format!("{} ", q),
                                (_, _) => String::new(),
                            };
                        
                            ingredient_label.set_markup(&format!("• {}{}", quantity_text, ingredient.ingredient));
                            ingredient_label.set_halign(gtk::Align::Start);
                            ingredients_list.append(&ingredient_label);
                        }

                        details_container.append(&ingredients_list);

                        // Instructions section
                        let instructions_label = gtk::Label::new(None);
                        instructions_label.set_markup("<span size='large' weight='bold'>Instructions</span>");
                        instructions_label.set_halign(gtk::Align::Start);
                        instructions_label.set_margin_bottom(5);
                        details_container.append(&instructions_label);

                        let instructions_text = gtk::Label::new(Some(&recipe.instructions));
                        instructions_text.set_halign(gtk::Align::Start);
                        instructions_text.set_wrap(true);
                        instructions_text.set_xalign(0.0);
                        instructions_text.set_margin_start(10);
                        details_container.append(&instructions_text);

                        recipe_details_scroll.set_child(Some(&details_container));
                        widgets.recipes_details.append(&recipe_details_scroll);
                    } else {
                        // Recipe not found
                        let not_found_label = gtk::Label::new(Some(&format!("Recipe '{}' not found", recipe_name)));
                        not_found_label.set_halign(gtk::Align::Center);
                        not_found_label.set_valign(gtk::Align::Center);
                        widgets.recipes_details.append(&not_found_label);
                    }
                } else {
                    // Data manager not available
                    let error_label = gtk::Label::new(Some("Unable to load recipe: data manager not available"));
                    error_label.set_halign(gtk::Align::Center);
                    error_label.set_valign(gtk::Align::Center);
                    widgets.recipes_details.append(&error_label);
                }
            } else {
                // No recipe selected
                let select_label = gtk::Label::new(Some("Select a recipe to view details"));
                select_label.set_halign(gtk::Align::Center);
                select_label.set_valign(gtk::Align::Center);

                // Clear previous content
                while let Some(child) = widgets.recipes_details.first_child() {
                    widgets.recipes_details.remove(&child);
                }

                widgets.recipes_details.append(&select_label);
            }
        }
        
        // Handle About dialog
        if self.show_about_dialog {
            // Create and show the dialog
            let about_dialog = gtk::AboutDialog::builder()
                .program_name("Cookbook")
                .version("0.1.0")
                .copyright("© 2025 Cookbook Team")
                .comments("A cross-platform recipe and pantry manager")
                .website("https://github.com/cookbook")
                .website_label("GitHub Repository")
                .license("MIT License")
                .transient_for(&widgets.window)
                .build();
            
            // Reset the flag immediately after creating the dialog
            sender.input(AppMsg::ResetDialogs);
            
            // Also reset when dialog is hidden as a safety measure
            let sender_clone = sender.clone();
            about_dialog.connect_hide(move |_| {
                sender_clone.input(AppMsg::ResetDialogs);
            });
            
            about_dialog.present();
        }
        
        // Handle Help dialog
        if self.show_help_dialog {
            // Create and show the dialog
            let help_dialog = gtk::MessageDialog::builder()
                .title("Cookbook Help")
                .text("Help documentation will be implemented in a future version.")
                .modal(true)
                .buttons(gtk::ButtonsType::Ok)
                .transient_for(&widgets.window)
                .build();
            
            // Reset the flag immediately after creating the dialog
            sender.input(AppMsg::ResetDialogs);
            
            // Also reset when dialog is closed as a safety measure
            let sender_clone = sender.clone();
            help_dialog.connect_response(move |dialog, _| {
                dialog.close();
                sender_clone.input(AppMsg::ResetDialogs);
            });
            
            help_dialog.present();
        }

        // Rebuild pantry list when filters change or search text changes
        if self.current_tab == Tab::Pantry {
            // Step 1: Rebuild the pantry list with filtered items
            
            // Clear current list
            while let Some(child) = widgets.pantry_list.first_child() {
                widgets.pantry_list.remove(&child);
            }
            
            if let Some(ref dm) = self.data_manager {
                let pantry = dm.get_pantry();
                
                // Group ingredients by category
                let mut pantry_items_by_category: std::collections::HashMap<String, Vec<(String, Option<String>, Option<String>, bool)>> = std::collections::HashMap::new();
                
                for ingredient in dm.get_all_ingredients() {
                    let is_in_stock = dm.is_in_pantry(&ingredient.name);
                    
                    // Apply filters
                    if self.show_in_stock_only && !is_in_stock {
                        continue; // Skip items not in stock if filter is active
                    }
                    
                    if !self.selected_pantry_categories.is_empty() && !self.selected_pantry_categories.contains(&ingredient.category) {
                        continue; // Skip items not in selected categories
                    }
                    
                    // Apply search filter if any
                    if !self.search_text.is_empty() && !ingredient.name.to_lowercase().contains(&self.search_text.to_lowercase()) {
                        continue; // Skip items not matching search
                    }
                    
                    // Get quantity information if in pantry
                    let (quantity, quantity_type) = if let Some(pantry_item) = dm.get_pantry_item(&ingredient.name) {
                        (pantry_item.quantity.map(|q| q.to_string()), pantry_item.quantity_type.clone())
                    } else {
                        (None, None)
                    };
                    
                    pantry_items_by_category
                        .entry(ingredient.category.clone())
                        .or_default()
                        .push((ingredient.name.clone(), quantity, quantity_type, is_in_stock));
                }
                
                // Sort categories and ingredients
                let mut sorted_categories: Vec<String> = pantry_items_by_category.keys().cloned().collect();
                sorted_categories.sort();
                
                if pantry_items_by_category.is_empty() {
                    // No items match the filters
                    let no_items_label = gtk::Label::new(Some("No ingredients match the current filters"));
                    no_items_label.set_margin_all(20);
                    widgets.pantry_list.append(&no_items_label);
                } else {
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
                                if let (Some(q), Some(t)) = (quantity, quantity_type) {
                                    label_text = format!("{} ({} {})", name, q, t);
                                } else if let (Some(q), None) = (quantity, quantity_type) {
                                    label_text = format!("{} ({})", name, q);
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
                                
                                // Highlight if selected
                                if let Some(selected) = &self.selected_ingredient {
                                    if selected == name {
                                        item_row.add_css_class("selected");
                                    }
                                }
                                
                                let sender_clone = sender.clone();
                                let name_clone = name.clone();
                                click_gesture.connect_pressed(move |_, _, _, _| {
                                    sender_clone.input(AppMsg::SelectIngredient(name_clone.clone()));
                                });
                                
                                category_box.append(&item_row);
                            }
                        }
                        
                        widgets.pantry_list.append(&category_frame);
                    }
                }
            } else {
                // No data available
                let no_data_label = gtk::Label::new(Some("No ingredient data available"));
                no_data_label.set_margin_all(10);
                widgets.pantry_list.append(&no_data_label);
            }
        }
    }
}
// The main function initializes the GTK application and runs the app
fn main() {
    let app = RelmApp::new("org.cookbook.CookbookGtk");
    app.run::<AppModel>(());
}
