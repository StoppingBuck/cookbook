use crate::types::AppMsg;
use crate::ui_constants::*;
use gdk_pixbuf;
use gtk::prelude::*;
use gtk::glib;
use relm4::gtk;
use relm4::ComponentSender;
use relm4::RelmWidgetExt;
use std::cell::RefCell;
use std::rc::Rc;

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
    dialog.set_default_width(750); // Increased from 700
    dialog.set_default_height(850); // Increased from 800

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
    content_area.set_spacing(SECTION_SPACING); // Spacing for direct children of content_area

    let scrolled_window = gtk::ScrolledWindow::new();
    scrolled_window.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scrolled_window.set_vexpand(true);

    let form_container = gtk::Box::new(gtk::Orientation::Vertical, SECTION_SPACING); // Spacing between sections in the form
    form_container.set_margin_all(DEFAULT_MARGIN); // Margin for the form itself within scrolled_window

    // --- Metadata Grid (Title, Times, Servings, Tags) ---
    let metadata_grid = gtk::Grid::new();
    metadata_grid.set_column_spacing(DEFAULT_MARGIN.try_into().unwrap_or(0));
    metadata_grid.set_row_spacing(TAG_SPACING.try_into().unwrap_or(0));
    metadata_grid.set_margin_bottom(SECTION_SPACING); // Space after the grid

    // Title field
    let title_label = gtk::Label::new(Some("Title:"));
    title_label.set_halign(gtk::Align::Start);
    title_label.set_valign(gtk::Align::Center);
    let title_entry = gtk::Entry::new();
    title_entry.set_text(&recipe.title);
    title_entry.set_hexpand(true);
    title_entry.set_valign(gtk::Align::Center);
    metadata_grid.attach(&title_label, 0, 0, 1, 1);
    metadata_grid.attach(&title_entry, 1, 0, 3, 1);

    // Prep time field (SpinButton)
    let prep_time_label = gtk::Label::new(Some("Prep Time (min):"));
    prep_time_label.set_halign(gtk::Align::Start);
    prep_time_label.set_valign(gtk::Align::Center);
    let prep_time_spin = gtk::SpinButton::with_range(0.0, 999.0, 1.0);
    prep_time_spin.set_width_chars(4);
    prep_time_spin.set_valign(gtk::Align::Center);
    if let Some(prep_time) = recipe.prep_time {
        prep_time_spin.set_value(prep_time as f64);
    }
    metadata_grid.attach(&prep_time_label, 0, 1, 1, 1);
    metadata_grid.attach(&prep_time_spin, 1, 1, 1, 1);

    // Servings field (SpinButton)
    let servings_label = gtk::Label::new(Some("Servings:"));
    servings_label.set_halign(gtk::Align::Start);
    servings_label.set_valign(gtk::Align::Center);
    let servings_spin = gtk::SpinButton::with_range(0.0, 99.0, 1.0);
    servings_spin.set_width_chars(3);
    servings_spin.set_valign(gtk::Align::Center);
    if let Some(servings) = recipe.servings {
        servings_spin.set_value(servings as f64);
    }
    metadata_grid.attach(&servings_label, 2, 1, 1, 1);
    metadata_grid.attach(&servings_spin, 3, 1, 1, 1);

    // Downtime field (Cook Time, SpinButton)
    let downtime_label = gtk::Label::new(Some("Cook Time (min):"));
    downtime_label.set_halign(gtk::Align::Start);
    downtime_label.set_valign(gtk::Align::Center);
    let downtime_spin = gtk::SpinButton::with_range(0.0, 999.0, 1.0);
    downtime_spin.set_width_chars(4);
    downtime_spin.set_valign(gtk::Align::Center);
    if let Some(downtime) = recipe.downtime {
        downtime_spin.set_value(downtime as f64);
    }
    metadata_grid.attach(&downtime_label, 0, 2, 1, 1);
    metadata_grid.attach(&downtime_spin, 1, 2, 1, 1);

    // Tags field
    let tags_label = gtk::Label::new(Some("Tags (comma-sep):"));
    tags_label.set_halign(gtk::Align::Start);
    tags_label.set_valign(gtk::Align::Center);
    let tags_entry = gtk::Entry::new();
    tags_entry.set_text(&recipe.tags.clone().unwrap_or_default().join(", "));
    tags_entry.set_hexpand(true);
    tags_entry.set_valign(gtk::Align::Center);
    metadata_grid.attach(&tags_label, 0, 3, 1, 1);
    metadata_grid.attach(&tags_entry, 1, 3, 3, 1);

    // --- Image section (now in metadata grid, rightmost column, spanning rows 0-3) ---
    let image_section = gtk::Box::new(gtk::Orientation::Vertical, TAG_SPACING);
    image_section.set_valign(gtk::Align::Center);
    image_section.set_margin_start(SECTION_SPACING);
    // Smaller preview
    let image_preview = if let Some(image_name) = &recipe.image {
        if let Some(dm) = &data_manager {
            let data_dir = dm.get_data_dir();
            let img_path = data_dir.join("recipes/img").join(image_name);
            if img_path.exists() {
                let img = gtk::Image::from_file(img_path);
                img.set_pixel_size(80);
                img.set_size_request(80, 80);
                img.set_visible(true);
                img
            } else {
                let img = gtk::Image::new();
                img.set_pixel_size(80);
                img.set_size_request(80, 80);
                img.set_visible(true);
                img
            }
        } else {
            let img = gtk::Image::new();
            img.set_pixel_size(80);
            img.set_size_request(80, 80);
            img.set_visible(true);
            img
        }
    } else {
        let img = gtk::Image::new();
        img.set_pixel_size(80);
        img.set_size_request(80, 80);
        img.set_visible(true);
        img
    };
    image_section.append(&image_preview);
    let image_buttons_box = gtk::Box::new(gtk::Orientation::Horizontal, TAG_SPACING);
    let set_image_button = gtk::Button::with_label("Set Image");
    let clear_image_button = gtk::Button::with_label("Clear Image");
    image_buttons_box.append(&set_image_button);
    image_buttons_box.append(&clear_image_button);
    image_section.append(&image_buttons_box);
    // Place image_section in metadata_grid (col 4, row 0, span 1x4)
    metadata_grid.attach(&image_section, 4, 0, 1, 4);

    form_container.append(&metadata_grid);
    // ...existing code...

    // No separator needed here, metadata_grid already has margin_bottom

    // Ingredients section heading
    let ingredients_label = gtk::Label::new(Some("Ingredients"));
    ingredients_label.set_markup("<span weight='bold'>Ingredients</span>");
    ingredients_label.set_halign(gtk::Align::Start);
    ingredients_label.set_margin_bottom(TAG_SPACING);
    form_container.append(&ingredients_label);

    // Ingredients container (will hold ingredient rows)
    let ingredients_container = gtk::Box::new(gtk::Orientation::Vertical, TAG_SPACING); // Spacing between ingredient rows

    // Collect all ingredient names for completion (if data_manager is available)
    let ingredient_names: Vec<String> = data_manager
        .as_ref()
        .map(|dm| {
            dm.get_all_ingredients()
                .into_iter()
                .map(|i| i.name.clone())
                .collect()
        })
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
        let row = gtk::Box::new(gtk::Orientation::Horizontal, TAG_SPACING); // Spacing within an ingredient row

        let name_entry = create_name_entry_with_completion(Some(&ingredient.ingredient));
        row.append(&name_entry);

        let qty_spin = gtk::SpinButton::with_range(0.0, 999.0, 1.0);
        qty_spin.set_width_chars(4);
        if let Some(qty) = ingredient.quantity {
            qty_spin.set_value(qty);
        }
        row.append(&qty_spin);

        let qty_type_entry = gtk::Entry::new();
        if let Some(qty_type) = &ingredient.quantity_type {
            qty_type_entry.set_text(qty_type);
        }
        qty_type_entry.set_placeholder_text(Some("Unit"));
        qty_type_entry.set_width_chars(8); // Kept at 8
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
    add_ingredient_button.set_margin_top(TAG_SPACING);

    let ingredients_container_ref = ingredients_container.clone();
    let ingredient_names_clone = ingredient_names.clone();
    add_ingredient_button.connect_clicked(move |_| {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, TAG_SPACING);

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
        let qty_spin = gtk::SpinButton::with_range(0.0, 999.0, 1.0);
        qty_spin.set_width_chars(4);
        row.append(&qty_spin);

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
    image_section.set_valign(gtk::Align::Center);
    image_section.set_margin_top(TAG_SPACING);
    image_section.set_margin_bottom(TAG_SPACING);

    let image_preview = if let Some(image_name) = &recipe.image {
        if let Some(dm) = &data_manager {
            let data_dir = dm.get_data_dir();
            let img_path = data_dir.join("recipes/img").join(image_name);
            if img_path.exists() {
                let img = gtk::Image::from_file(img_path);
                img.set_pixel_size(100);
                img.set_size_request(100, 100); // Ensure size request
                img.set_visible(true); // Ensure visible
                img
            } else {
                let img = gtk::Image::new();
                img.set_pixel_size(100);
                img.set_size_request(100, 100); // Ensure size request
                img.set_visible(true); // Ensure visible
                img
            }
        } else {
            let img = gtk::Image::new();
            img.set_pixel_size(100);
            img.set_size_request(100, 100); // Ensure size request
            img.set_visible(true); // Ensure visible
            img
        }
    } else {
        let img = gtk::Image::new();
        img.set_pixel_size(100); // Ensure consistent size for placeholder
        img.set_size_request(100, 100); // Ensure size request
        img.set_visible(true); // Ensure visible
        img
    };
    image_preview.set_margin_end(SECTION_SPACING);
    image_section.append(&image_preview);

    let image_buttons_box = gtk::Box::new(gtk::Orientation::Vertical, TAG_SPACING);
    let set_image_button = gtk::Button::with_label("Set Image");
    let clear_image_button = gtk::Button::with_label("Clear Image");
    image_buttons_box.append(&set_image_button);
    image_buttons_box.append(&clear_image_button);
    image_section.append(&image_buttons_box);

    let cleared_image: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let image_preview_clone = image_preview.clone();
    let recipe_image_field_clone = recipe.image.clone();
    let cleared_image_clone = cleared_image.clone();
    // Clone selected_image_path for the clear_image_button closure *before* it's potentially moved.
    let selected_image_path_for_clear = selected_image_path.clone();

    clear_image_button.connect_clicked(move |_| {
        if recipe_image_field_clone.is_some() {
            image_preview_clone.set_from_pixbuf(None::<&gdk_pixbuf::Pixbuf>);
            *cleared_image_clone.borrow_mut() = recipe_image_field_clone.clone();
            *selected_image_path_for_clear.borrow_mut() = None; // Use the pre-cloned version
        }
    });

    let selected_image_path_clone = selected_image_path.clone(); // Used in set_image_button closure
    let image_preview_clone_for_set = image_preview.clone(); // Separate clone for set_image_button
    let cleared_image_clone_for_set = cleared_image.clone(); // If user sets then clears, then sets again
    set_image_button.connect_clicked(move |_| {
        let file_chooser = gtk::FileChooserNative::new(
            Some("Select Recipe Image"),
            None::<&gtk::Window>, // No direct parent window for native dialog
            gtk::FileChooserAction::Open,
            Some("Open"),
            Some("Cancel"),
        );
        file_chooser.set_modal(true); // Make it modal to the application

        // Need to clone for the inner closure
        let selected_image_path_clone_inner = selected_image_path_clone.clone();
        let image_preview_clone_inner = image_preview_clone_for_set.clone();
        let cleared_image_clone_inner = cleared_image_clone_for_set.clone();

        file_chooser.connect_response(move |dialog, response| {
            if response == gtk::ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        image_preview_clone_inner.set_from_file(Some(&path));
                        image_preview_clone_inner.set_visible(true);
                        *selected_image_path_clone_inner.borrow_mut() = Some(path);
                        *cleared_image_clone_inner.borrow_mut() = None; // New image selected, so not clearing
                    }
                }
            }
            dialog.destroy();
        });
        file_chooser.show();
    });

    // Instructions section
    let instructions_label = gtk::Label::new(Some("Instructions"));
    instructions_label.set_markup("<span weight='bold'>Instructions</span>");
    instructions_label.set_halign(gtk::Align::Start);
    instructions_label.set_margin_bottom(TAG_SPACING); // Add some space before the text view
    let instructions_text_view = gtk::TextView::new();
    instructions_text_view.set_wrap_mode(gtk::WrapMode::Word);
    instructions_text_view.set_hexpand(true);
    instructions_text_view.set_vexpand(true);
    instructions_text_view.set_height_request(150);
    instructions_text_view
        .buffer()
        .set_text(&recipe.instructions);

    // --- REVISED APPEND ORDER FOR form_container ---
    form_container.append(&metadata_grid);
    form_container.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

    form_container.append(&ingredients_label);

    // Ensure ingredients_container is visible and has some minimum space
    ingredients_container.set_visible(true);
    ingredients_container.set_height_request(100); // Request minimum 100px height
    form_container.append(&ingredients_container);

    // Ensure add_ingredient_button is visible
    add_ingredient_button.set_visible(true);
    form_container.append(&add_ingredient_button);
    form_container.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

    // Image section is now in metadata grid, not here

    form_container.append(&instructions_label);
    form_container.append(&instructions_text_view);
    form_container.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    // --- END OF REVISED APPEND ORDER ---

    scrolled_window.set_child(Some(&form_container));
    content_area.append(&scrolled_window);

    let recipe_image_field = recipe.image.clone(); // Used in the main save response closure
                                                   // Clones for closure
    let sender_clone = sender.clone();
    let recipe_title_clone = recipe_title.clone();
    let data_manager_clone = data_manager.clone();
    let ingredients_container_ref = ingredients_container.clone();

    // Determine if this is an add or update dialog
    let is_add = recipe_title.is_empty();

    let _cancel_button = dialog.add_button("Cancel", gtk::ResponseType::Cancel);
    let save_button = dialog.add_button("Save", gtk::ResponseType::Accept);

    // Style action area - get the parent of one of the buttons
    if let Some(button_parent) = save_button.parent() {
        if let Some(action_area_box) = button_parent.downcast_ref::<gtk::Box>() {
            action_area_box.set_margin_all(DEFAULT_MARGIN); // Add margin around the action buttons area
            action_area_box.set_spacing(SECTION_SPACING.try_into().unwrap_or(0));
        }
    }

    dialog.connect_response(move |dialog, response| {
        if response == gtk::ResponseType::Accept {
            let new_title = title_entry.text().to_string();
            let prep_time = Some(prep_time_spin.value() as u32);
            let downtime = Some(downtime_spin.value() as u32);
            let servings = Some(servings_spin.value() as u32);
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
                        if let (Some(name_entry), Some(qty_spin), Some(type_entry)) = (
                            name_widget.downcast_ref::<gtk::Entry>(),
                            qty_widget.downcast_ref::<gtk::SpinButton>(),
                            type_widget.downcast_ref::<gtk::Entry>(),
                        ) {
                            let ingredient_name = name_entry.text().to_string();
                            if !ingredient_name.is_empty() {
                                let qty_val = qty_spin.value();
                                let qty = if qty_val == 0.0 { None } else { Some(qty_val) };
                                let qty_type = type_entry.text().to_string();
                                ingredients.push(cookbook_engine::RecipeIngredient {
                                    ingredient: ingredient_name,
                                    quantity: qty,
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
                .text(
                    &instructions_text_view.buffer().start_iter(),
                    &instructions_text_view.buffer().end_iter(),
                    false,
                )
                .to_string();

            // Handle image copy if a new image was selected
            let mut image_field: Option<String> = recipe_image_field.clone();
            if let Some(selected_path) = selected_image_path.borrow().as_ref() {
                if let Some(dm) = &data_manager_clone {
                    let data_dir = dm.get_data_dir();
                    let img_dir = data_dir.join("recipes/img");
                    let ext = selected_path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("");
                    let safe_title = title_entry.text().replace(' ', "_");
                    let new_filename = format!("{}.{}", safe_title, ext);
                    let dest_path = img_dir.join(&new_filename);
                    let _ = std::fs::create_dir_all(&img_dir);
                    let _ = std::fs::copy(selected_path, &dest_path);
                    image_field = Some(new_filename);
                }
            }
            // Handle image clearing
            if let Some(image_to_delete) = cleared_image.borrow().as_ref() {
                // Remove image field
                image_field = None;
                // Delete the image file from data_dir/recipes/img/
                if let Some(dm) = &data_manager_clone {
                    let data_dir = dm.get_data_dir();
                    let img_path = data_dir.join("recipes/img").join(image_to_delete);
                    let _ = std::fs::remove_file(img_path);
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
                let sender_clone2 = sender_clone.clone();
                if is_add {
                    sender_clone2.input(AppMsg::CreateRecipe(new_recipe.clone()));
                } else {
                    let original_title = recipe_title_clone.clone();
                    sender_clone2.input(AppMsg::UpdateRecipe(original_title, new_recipe.clone()));
                }

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

/// Show the Add Recipe dialog with a blank recipe.
pub fn show_add_recipe_dialog(
    data_manager: Option<Rc<cookbook_engine::DataManager>>,
    sender: ComponentSender<crate::types::AppModel>,
) {
    let blank_recipe = cookbook_engine::Recipe {
        title: String::new(),
        ingredients: Vec::new(),
        prep_time: None,
        downtime: None,
        servings: None,
        tags: Some(Vec::new()),
        image: None,
        instructions: String::new(),
    };
    show_edit_recipe_dialog(&blank_recipe, data_manager, sender, String::new());
}
