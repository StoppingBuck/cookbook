mod app;
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

use relm4::RelmApp;
use types::AppModel;

fn main() {
    env_logger::init();
    let app = RelmApp::new("org.cookbook.CookbookGtk");
    app.run::<AppModel>(());
}
