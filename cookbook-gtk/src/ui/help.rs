use gtk4::prelude::*;
use gtk4::{Dialog, ResponseType, Box, Orientation, Label, Window};

pub fn show_help_dialog() {
    let dialog = Dialog::builder()
        .title("Cookbook Help")
        .default_width(600)
        .default_height(400)
        .modal(true)
        .build();
    
    dialog.add_button("Close", ResponseType::Close);
    
    let content_area = dialog.content_area();
    content_area.set_spacing(15);
    content_area.set_margin_all(20);
    
    // Add title
    let title = Label::new(None);
    title.set_markup("<span size='xx-large'>Cookbook Help</span>");
    content_area.append(&title);
    
    // Add scrollable content area
    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .hexpand(true)
        .vexpand(true)
        .build();
    
    let content_box = Box::new(Orientation::Vertical, 10);
    
    // Add help sections
    add_help_section(&content_box, "Getting Started", 
        "Cookbook is a recipe and pantry manager application that helps you organize your \
        recipes, track ingredients in your pantry, and learn about different ingredients \
        through the Knowledge Base.");
    
    add_help_section(&content_box, "Recipes", 
        "The Recipes section allows you to browse, search, and manage your recipes. \
        Click on a recipe in the list to view its details, including ingredients, \
        preparation time, and cooking instructions.");
    
    add_help_section(&content_box, "Pantry", 
        "The Pantry section helps you keep track of ingredients you have on hand. \
        You can add new items, update quantities, and see details about each ingredient.");
    
    add_help_section(&content_box, "Knowledge Base", 
        "The Knowledge Base provides information about various ingredients, including \
        nutrition facts, history, and cooking tips. Browse through the entries to learn \
        more about the food you cook with.");
    
    add_help_section(&content_box, "Settings", 
        "In the Settings section, you can customize the application to your preferences. \
        You can change the theme between light and dark modes, adjust the font size, \
        and select a different data directory.");
    
    add_help_section(&content_box, "Frequently Asked Questions", 
        "Q: How do I add a new recipe?\n\
        A: Click the \"Add Recipe\" button in the Recipes section.\n\n\
        Q: Can I export my recipes?\n\
        A: This feature is coming soon in a future version.\n\n\
        Q: How do I update the quantity of an item in my pantry?\n\
        A: Select the item in your pantry and click \"Edit\".");
    
    add_help_section(&content_box, "Keyboard Shortcuts", 
        "Ctrl+F: Search\n\
        Ctrl+N: New Recipe\n\
        Ctrl+S: Save Changes\n\
        F1: Show Help");
    
    scrolled.set_child(Some(&content_box));
    content_area.append(&scrolled);
    
    dialog.connect_response(|dialog, response| {
        if response == ResponseType::Close {
            dialog.destroy();
        }
    });
    
    dialog.present();
}

fn add_help_section(container: &Box, title: &str, content: &str) {
    let section_box = Box::new(Orientation::Vertical, 5);
    
    let title_label = Label::new(None);
    title_label.set_markup(&format!("<b>{}</b>", title));
    title_label.set_xalign(0.0);
    
    let content_label = Label::new(Some(content));
    content_label.set_xalign(0.0);
    content_label.set_yalign(0.0);
    content_label.set_wrap(true);
    content_label.set_selectable(true);
    
    section_box.append(&title_label);
    section_box.append(&content_label);
    
    // Add separator
    let separator = gtk4::Separator::new(Orientation::Horizontal);
    separator.set_margin_top(10);
    separator.set_margin_bottom(10);
    
    container.append(&section_box);
    container.append(&separator);
}
