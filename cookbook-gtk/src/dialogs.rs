use gtk::prelude::*;
use relm4::gtk;
use relm4::ComponentSender;

/// Shows the About dialog for the Cookbook application.
///
/// # Arguments
/// * `parent_window` - The parent window for the dialog
/// * `sender` - The component sender to use for sending messages
/// * `reset_message` - The message to send when the dialog should be closed
pub fn show_about_dialog<Msg, C>(
    parent_window: &gtk::ApplicationWindow,
    sender: &ComponentSender<C>,
    reset_message: Msg,
) where
    C: relm4::Component<Input = Msg>,
    Msg: Clone + 'static,
{
    // Find the absolute path to the app icon for the About dialog
    use std::path::PathBuf;
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    // Use the project source directory to find the app icon during development
    let icon_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/app_icon.png");
    eprintln!("[AboutDialog DEBUG] exe_dir: {:?}", exe_dir);
    eprintln!("[AboutDialog DEBUG] icon_path: {:?}", icon_path);
    let logo_pixbuf = match gdk_pixbuf::Pixbuf::from_file(&icon_path) {
        Ok(pixbuf) => {
            eprintln!(
                "[AboutDialog DEBUG] Successfully loaded Pixbuf: {:?}",
                icon_path
            );
            Some(pixbuf)
        }
        Err(err) => {
            eprintln!(
                "[AboutDialog DEBUG] Failed to load Pixbuf: {:?} | Error: {}",
                icon_path, err
            );
            None
        }
    };
    let logo_image = logo_pixbuf
        .as_ref()
        .map(|pixbuf| gtk::gdk::Texture::for_pixbuf(pixbuf));

    let about_dialog = gtk::AboutDialog::builder()
        .program_name("Cookbook")
        .version("0.1.0")
        .copyright("© 2025 Mads Peter Rommedahl")
        .comments("A cross-platform recipe and pantry manager")
        .website("https://github.com/StoppingBuck/cookbook")
        .website_label("GitHub Repository")
        .license("MIT OR Apache-2.0")
        .license_type(gtk::License::MitX11)
        .transient_for(parent_window)
        .build();

    if let Some(texture) = logo_image {
        eprintln!("[AboutDialog DEBUG] Setting logo texture");
        about_dialog.set_logo(Some(&texture));
    } else {
        eprintln!("[AboutDialog DEBUG] No logo texture to set");
    }

    // Reset the flag immediately after creating the dialog
    sender.input(reset_message.clone());

    // Also reset when dialog is hidden as a safety measure
    let sender_clone = sender.clone();
    let reset_message_clone = reset_message.clone();
    about_dialog.connect_hide(move |_| {
        sender_clone.input(reset_message_clone.clone());
    });

    about_dialog.present();
}

/// Shows the Help dialog for the Cookbook application.
///
/// # Arguments
/// * `parent_window` - The parent window for the dialog
/// * `sender` - The component sender to use for sending messages
/// * `reset_message` - The message to send when the dialog should be closed
pub fn show_help_dialog<Msg, C>(
    parent_window: &gtk::ApplicationWindow,
    sender: &ComponentSender<C>,
    reset_message: Msg,
) where
    C: relm4::Component<Input = Msg>,
    Msg: Clone + 'static,
{
    // Create and show the dialog
    let help_dialog = gtk::MessageDialog::builder()
        .title("Cookbook Help")
        .text("Help documentation will be implemented in a future version.")
        .modal(true)
        .buttons(gtk::ButtonsType::Ok)
        .transient_for(parent_window)
        .build();

    // Reset the flag immediately after creating the dialog
    sender.input(reset_message.clone());

    // Also reset when dialog is closed as a safety measure
    let sender_clone = sender.clone();
    let reset_message_clone = reset_message.clone();
    help_dialog.connect_response(move |dialog, _| {
        dialog.close();
        sender_clone.input(reset_message_clone.clone());
    });

    help_dialog.present();
}
/// Shows an error dialog with the given message.
pub fn show_error_dialog(parent: &gtk::ApplicationWindow, message: &str) {
    let dialog = gtk::MessageDialog::builder()
        .modal(true)
        .buttons(gtk::ButtonsType::Ok)
        .message_type(gtk::MessageType::Error)
        .text(message)
        .transient_for(parent)
        .build();

    dialog.connect_response(|dialog, _| dialog.close());
    dialog.present();
}
