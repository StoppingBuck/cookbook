use relm4::RelmApp;

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    RelmApp::new("dev.cookbook.Cookbook").run::<cookbook_gtk::app::App>(());
}
