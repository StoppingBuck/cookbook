pub mod detail;
pub mod edit;
pub mod list;

#[allow(unused_imports)]
pub use detail::{build_recipe_detail_view, update_recipe_details};
pub use edit::{show_add_recipe_dialog, show_edit_recipe_dialog};
pub use list::{refresh_recipes_ui, update_recipes_list};

use crate::types::AppModel;
use crate::types::AppMsg;
use crate::ui_constants::*;
use gtk::prelude::*;
use relm4::gtk;
use relm4::ComponentSender;

/// Builds the Recipes tab UI and returns the main container, list box, and details box.
pub fn build_recipes_tab(
    model: &AppModel,
    sender: Option<ComponentSender<AppModel>>,
) -> (gtk::Box, gtk::ListBox, gtk::Box) {
    // Recipes Tab UI Structure:
    // - Recipe List Pane (middle): shows all recipes
    // - Recipe Details Pane (right): shows details for selected recipe
    // - Navigation Pane (left): handled by main app sidebar, not here
    // The panes are uncoupled except:
    //   - Selecting a recipe in the List Pane updates the Details Pane
    //   - Changing tab in Navigation triggers List Pane update

    let recipes_container = gtk::Box::new(gtk::Orientation::Vertical, SECTION_SPACING);

    // Header: title + search
    let recipes_header = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
    recipes_header.set_margin_top(DEFAULT_MARGIN);
    recipes_header.set_margin_bottom(DEFAULT_MARGIN);
    recipes_header.set_margin_start(DEFAULT_MARGIN);
    recipes_header.set_margin_end(DEFAULT_MARGIN);

    let recipes_title = gtk::Label::new(Some("Recipes"));
    recipes_title.set_markup("<span size='x-large' weight='bold'>Recipes</span>");
    recipes_title.set_halign(gtk::Align::Start);
    recipes_title.set_hexpand(true);

    let search_entry = gtk::SearchEntry::new();
    search_entry.set_placeholder_text(Some("Search recipes..."));

    let sender_clone = sender.clone();
    search_entry.connect_search_changed(move |entry| {
        if let Some(sender) = &sender_clone {
            sender.input(AppMsg::SearchTextChanged(entry.text().to_string()));
        }
    });

    // Add Recipe button
    let add_recipe_button = gtk::Button::with_label("Add Recipe");
    add_recipe_button.add_css_class("suggested-action");
    let sender_clone = sender.clone();
    add_recipe_button.connect_clicked(move |_| {
        if let Some(sender) = &sender_clone {
            sender.input(AppMsg::AddRecipe);
        }
    });

    recipes_header.append(&recipes_title);
    recipes_header.append(&search_entry);
    recipes_header.append(&add_recipe_button);
    recipes_container.append(&recipes_header);

    // Split view: Recipe List Pane (middle), Recipe Details Pane (right)
    let recipes_content = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
    recipes_content.set_hexpand(true);
    recipes_content.set_vexpand(true);

    // Recipe List Pane
    let recipe_list_pane_scroll = gtk::ScrolledWindow::new();
    recipe_list_pane_scroll.set_hexpand(false);
    recipe_list_pane_scroll.set_vexpand(true);
    recipe_list_pane_scroll.set_min_content_width(250);

    let recipe_list_pane = gtk::ListBox::new();
    recipe_list_pane.set_selection_mode(gtk::SelectionMode::Single);

    // Populate the Recipe List Pane
    if let Some(ref sender) = sender {
        crate::recipes::update_recipes_list(
            &recipe_list_pane,
            &model.data_manager,
            &model.search_text,
            model.selected_recipe.as_ref(),
            Some(sender),
            |title| AppMsg::SelectRecipe(title),
        );
    }

    // Recipe List Pane selection handler
    let sender_clone = sender.clone();
    recipe_list_pane.connect_row_selected(move |_list, row_opt| {
        if let Some(row) = row_opt {
            if let Some(box_layout) = row.child().and_then(|w| w.downcast::<gtk::Box>().ok()) {
                if let Some(label) = box_layout
                    .first_child()
                    .and_then(|w| w.downcast::<gtk::Label>().ok())
                {
                    let recipe_title = label.text().to_string();
                    // Selecting a recipe in the Recipe List Pane updates the Recipe Details Pane
                    // Prevent feedback loop: only send if model's selected_recipe is different
                    // NOTE: This closure should capture the selected_recipe value from the model at connect time
                    // If sender_clone is a ComponentSender<AppModel>, we can use its get() method
                    // But safest is to use a RefCell to share state, or pass selected_recipe in as an argument
                    // For now, we skip sending if the row is already selected
                    // (update_view will keep the selection in sync)
                    // So do nothing here; update_view will handle selection
                }
            }
        }
    });

    recipe_list_pane_scroll.set_child(Some(&recipe_list_pane));

    // Recipe Details Pane
    let recipe_details_pane = gtk::Box::new(gtk::Orientation::Vertical, SECTION_SPACING);
    recipe_details_pane.set_hexpand(true);
    recipe_details_pane.set_vexpand(true);

    let details_label = gtk::Label::new(Some("Select a recipe to view details"));
    details_label.set_halign(gtk::Align::Center);
    details_label.set_valign(gtk::Align::Center);
    details_label.set_hexpand(true);
    details_label.set_vexpand(true);

    recipe_details_pane.append(&details_label);

    recipes_content.append(&recipe_list_pane_scroll);
    recipes_content.append(&recipe_details_pane);

    recipes_container.append(&recipes_content);

    // Return the main container, Recipe List Pane, and Recipe Details Pane
    (recipes_container, recipe_list_pane, recipe_details_pane)
}
