pub mod app;
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

pub use app::build_app_model_and_widgets;
pub use types::{AppModel, AppMsg, AppWidgets, Tab};
