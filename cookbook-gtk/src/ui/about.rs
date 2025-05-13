use gtk4::prelude::*;
use gtk4::{AboutDialog, License};

pub fn show_about_dialog() {
    let dialog = AboutDialog::builder()
        .title("About Cookbook")
        .program_name("Cookbook")
        .version("0.1.0")
        .logo_icon_name("applications-cooking")
        .comments("A cross-platform recipe and pantry manager")
        .website("https://github.com/cookbook")
        .website_label("GitHub Repository")
        .license_type(License::Mit)
        .copyright("© 2025 Cookbook Team")
        .authors(vec!["Cookbook Team".to_string()])
        .build();
    
    dialog.present();
}
