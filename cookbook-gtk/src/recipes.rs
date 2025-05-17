use cookbook_engine::DataManager;
use gtk::glib;
use gtk::prelude::*;
use relm4::gtk;
use relm4::ComponentSender;
use relm4::RelmWidgetExt;
use std::rc::Rc;
use std::cell::RefCell;

use crate::types::AppModel;
use crate::types::AppMsg;
use crate::ui_constants::*;
use crate::utils;

/// Updates the recipes list based on search text and other filters
pub fn update_recipes_list<C>(
    recipes_list_box: &gtk::ListBox,
    data_manager: &Option<Rc<DataManager>>,
    search_text: &str,
    sender: &ComponentSender<C>,
    select_recipe_msg: impl Fn(String) -> C::Input + Clone + 'static,
) where
    C: relm4::Component,
{
    // Clear the recipes list
    utils::clear_list_box(&recipes_list_box);

    if let Some(ref dm) = data_manager {
        // Use engine method to search recipes
        let filtered_recipes = dm.search_recipes(search_text);

        if !filtered_recipes.is_empty() {
            for recipe in filtered_recipes {
                let row = gtk::ListBoxRow::new();
                let box_layout = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
                box_layout.set_margin_all(LIST_ROW_MARGIN);

                // Create recipe item with title and status icon
                let title_label = gtk::Label::new(Some(&recipe.title));
                title_label.set_halign(gtk::Align::Start);
                title_label.set_hexpand(true);
                box_layout.append(&title_label);

                // Add checkmark if all ingredients are in pantry
                let all_in_stock = dm.are_all_ingredients_in_pantry(&recipe.title);

                // Check if any ingredient in the recipe has unknown quantity in pantry
                let mut any_unknown = false;
                if let Some(pantry) = dm.get_pantry() {
                    if let Some(recipe_obj) = dm.get_recipe(&recipe.title) {
                        for ing in &recipe_obj.ingredients {
                            if let Some(pantry_item) = pantry
                                .items
                                .iter()
                                .find(|item| item.ingredient == ing.ingredient)
                            {
                                if pantry_item.quantity.is_none() {
                                    any_unknown = true;
                                    break;
                                }
                            }
                        }
                    }
                }

                if all_in_stock {
                    let icon = gtk::Image::from_icon_name("emblem-ok-symbolic");
                    icon.set_halign(gtk::Align::End);
                    if any_unknown {
                        // Wrap the icon in a box with parentheses labels
                        let icon_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                        let left_paren = gtk::Label::new(Some("("));
                        let right_paren = gtk::Label::new(Some(")"));
                        icon_box.append(&left_paren);
                        icon_box.append(&icon);
                        icon_box.append(&right_paren);
                        box_layout.append(&icon_box);
                    } else {
                        box_layout.append(&icon);
                    }
                }

                row.set_child(Some(&box_layout));

                // Add click handler
                let sender_clone = sender.clone();
                let recipe_title = recipe.title.clone();
                let select_msg = select_recipe_msg.clone();

                let click_gesture = gtk::GestureClick::new();
                row.add_controller(click_gesture.clone());
                click_gesture.connect_pressed(move |_, _, _, _| {
                    sender_clone.input(select_msg(recipe_title.clone()));
                });

                recipes_list_box.append(&row);
            }
        } else {
            // No recipes match the search
            let no_recipes_row = gtk::ListBoxRow::new();
            let no_recipes_label = if search_text.is_empty() {
                gtk::Label::new(Some("No recipes available"))
            } else {
                gtk::Label::new(Some(&format!("No recipes match '{}'", search_text)))
            };
            no_recipes_label.set_margin_all(DEFAULT_MARGIN);
            no_recipes_row.set_child(Some(&no_recipes_label));
            recipes_list_box.append(&no_recipes_row);
        }
    } else {
        // Data manager not available
        let no_data_row = gtk::ListBoxRow::new();
        let no_data_label = gtk::Label::new(Some("Failed to load recipe data"));
        no_data_label.set_margin_all(DEFAULT_MARGIN);
        no_data_row.set_child(Some(&no_data_label));
        recipes_list_box.append(&no_data_row);
    }
}

/// Builds and returns the recipe detail view for a selected recipe
pub fn build_recipe_detail_view<C>(
    data_manager: &Rc<DataManager>,
    recipe_name: &str,
    sender: &ComponentSender<C>,
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

        let sender_clone = sender.clone();
        let recipe_title = recipe.title.clone();
        let edit_msg = edit_recipe_msg.clone();
        edit_button.connect_clicked(move |_| {
            sender_clone.input(edit_msg(recipe_title.clone()));
        });
        header_box.append(&edit_button);

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
        metadata_grid.set_column_spacing(20);
        metadata_grid.set_row_spacing(5);
        metadata_grid.set_margin_bottom(HEADER_MARGIN);

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

            // Add checkmark if ingredient is in pantry
            let is_in_pantry = pantry_items.contains(&ingredient.ingredient);
            if is_in_pantry {
                let check_icon = gtk::Image::from_icon_name("emblem-ok-symbolic");
                ingredient_box.append(&check_icon);
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

/// Updates the recipe details view based on the selected recipe
pub fn update_recipe_details<C>(
    selected_recipe: Option<&str>,
    recipes_details: &gtk::Box,
    data_manager: &Option<std::rc::Rc<cookbook_engine::DataManager>>,
    sender: &ComponentSender<C>,
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

/// Show the Edit Recipe dialog and handle updating the recipe.
pub fn show_edit_recipe_dialog(
    recipe: &cookbook_engine::Recipe,
    data_manager: Option<Rc<cookbook_engine::DataManager>>,
    sender: ComponentSender<crate::types::AppModel>,
    recipe_title: String,
) {
    let selected_image_path: Rc<RefCell<Option<std::path::PathBuf>>> = Rc::new(RefCell::new(None));

    let dialog = gtk::Dialog::new();
    dialog.set_title(Some(&format!("Edit Recipe: {}", recipe_title)));
    dialog.set_modal(true);
    dialog.set_default_width(700); // Increased from 500
    dialog.set_default_height(800); // Increased from 600

    // Set transient parent to an appropriate application window
    for window in gtk::Window::list_toplevels() {
        if let Some(win) = window.downcast_ref::<gtk::Window>() {
            if win != dialog.upcast_ref::<gtk::Window>() {
                dialog.set_transient_for(Some(win));
                break;
            }
        }
    }

    let content_area = dialog.content_area();
    content_area.set_margin_all(DEFAULT_MARGIN);
    content_area.set_spacing(SECTION_SPACING);

    let scrolled_window = gtk::ScrolledWindow::new();
    scrolled_window.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scrolled_window.set_vexpand(true);

    let form_container = gtk::Box::new(gtk::Orientation::Vertical, SECTION_SPACING);
    form_container.set_margin_all(DEFAULT_MARGIN);

    // Title field
    let title_box = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
    let title_label = gtk::Label::new(Some("Title:"));
    title_label.set_halign(gtk::Align::Start);
    title_label.set_width_chars(12);
    let title_entry = gtk::Entry::new();
    title_entry.set_text(&recipe.title);
    title_entry.set_hexpand(true);
    title_box.append(&title_label);
    title_box.append(&title_entry);

    // Prep time field
    let prep_time_box = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
    let prep_time_label = gtk::Label::new(Some("Prep Time (min):"));
    prep_time_label.set_halign(gtk::Align::Start);
    prep_time_label.set_width_chars(12);
    let prep_time_entry = gtk::Entry::new();
    if let Some(prep_time) = recipe.prep_time {
        prep_time_entry.set_text(&prep_time.to_string());
    }
    prep_time_entry.set_hexpand(true);
    prep_time_box.append(&prep_time_label);
    prep_time_box.append(&prep_time_entry);

    // Downtime field
    let downtime_box = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
    let downtime_label = gtk::Label::new(Some("Cook Time (min):"));
    downtime_label.set_halign(gtk::Align::Start);
    downtime_label.set_width_chars(12);
    let downtime_entry = gtk::Entry::new();
    if let Some(downtime) = recipe.downtime {
        downtime_entry.set_text(&downtime.to_string());
    }
    downtime_entry.set_hexpand(true);
    downtime_box.append(&downtime_label);
    downtime_box.append(&downtime_entry);

    // Servings field
    let servings_box = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
    let servings_label = gtk::Label::new(Some("Servings:"));
    servings_label.set_halign(gtk::Align::Start);
    servings_label.set_width_chars(12);
    let servings_entry = gtk::Entry::new();
    if let Some(servings) = recipe.servings {
        servings_entry.set_text(&servings.to_string());
    }
    servings_entry.set_hexpand(true);
    servings_box.append(&servings_label);
    servings_box.append(&servings_entry);

    // Tags field (comma-separated)
    let tags_box = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
    let tags_label = gtk::Label::new(Some("Tags:"));
    tags_label.set_halign(gtk::Align::Start);
    tags_label.set_width_chars(12);
    let tags_entry = gtk::Entry::new();
    tags_entry.set_text(&recipe.tags.clone().unwrap_or_default().join(", "));
    tags_entry.set_hexpand(true);
    tags_box.append(&tags_label);
    tags_box.append(&tags_entry);

    // Separator
    form_container.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

    // Ingredients section heading
    let ingredients_label = gtk::Label::new(Some("Ingredients"));
    ingredients_label.set_markup("<span weight='bold'>Ingredients</span>");
    ingredients_label.set_halign(gtk::Align::Start);
    form_container.append(&ingredients_label);

    // Ingredients container (will hold ingredient rows)
    let ingredients_container = gtk::Box::new(gtk::Orientation::Vertical, TAG_SPACING);

    // Collect all ingredient names for completion (if data_manager is available)
    let ingredient_names: Vec<String> = data_manager
        .as_ref()
        .map(|dm| dm.get_all_ingredients().into_iter().map(|i| i.name.clone()).collect())
        .unwrap_or_default();

    // Helper to create a ListStore for completion
    fn create_ingredient_list_store(names: &[String]) -> gtk::ListStore {
        let store = gtk::ListStore::new(&[String::static_type()]);
        for name in names {
            let iter = store.append();
            store.set(&iter, &[(0, &name)]);
        }
        store
    }

    // Helper to create a name_entry with completion, triggers dropdown on focus, limits to 8 results
    let create_name_entry_with_completion = |default_text: Option<&str>| {
        let entry = gtk::Entry::new();
        if let Some(text) = default_text {
            entry.set_text(text);
        }
        entry.set_placeholder_text(Some("Ingredient name"));
        entry.set_hexpand(true);
        if !ingredient_names.is_empty() {
            let completion = gtk::EntryCompletion::new();
            let store = create_ingredient_list_store(&ingredient_names);
            completion.set_model(Some(&store));
            completion.set_text_column(0);
            completion.set_minimum_key_length(0); // Show dropdown on focus

            // Remove match_func for now to let GTK handle filtering and ensure dropdown works
            // Show dropdown immediately on focus using EventControllerFocus
            let completion_clone = completion.clone();
            let focus_controller = gtk::EventControllerFocus::new();
            focus_controller.connect_enter(move |_| {
                completion_clone.complete();
            });
            entry.add_controller(focus_controller);
            entry.set_completion(Some(&completion));
        }
        entry
    };

    // Add existing ingredients
    for ingredient in &recipe.ingredients {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, TAG_SPACING);

        let name_entry = create_name_entry_with_completion(Some(&ingredient.ingredient));
        row.append(&name_entry);

        let qty_entry = gtk::Entry::new();
        if let Some(qty) = &ingredient.quantity {
            qty_entry.set_text(&qty.to_string());
        }
        qty_entry.set_placeholder_text(Some("Quantity"));
        qty_entry.set_width_chars(8);
        row.append(&qty_entry);

        let qty_type_entry = gtk::Entry::new();
        if let Some(qty_type) = &ingredient.quantity_type {
            qty_type_entry.set_text(qty_type);
        }
        qty_type_entry.set_placeholder_text(Some("Unit"));
        qty_type_entry.set_width_chars(8);
        row.append(&qty_type_entry);

        let remove_button = gtk::Button::from_icon_name("list-remove");
        remove_button.set_tooltip_text(Some("Remove ingredient"));

        let row_clone = gtk::Box::clone(&row);
        remove_button.connect_clicked(move |_| {
            if let Some(parent) = row_clone.parent() {
                if let Some(box_parent) = parent.downcast_ref::<gtk::Box>() {
                    box_parent.remove(&row_clone);
                }
            }
        });

        row.append(&remove_button);
        ingredients_container.append(&row);
    }

    // Add button for ingredients
    let add_ingredient_button = gtk::Button::with_label("Add Ingredient");
    add_ingredient_button.set_halign(gtk::Align::Start);

    let ingredients_container_ref = ingredients_container.clone();
    let ingredient_names_clone = ingredient_names.clone();
    add_ingredient_button.connect_clicked(move |_| {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, TAG_SPACING);

        // Use the same helper for new ingredient rows
        let name_entry = {
            let entry = gtk::Entry::new();
            entry.set_placeholder_text(Some("Ingredient name"));
            entry.set_hexpand(true);
            if !ingredient_names_clone.is_empty() {
                let completion = gtk::EntryCompletion::new();
                let store = {
                    let store = gtk::ListStore::new(&[String::static_type()]);
                    for name in &ingredient_names_clone {
                        let iter = store.append();
                        store.set(&iter, &[(0, &name)]);
                    }
                    store
                };
                completion.set_model(Some(&store));
                completion.set_text_column(0);
                completion.set_minimum_key_length(0);
                // Remove match_func for now to let GTK handle filtering and ensure dropdown works
                // Show dropdown immediately on focus using EventControllerFocus
                let completion_clone = completion.clone();
                let focus_controller = gtk::EventControllerFocus::new();
                focus_controller.connect_enter(move |_| {
                    completion_clone.complete();
                });
                entry.add_controller(focus_controller);
                entry.set_completion(Some(&completion));
            }
            entry
        };
        row.append(&name_entry);

        let qty_entry = gtk::Entry::new();
        qty_entry.set_placeholder_text(Some("Quantity"));
        qty_entry.set_width_chars(8);
        row.append(&qty_entry);

        let qty_type_entry = gtk::Entry::new();
        qty_type_entry.set_placeholder_text(Some("Unit"));
        qty_type_entry.set_width_chars(8);
        row.append(&qty_type_entry);

        let remove_button = gtk::Button::from_icon_name("list-remove");
        remove_button.set_tooltip_text(Some("Remove ingredient"));

        let row_clone = gtk::Box::clone(&row);
        remove_button.connect_clicked(move |_| {
            if let Some(parent) = row_clone.parent() {
                if let Some(box_parent) = parent.downcast_ref::<gtk::Box>() {
                    box_parent.remove(&row_clone);
                }
            }
        });

        row.append(&remove_button);
        ingredients_container_ref.append(&row);
    });

    // Image section
    let image_section = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
    let image_preview = if let Some(image_name) = &recipe.image {
        if let Some(dm) = &data_manager {
            let data_dir = dm.get_data_dir();
            let img_path = data_dir.join("recipes/img").join(image_name);
            if img_path.exists() {
                let img = gtk::Image::from_file(img_path);
                img.set_pixel_size(64);
                img
            } else {
                gtk::Image::new()
            }
        } else {
            gtk::Image::new()
        }
    } else {
        gtk::Image::new()
    };
    image_section.append(&image_preview);
    let set_image_button = gtk::Button::with_label("Set Image");
    let selected_image_path_clone = selected_image_path.clone();
    set_image_button.connect_clicked(move |_| {
        let file_chooser = gtk::FileChooserNative::new(
            Some("Select Recipe Image"),
            None::<&gtk::Window>,
            gtk::FileChooserAction::Open,
            Some("Open"),
            Some("Cancel"),
        );
        file_chooser.set_modal(true);
        let selected_image_path_clone_inner = selected_image_path_clone.clone();
        file_chooser.connect_response(move |dialog, response| {
            if response == gtk::ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        *selected_image_path_clone_inner.borrow_mut() = Some(path);
                    }
                }
            }
            dialog.destroy();
        });
        file_chooser.show();
    });
    image_section.append(&set_image_button);
    form_container.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    form_container.append(&image_section);

    // Instructions section (create before closure so it's in scope)
    let instructions_label = gtk::Label::new(Some("Instructions"));
    instructions_label.set_markup("<span weight='bold'>Instructions</span>");
    instructions_label.set_halign(gtk::Align::Start);
    let instructions_text_view = gtk::TextView::new();
    instructions_text_view.set_wrap_mode(gtk::WrapMode::Word);
    instructions_text_view.set_hexpand(true);
    instructions_text_view.set_vexpand(false);
    instructions_text_view.set_height_request(120);
    instructions_text_view.buffer().set_text(&recipe.instructions);

    // Append all field widgets to form_container in the correct order
    form_container.append(&title_box);
    form_container.append(&prep_time_box);
    form_container.append(&downtime_box);
    form_container.append(&servings_box);
    form_container.append(&tags_box);
    form_container.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    form_container.append(&ingredients_label);
    form_container.append(&ingredients_container);
    form_container.append(&add_ingredient_button);
    form_container.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    form_container.append(&image_section);
    form_container.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    form_container.append(&instructions_label);
    form_container.append(&instructions_text_view);
    form_container.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

    scrolled_window.set_child(Some(&form_container));
    content_area.append(&scrolled_window);

    // Clone image field to avoid borrow checker issues in closure
    let recipe_image_field = recipe.image.clone();
    // Clones for closure
    let sender_clone = sender.clone();
    let recipe_title_clone = recipe_title.clone();
    let data_manager_clone = data_manager.clone();
    let ingredients_container_ref = ingredients_container.clone();

    dialog.add_button("Cancel", gtk::ResponseType::Cancel);
    dialog.add_button("Save", gtk::ResponseType::Accept);

    dialog.connect_response(move |dialog, response| {
        if response == gtk::ResponseType::Accept {
            let new_title = title_entry.text().to_string();
            let prep_time = prep_time_entry.text().to_string().parse::<u32>().ok();
            let downtime = downtime_entry.text().to_string().parse::<u32>().ok();
            let servings = servings_entry.text().to_string().parse::<u32>().ok();
            let tags = tags_entry
                .text()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<String>>();

            let mut ingredients = Vec::new();
            let mut current = ingredients_container_ref.first_child();
            while let Some(child) = current {
                if let Some(row) = child.downcast_ref::<gtk::Box>() {
                    let name_widget = row.first_child();
                    let qty_widget = name_widget.as_ref().and_then(|w| w.next_sibling());
                    let type_widget = qty_widget.as_ref().and_then(|w| w.next_sibling());

                    if let (Some(name_widget), Some(qty_widget), Some(type_widget)) =
                        (name_widget, qty_widget, type_widget)
                    {
                        if let (Some(name_entry), Some(qty_entry), Some(type_entry)) = (
                            name_widget.downcast_ref::<gtk::Entry>(),
                            qty_widget.downcast_ref::<gtk::Entry>(),
                            type_widget.downcast_ref::<gtk::Entry>(),
                        ) {
                            let ingredient_name = name_entry.text().to_string();
                            if !ingredient_name.is_empty() {
                                let qty = qty_entry.text().to_string();
                                let qty_type = type_entry.text().to_string();
                                let parsed_qty = if qty.is_empty() {
                                    None
                                } else {
                                    match qty.parse::<f64>() {
                                        Ok(value) => Some(value),
                                        Err(_) => None,
                                    }
                                };
                                ingredients.push(cookbook_engine::RecipeIngredient {
                                    ingredient: ingredient_name,
                                    quantity: parsed_qty,
                                    quantity_type: if qty_type.is_empty() {
                                        None
                                    } else {
                                        Some(qty_type)
                                    },
                                });
                            }
                        }
                    }
                }
                current = child.next_sibling();
            }

            let instructions = instructions_text_view
                .buffer()
                .text(&instructions_text_view.buffer().start_iter(), &instructions_text_view.buffer().end_iter(), false)
                .to_string();

            // Handle image copy if a new image was selected
            let mut image_field: Option<String> = recipe_image_field.clone();
            if let Some(selected_path) = selected_image_path.borrow().as_ref() {
                if let Some(dm) = &data_manager_clone {
                    let data_dir = dm.get_data_dir();
                    let img_dir = data_dir.join("recipes/img");
                    let ext = selected_path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    let safe_title = title_entry.text().replace(' ', "_");
                    let new_filename = format!("{}.{}", safe_title, ext);
                    let dest_path = img_dir.join(&new_filename);
                    let _ = std::fs::create_dir_all(&img_dir);
                    let _ = std::fs::copy(selected_path, &dest_path);
                    image_field = Some(new_filename);
                }
            }

            let new_recipe = cookbook_engine::Recipe {
                title: new_title,
                ingredients,
                prep_time,
                downtime,
                servings,
                tags: if tags.is_empty() { None } else { Some(tags) },
                image: image_field,
                instructions,
            };

            if let Some(_rc_dm) = &data_manager_clone {
                let original_title = recipe_title_clone.clone();
                let recipe_clone = new_recipe.clone();
                let sender_clone2 = sender_clone.clone();
                sender_clone2.input(AppMsg::UpdateRecipe(original_title, recipe_clone));

                match Result::<bool, cookbook_engine::CookbookError>::Ok(true) {
                    Ok(_) => {
                        let new_selected_title = new_recipe.title.clone();
                        let sender_clone_inner = sender_clone.clone();
                        glib::spawn_future_local(async move {
                            sender_clone_inner.input(AppMsg::SelectRecipe(new_selected_title));
                        });
                    }
                    Err(err) => {
                        let error_dialog = gtk::MessageDialog::new(
                            None::<&gtk::Window>,
                            gtk::DialogFlags::MODAL,
                            gtk::MessageType::Error,
                            gtk::ButtonsType::Ok,
                            &format!("Failed to update recipe: {}", err),
                        );
                        error_dialog.connect_response(|dialog, _| dialog.destroy());
                        error_dialog.show();
                    }
                }
            }
        }
        dialog.destroy();
    });

    dialog.show();
}

/// Builds the Recipes tab UI and returns the main container, list box, and details box.
pub fn build_recipes_tab(
    model: &AppModel,
    sender: &ComponentSender<AppModel>,
) -> (gtk::Box, gtk::ListBox, gtk::Box) {
    // Main vertical container for the Recipes tab
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
        sender_clone.input(AppMsg::SearchTextChanged(entry.text().to_string()));
    });

    recipes_header.append(&recipes_title);
    recipes_header.append(&search_entry);
    recipes_container.append(&recipes_header);

    // Split view: list (left), details (right)
    let recipes_content = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
    recipes_content.set_hexpand(true);
    recipes_content.set_vexpand(true);

    // Recipe list
    let recipes_list_scroll = gtk::ScrolledWindow::new();
    recipes_list_scroll.set_hexpand(false);
    recipes_list_scroll.set_vexpand(true);
    recipes_list_scroll.set_min_content_width(250);

    let recipes_list_box = gtk::ListBox::new();
    recipes_list_box.set_selection_mode(gtk::SelectionMode::Single);

    // Populate the list
    crate::recipes::update_recipes_list(
        &recipes_list_box,
        &model.data_manager,
        &model.search_text,
        sender,
        |title| AppMsg::SelectRecipe(title),
    );

    // Recipe selection handler
    let sender_clone = sender.clone();
    recipes_list_box.connect_row_selected(move |_list, row_opt| {
        if let Some(row) = row_opt {
            if let Some(box_layout) = row.child().and_then(|w| w.downcast::<gtk::Box>().ok()) {
                if let Some(label) = box_layout.first_child().and_then(|w| w.downcast::<gtk::Label>().ok()) {
                    let recipe_title = label.text().to_string();
                    sender_clone.input(AppMsg::SelectRecipe(recipe_title));
                }
            }
        }
    });

    recipes_list_scroll.set_child(Some(&recipes_list_box));

    // Recipe details view (right side)
    let recipes_details = gtk::Box::new(gtk::Orientation::Vertical, SECTION_SPACING);
    recipes_details.set_hexpand(true);
    recipes_details.set_vexpand(true);

    let recipes_label = gtk::Label::new(Some("Select a recipe to view details"));
    recipes_label.set_halign(gtk::Align::Center);
    recipes_label.set_valign(gtk::Align::Center);
    recipes_label.set_hexpand(true);
    recipes_label.set_vexpand(true);

    recipes_details.append(&recipes_label);

    recipes_content.append(&recipes_list_scroll);
    recipes_content.append(&recipes_details);

    recipes_container.append(&recipes_content);

    (recipes_container, recipes_list_box, recipes_details)
}