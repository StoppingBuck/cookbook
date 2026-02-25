use crate::types::AppModel;
use crate::types::AppMsg;
use crate::ui_constants::*;
use crate::utils;
use cookbook_engine::DataManager;
use gtk::prelude::*;
use relm4::gtk;
use relm4::ComponentSender;
use relm4::RelmWidgetExt;
use std::rc::Rc;

/// Builds and returns the Recipe Details Pane for a selected recipe
pub fn build_recipe_detail_view<C>(
    data_manager: &Rc<DataManager>,
    recipe_name: &str,
    sender: Option<&ComponentSender<C>>,
    edit_recipe_msg: impl Fn(String) -> C::Input + Clone + 'static,
) -> gtk::ScrolledWindow
where
    C: relm4::Component,
{
    let recipe_details_scroll = gtk::ScrolledWindow::new();
    recipe_details_scroll.set_vexpand(true);
    recipe_details_scroll.set_hexpand(true);

    if let Some(recipe) = data_manager.get_recipe(recipe_name) {
        let recipe_box = gtk::Box::new(gtk::Orientation::Vertical, SECTION_SPACING);
        recipe_box.set_margin_all(DETAILS_MARGIN);

        // IMAGE SECTION
        let image_frame = gtk::Frame::new(None);
        image_frame.set_margin_bottom(HEADER_MARGIN);
        image_frame.set_hexpand(true);
        image_frame.set_vexpand(false);
        let image_widget = if let Some(image_name) = &recipe.image {
            let data_dir = data_manager.get_data_dir();
            let img_path = data_dir.join("recipes/img").join(image_name);
            if img_path.exists() {
                let img = gtk::Image::from_file(img_path);
                img.set_pixel_size(200);
                img.set_halign(gtk::Align::Center);
                img.set_valign(gtk::Align::Start);
                img.upcast::<gtk::Widget>()
            } else {
                let missing = gtk::Label::new(Some("No image available"));
                missing.set_halign(gtk::Align::Center);
                missing.upcast::<gtk::Widget>()
            }
        } else {
            let missing = gtk::Label::new(Some("No image available"));
            missing.set_halign(gtk::Align::Center);
            missing.upcast::<gtk::Widget>()
        };
        image_frame.set_child(Some(&image_widget));
        recipe_box.append(&image_frame);

        // Header with title and edit button
        let header_box = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);

        let title_label = gtk::Label::new(None);
        title_label.set_markup(&format!(
            "<span size='x-large' weight='bold'>{}</span>",
            recipe.title
        ));
        title_label.set_halign(gtk::Align::Start);
        title_label.set_hexpand(true);
        header_box.append(&title_label);

        // Edit button
        let edit_button = gtk::Button::with_label("Edit");
        edit_button.add_css_class("suggested-action");
        if let Some(sender) = sender {
            let sender_clone = sender.clone();
            let recipe_title = recipe.title.clone();
            let edit_msg = edit_recipe_msg.clone();
            edit_button.connect_clicked(move |_| {
                sender_clone.input(edit_msg(recipe_title.clone()));
            });
        }
        header_box.append(&edit_button);

        // Delete button
        let delete_button = gtk::Button::with_label("Delete");
        delete_button.add_css_class("destructive-action");
        if let Some(sender) = sender {
            let sender_clone = sender.clone();
            let recipe_title = recipe.title.clone();
            delete_button.connect_clicked(move |_| {
                // Only send if C::Input == AppMsg
                if let Some(appmsg_sender) = (&sender_clone as &dyn std::any::Any)
                    .downcast_ref::<ComponentSender<AppModel>>()
                {
                    appmsg_sender.input(AppMsg::DeleteRecipe(recipe_title.clone()));
                }
            });
        }
        header_box.append(&delete_button);

        recipe_box.append(&header_box);

        // Recipe tags
        if let Some(ref tags) = recipe.tags {
            if !tags.is_empty() {
                let tags_box = gtk::Box::new(gtk::Orientation::Horizontal, TAG_SPACING);
                tags_box.set_margin_bottom(DEFAULT_MARGIN);

                for tag in tags {
                    let tag_button = gtk::Button::with_label(tag);
                    tag_button.add_css_class("tag");
                    tags_box.append(&tag_button);
                }

                recipe_box.append(&tags_box);
            }
        }

        // Recipe metadata: prep time, downtime, servings
        let metadata_grid = gtk::Grid::new();
        metadata_grid.set_column_spacing(DEFAULT_MARGIN.try_into().unwrap_or(0));
        metadata_grid.set_row_spacing(TAG_SPACING.try_into().unwrap_or(0));
        metadata_grid.set_margin_bottom(SECTION_SPACING); // Space after the grid

        // Prep time
        let prep_label = gtk::Label::new(None);
        prep_label.set_markup("<b>Prep Time:</b>");
        prep_label.set_halign(gtk::Align::Start);
        metadata_grid.attach(&prep_label, 0, 0, 1, 1);

        let prep_value = gtk::Label::new(Some(&format!("{} min", recipe.prep_time.unwrap_or(0))));
        prep_value.set_halign(gtk::Align::Start);
        metadata_grid.attach(&prep_value, 1, 0, 1, 1);

        // Downtime
        let downtime_label = gtk::Label::new(None);
        downtime_label.set_markup("<b>Cook Time:</b>");
        downtime_label.set_halign(gtk::Align::Start);
        metadata_grid.attach(&downtime_label, 0, 1, 1, 1);

        let downtime_value =
            gtk::Label::new(Some(&format!("{} min", recipe.downtime.unwrap_or(0))));
        downtime_value.set_halign(gtk::Align::Start);
        metadata_grid.attach(&downtime_value, 1, 1, 1, 1);

        // Servings
        let servings_label = gtk::Label::new(None);
        servings_label.set_markup("<b>Servings:</b>");
        servings_label.set_halign(gtk::Align::Start);
        metadata_grid.attach(&servings_label, 2, 0, 1, 1);

        let servings_value = gtk::Label::new(Some(&format!("{}", recipe.servings.unwrap_or(0))));
        servings_value.set_halign(gtk::Align::Start);
        metadata_grid.attach(&servings_value, 3, 0, 1, 1);

        // Total time
        let total_label = gtk::Label::new(None);
        total_label.set_markup("<b>Total Time:</b>");
        total_label.set_halign(gtk::Align::Start);
        metadata_grid.attach(&total_label, 2, 1, 1, 1);

        let total_time = recipe.prep_time.unwrap_or(0) + recipe.downtime.unwrap_or(0);
        let total_value = gtk::Label::new(Some(&format!("{} min", total_time)));
        total_value.set_halign(gtk::Align::Start);
        metadata_grid.attach(&total_value, 3, 1, 1, 1);

        recipe_box.append(&metadata_grid);

        // Ingredients section
        let ingredients_header = gtk::Label::new(None);
        ingredients_header.set_markup("<span size='large' weight='bold'>Ingredients</span>");
        ingredients_header.set_halign(gtk::Align::Start);
        ingredients_header.set_margin_bottom(LIST_ROW_MARGIN);
        recipe_box.append(&ingredients_header);

        // Ingredients list in a frame
        let ingredients_frame = gtk::Frame::new(None);
        ingredients_frame.set_margin_bottom(HEADER_MARGIN);

        let ingredients_list = gtk::Box::new(gtk::Orientation::Vertical, 0);
        ingredients_list.set_margin_all(DEFAULT_MARGIN);

        // Check which ingredients are in pantry
        let pantry_items = data_manager
            .get_pantry()
            .map(|p| {
                p.items
                    .iter()
                    .map(|i| i.ingredient.clone())
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();
        for ingredient in &recipe.ingredients {
            let ingredient_box = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
            ingredient_box.set_margin_bottom(LIST_ROW_MARGIN);

            // Add '(in stock)' text if ingredient is in pantry
            let is_in_pantry = pantry_items.contains(&ingredient.ingredient);
            if is_in_pantry {
                let in_stock_label = gtk::Label::new(Some("(in stock)"));
                in_stock_label.add_css_class("in-stock-label");
                ingredient_box.append(&in_stock_label);
            } else {
                let space_label = gtk::Label::new(Some(" "));
                space_label.set_width_chars(2);
                ingredient_box.append(&space_label);
            }

            // Format the ingredient text
            let ingredient_text = if let Some(ref q) = ingredient.quantity {
                if let Some(ref qt) = ingredient.quantity_type {
                    if !qt.is_empty() {
                        format!("{} {} {}", q, qt, ingredient.ingredient)
                    } else {
                        format!("{} {}", q, ingredient.ingredient)
                    }
                } else {
                    format!("{} {}", q, ingredient.ingredient)
                }
            } else {
                ingredient.ingredient.clone()
            };

            let ingredient_label = gtk::Label::new(Some(&ingredient_text));
            ingredient_label.set_halign(gtk::Align::Start);
            ingredient_label.set_hexpand(true);

            // Style the label based on pantry status
            if is_in_pantry {
                ingredient_label.add_css_class("ingredient-available");
            } else {
                ingredient_label.add_css_class("ingredient-missing");
            }

            ingredient_box.append(&ingredient_label);
            ingredients_list.append(&ingredient_box);
        }

        ingredients_frame.set_child(Some(&ingredients_list));
        recipe_box.append(&ingredients_frame);

        // Instructions section (for details view, not edit dialog!)
        let instructions_header = gtk::Label::new(None);
        instructions_header.set_markup("<span size='large' weight='bold'>Instructions</span>");
        instructions_header.set_halign(gtk::Align::Start);
        instructions_header.set_margin_bottom(LIST_ROW_MARGIN);
        recipe_box.append(&instructions_header);

        let instructions_frame = gtk::Frame::new(None);
        let instructions_text = gtk::Label::new(Some(&recipe.instructions));
        instructions_text.set_wrap(true);
        instructions_text.set_halign(gtk::Align::Start);
        instructions_text.set_margin_all(DEFAULT_MARGIN);
        instructions_frame.set_child(Some(&instructions_text));
        recipe_box.append(&instructions_frame);

        recipe_details_scroll.set_child(Some(&recipe_box));
    } else {
        // Recipe not found
        let not_found_label = gtk::Label::new(Some(&format!("Recipe '{}' not found", recipe_name)));
        not_found_label.set_halign(gtk::Align::Center);
        not_found_label.set_valign(gtk::Align::Center);
        recipe_details_scroll.set_child(Some(&not_found_label));
    }

    recipe_details_scroll
}

/// Updates the Recipe Details Pane based on the selected recipe in the Recipe List Pane
pub fn update_recipe_details<C>(
    selected_recipe: Option<&str>,
    recipes_details: &gtk::Box,
    data_manager: &Option<std::rc::Rc<cookbook_engine::DataManager>>,
    sender: Option<&ComponentSender<C>>,
    edit_recipe_msg: impl Fn(String) -> C::Input + Clone + 'static,
) where
    C: relm4::Component,
{
    // Clear previous content
    utils::clear_box(&recipes_details);

    if let Some(recipe_name) = selected_recipe {
        // Find the selected recipe in the data manager
        if let Some(ref dm) = data_manager {
            let recipe_details_scroll =
                build_recipe_detail_view(dm, recipe_name, sender, edit_recipe_msg);
            recipes_details.append(&recipe_details_scroll);
        } else {
            // Data manager not available
            let error_label =
                gtk::Label::new(Some("Unable to load recipe: data manager not available"));
            error_label.set_halign(gtk::Align::Center);
            error_label.set_valign(gtk::Align::Center);
            recipes_details.append(&error_label);
        }
    } else {
        // No recipe selected
        let select_label = gtk::Label::new(Some("Select a recipe to view details"));
        select_label.set_halign(gtk::Align::Center);
        select_label.set_valign(gtk::Align::Center);
        recipes_details.append(&select_label);
    }
}
