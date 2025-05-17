use relm4::gtk;
use std::path::PathBuf;
use std::rc::Rc;
use cookbook_engine::DataManager;

/// Represents the different tabs in the application UI
#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Recipes,
    Pantry,
    KnowledgeBase,
    Settings,
}

/// Messages that can be sent within the application to trigger state changes
#[derive(Debug, Clone)]
pub enum AppMsg {
    SwitchTab(Tab),
    ShowAbout,
    ShowHelp,
    ResetDialogs,
    SelectRecipe(String),
    SelectIngredient(String),
    SelectKnowledgeBaseEntry(String),
    ToggleInStockFilter(bool),
    SearchTextChanged(String),
    EditIngredient(String),
    EditRecipe(String),
    TogglePantryCategory(String, bool),
    UpdateIngredientWithPantry(
        String,
        cookbook_engine::Ingredient,
        Option<f64>,
        Option<String>,
        bool, // remove_from_pantry
    ), // (original_name, new_ingredient, quantity, quantity_type, remove_from_pantry)
    UpdateRecipe(String, cookbook_engine::Recipe), // (original_title, new_recipe)
    ClearError,
    AddIngredient, // Triggers the add ingredient dialog
    ReloadPantry, // Explicitly reload pantry data and UI
}

/// The main application model representing the application state
#[allow(dead_code)]
pub struct AppModel {
    pub data_manager: Option<Rc<DataManager>>,
    pub data_dir: PathBuf,
    pub current_tab: Tab,
    pub selected_recipe: Option<String>,
    pub selected_ingredient: Option<String>,
    pub selected_kb_entry: Option<String>,
    pub search_text: String,
    pub show_about_dialog: bool,
    pub show_help_dialog: bool,
    pub selected_pantry_categories: Vec<String>,
    pub show_in_stock_only: bool,
    pub error_message: Option<String>,
}

/// References to GTK widgets used in the application
#[allow(dead_code)]
pub struct AppWidgets {
    pub window: gtk::ApplicationWindow,
    pub main_stack: gtk::Stack,
    //pub recipes_label: gtk::Label,
    pub recipes_details: gtk::Box,
    pub recipes_list_box: gtk::ListBox,
    pub pantry_label: gtk::Label,
    pub pantry_list: gtk::Box,
    pub pantry_details: gtk::Box,
    pub pantry_category_filters: gtk::Box,
    pub pantry_in_stock_switch: gtk::Switch,
    pub kb_label: gtk::Label,
    pub kb_list_box: gtk::ListBox,
    pub kb_details: gtk::Box,
    //pub settings_label: gtk::Label,
    pub sidebar_buttons: Vec<gtk::Button>,
}