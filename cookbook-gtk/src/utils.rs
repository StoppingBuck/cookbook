use gtk4::prelude::*;
use gtk4::{ResponseType, MessageDialog, MessageType, ButtonsType, Window};

pub fn show_error_dialog(parent: Option<&Window>, title: &str, message: &str) {
    let dialog = MessageDialog::builder()
        .title(title)
        .text(message)
        .modal(true)
        .message_type(MessageType::Error)
        .buttons(ButtonsType::Close)
        .build();
    
    if let Some(parent) = parent {
        dialog.set_transient_for(Some(parent));
    }
    
    dialog.connect_response(|dialog, _| {
        dialog.destroy();
    });
    
    dialog.present();
}

pub fn show_confirmation_dialog<F: FnOnce() + 'static>(
    parent: Option<&Window>, 
    title: &str, 
    message: &str, 
    confirm_label: &str,
    on_confirm: F
) {
    let dialog = MessageDialog::builder()
        .title(title)
        .text(message)
        .modal(true)
        .message_type(MessageType::Question)
        .buttons(ButtonsType::None)
        .build();
    
    dialog.add_button("Cancel", ResponseType::Cancel);
    dialog.add_button(confirm_label, ResponseType::Accept);
    dialog.set_default_response(ResponseType::Cancel);
    
    if let Some(parent) = parent {
        dialog.set_transient_for(Some(parent));
    }
    
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            on_confirm();
        }
        dialog.destroy();
    });
    
    dialog.present();
}
