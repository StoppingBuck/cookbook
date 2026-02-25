use crate::ui_constants::*;
use cookbook_engine::DataManager;
use gtk::prelude::*;
use relm4::gtk;
use relm4::ComponentSender;
use relm4::RelmWidgetExt;
use std::rc::Rc;

/// Builds and returns the Pantry Details Pane for a selected ingredient
pub fn build_ingredient_detail_view<C>(
    data_manager: &Rc<DataManager>,
    ingredient_id: &str, // now this is always a slug
    sender: &ComponentSender<C>,
    switch_tab_msg: impl Fn(crate::types::Tab) -> C::Input + Clone + 'static,
    select_kb_entry_msg: impl Fn(String) -> C::Input + Clone + 'static,
    select_recipe_msg: impl Fn(String) -> C::Input + Clone + 'static,
    edit_ingredient_msg: impl Fn(String) -> C::Input + Clone + 'static,
    delete_ingredient_msg: impl Fn(String) -> C::Input + Clone + 'static,
) -> gtk::Box
where
    C: relm4::Component,
{
    // Create a small details view
    let details_container = gtk::Box::new(gtk::Orientation::Vertical, SECTION_SPACING);
    details_container.set_margin_all(DEFAULT_MARGIN);

    let lang = data_manager
        .get_all_ingredients()
        .first()
        .and_then(|_| Some("en")) // fallback if needed
        .unwrap_or("en");
    // Try to resolve by slug or translation
    let ingredient = data_manager.find_ingredient_by_name_or_translation(ingredient_id, lang);

    if let Some(ingredient) = ingredient {
        // Title with ingredient name and edit button in a horizontal box
        let title_box = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
        title_box.set_margin_bottom(DEFAULT_MARGIN);

        let title = gtk::Label::new(None);
        title.set_markup(&format!(
            "<span size='x-large' weight='bold'>{}</span>",
            ingredient.name
        ));
        title.set_halign(gtk::Align::Start);
        title.set_hexpand(true);
        title_box.append(&title);

        // Add Edit button
        let edit_button = gtk::Button::with_label("Edit");
        edit_button.add_css_class("suggested-action");
        let sender_clone = sender.clone();
        let ingredient_name = ingredient.name.clone();
        let edit_msg = edit_ingredient_msg.clone();
        edit_button.connect_clicked(move |_| {
            sender_clone.input(edit_msg(ingredient_name.clone()));
        });
        title_box.append(&edit_button);

        // Add Delete button (only enabled if not blank ingredient)
        if !ingredient.name.trim().is_empty() {
            let delete_button = gtk::Button::with_label("Delete");
            delete_button.add_css_class("destructive-action");
            let sender_clone = sender.clone();
            let ingredient_slug = ingredient.slug.clone();
            let delete_msg = delete_ingredient_msg.clone();
            delete_button.connect_clicked(move |_| {
                sender_clone.input(delete_msg(ingredient_slug.clone()));
            });
            title_box.append(&delete_button);
        }

        details_container.append(&title_box);

        // Category
        let category_label = gtk::Label::new(None);
        category_label.set_markup(&format!("<b>Category:</b> {}", ingredient.category));
        category_label.set_halign(gtk::Align::Start);
        category_label.set_margin_bottom(LIST_ROW_MARGIN);
        details_container.append(&category_label);

        // Tags (if any)
        if let Some(ref tags) = ingredient.tags {
            if !tags.is_empty() {
                let tags_label = gtk::Label::new(None);
                tags_label.set_markup("<b>Tags:</b>");
                tags_label.set_halign(gtk::Align::Start);
                tags_label.set_margin_bottom(LIST_ROW_MARGIN);
                details_container.append(&tags_label);

                let tags_box = gtk::Box::new(gtk::Orientation::Horizontal, TAG_SPACING);
                tags_box.set_margin_start(DEFAULT_MARGIN);
                tags_box.set_margin_bottom(DEFAULT_MARGIN);

                for tag in tags {
                    let tag_button = gtk::Button::new();
                    tag_button.set_label(tag);
                    tag_button.add_css_class("tag");
                    tags_box.append(&tag_button);
                }

                details_container.append(&tags_box);
            }
        }

        // Knowledge Base Link (if available)
        if let Some(kb_entry) = data_manager.get_kb_entry_for_ingredient(&ingredient.name) {
            let kb_box = gtk::Box::new(gtk::Orientation::Vertical, TAG_SPACING);
            kb_box.set_margin_top(DEFAULT_MARGIN);
            kb_box.set_margin_bottom(DEFAULT_MARGIN);

            let kb_label = gtk::Label::new(None);
            kb_label.set_markup("<b>Knowledge Base Entry:</b>");
            kb_label.set_halign(gtk::Align::Start);
            kb_box.append(&kb_label);

            let kb_button = gtk::Button::new();
            kb_button.set_label(&kb_entry.title);
            kb_button.set_halign(gtk::Align::Start);
            kb_button.add_css_class("link-button");

            let sender_clone = sender.clone();
            let kb_slug = kb_entry.slug.clone();
            let switch_tab = switch_tab_msg.clone();
            let select_kb_entry = select_kb_entry_msg.clone();
            kb_button.connect_clicked(move |_| {
                sender_clone.input(switch_tab(crate::types::Tab::KnowledgeBase));
                sender_clone.input(select_kb_entry(kb_slug.clone()));
            });

            kb_box.append(&kb_button);
            details_container.append(&kb_box);
        }

        // Pantry information (quantity, etc.)
        if let Some(pantry) = data_manager.get_pantry() {
            // Get the pantry data
            if let Some(pantry_item) = pantry
                .items
                .iter()
                .find(|item| item.ingredient == ingredient.name)
            {
                // Find the pantry item
                let stock_label = gtk::Label::new(None);
                stock_label.set_margin_top(DEFAULT_MARGIN);

                // Check if the item is in stock
                if let Some(quantity) = pantry_item.quantity {
                    if pantry_item.quantity_type.is_empty() {
                        stock_label.set_markup(&format!("<b>In stock:</b> {}", quantity));
                    } else {
                        stock_label.set_markup(&format!(
                            "<b>In stock:</b> {} {}",
                            quantity, pantry_item.quantity_type
                        ));
                    }
                    // Add checkmark
                    stock_label.set_text(&format!("{} ✅", stock_label.text()));
                } else {
                    stock_label.set_markup("<b>In stock:</b> quantity unknown ❓");
                }

                stock_label.set_halign(gtk::Align::Start);
                details_container.append(&stock_label);

                // Last updated date
                let updated_label = gtk::Label::new(None);
                updated_label.set_markup(&format!(
                    "<b>Last updated:</b> {}",
                    pantry_item.last_updated
                ));
                updated_label.set_halign(gtk::Align::Start);
                updated_label.set_margin_bottom(DEFAULT_MARGIN);
                details_container.append(&updated_label);
            } else {
                let not_in_stock_label = gtk::Label::new(None);
                not_in_stock_label.set_markup("<b>Not in pantry</b>");
                not_in_stock_label.set_halign(gtk::Align::Start);
                not_in_stock_label.set_margin_top(DEFAULT_MARGIN);
                details_container.append(&not_in_stock_label);
            }
        }

        // Recipes with this ingredient
        let ingredient_usage = data_manager.get_ingredient_usage();
        let recipes_with_ingredient = ingredient_usage
            .get(&ingredient.name)
            .cloned()
            .unwrap_or_else(Vec::new);

        if !recipes_with_ingredient.is_empty() {
            let recipes_header = gtk::Label::new(None);
            recipes_header.set_markup(&format!(
                "<span size='large' weight='bold'>Recipes with {}:</span>",
                ingredient.name
            ));
            recipes_header.set_halign(gtk::Align::Start);
            recipes_header.set_margin_top(DETAILS_MARGIN);
            recipes_header.set_margin_bottom(LIST_ROW_MARGIN);
            details_container.append(&recipes_header);

            let recipes_box = gtk::Box::new(gtk::Orientation::Vertical, TAG_SPACING);
            recipes_box.set_margin_start(DEFAULT_MARGIN);

            for recipe in recipes_with_ingredient {
                let recipe_button = gtk::Button::new();

                // Check if all ingredients for this recipe are in the pantry
                let all_ingredients_in_stock =
                    data_manager.are_all_ingredients_in_pantry(&recipe.title);

                // Set label with checkmark if all ingredients are in stock
                if all_ingredients_in_stock {
                    recipe_button.set_label(&format!("{} ✅", recipe.title));
                } else {
                    recipe_button.set_label(&recipe.title);
                }

                recipe_button.set_halign(gtk::Align::Start);
                recipe_button.add_css_class("link-button");

                let sender_clone = sender.clone();
                let recipe_title = recipe.title.clone();
                let switch_tab = switch_tab_msg.clone();
                let select_recipe = select_recipe_msg.clone();
                recipe_button.connect_clicked(move |_| {
                    sender_clone.input(switch_tab(crate::types::Tab::Recipes));
                    sender_clone.input(select_recipe(recipe_title.clone()));
                });

                recipes_box.append(&recipe_button);
            }

            details_container.append(&recipes_box);
        }
    } else {
        // Ingredient not found
        let not_found_label =
            gtk::Label::new(Some(&format!("Ingredient '{}' not found", ingredient_id)));
        not_found_label.set_halign(gtk::Align::Center);
        not_found_label.set_valign(gtk::Align::Center);
        details_container.append(&not_found_label);
    }

    details_container
}

pub fn show_edit_ingredient_dialog(
    ingredient: &cookbook_engine::Ingredient,
    pantry_item: Option<&cookbook_engine::PantryItem>,
    data_manager: Option<Rc<cookbook_engine::DataManager>>,
    sender: ComponentSender<crate::types::AppModel>,
    ingredient_name: String,
) {
    let dialog = gtk::Dialog::new();
    dialog.set_title(Some(&format!("Edit Ingredient: {}", ingredient_name)));
    dialog.set_modal(true);
    dialog.set_default_width(400);

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

    // Name field
    let name_box = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
    let name_label = gtk::Label::new(Some("Name:"));
    name_label.set_halign(gtk::Align::Start);
    name_label.set_width_chars(12);
    let name_entry = gtk::Entry::new();
    name_entry.set_text(&ingredient.name);
    name_entry.set_hexpand(true);
    name_box.append(&name_label);
    name_box.append(&name_entry);
    content_area.append(&name_box);

    // Name (plural) field
    let plural_box = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
    let plural_label = gtk::Label::new(Some("Name (plural):"));
    plural_label.set_halign(gtk::Align::Start);
    plural_label.set_width_chars(12);
    let plural_entry = gtk::Entry::new();
    // Pre-fill with translations.en.other if available, else empty
    if let Some(translations) = &ingredient.translations {
        if let Some(forms) = translations.get("en") {
            plural_entry.set_text(&forms.other);
        }
    }
    plural_entry.set_hexpand(true);
    plural_box.append(&plural_label);
    plural_box.append(&plural_entry);
    content_area.append(&plural_box);

    // Category field
    let category_box = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
    let category_label = gtk::Label::new(Some("Category:"));
    category_label.set_halign(gtk::Align::Start);
    category_label.set_width_chars(12);
    let category_entry = gtk::Entry::new();
    category_entry.set_text(&ingredient.category);
    category_entry.set_hexpand(true);
    category_box.append(&category_label);
    category_box.append(&category_entry);
    content_area.append(&category_box);

    // KB field
    let kb_box = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
    let kb_label = gtk::Label::new(Some("KB Reference:"));
    kb_label.set_halign(gtk::Align::Start);
    kb_label.set_width_chars(12);
    let kb_entry = gtk::Entry::new();
    if let Some(kb) = &ingredient.kb {
        kb_entry.set_text(kb);
    }
    kb_entry.set_hexpand(true);
    kb_box.append(&kb_label);
    kb_box.append(&kb_entry);
    content_area.append(&kb_box);

    // Tags field (comma-separated)
    let tags_box = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
    let tags_label = gtk::Label::new(Some("Tags:"));
    tags_label.set_halign(gtk::Align::Start);
    tags_label.set_width_chars(12);
    let tags_entry = gtk::Entry::new();
    tags_entry.set_text(&ingredient.tags.clone().unwrap_or_default().join(", "));
    tags_entry.set_hexpand(true);
    tags_box.append(&tags_label);
    tags_box.append(&tags_entry);
    content_area.append(&tags_box);

    // Separator
    content_area.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

    // Pantry "In Stock" checkbox
    let in_stock_check = gtk::CheckButton::with_label("In Stock");
    in_stock_check.set_active(pantry_item.is_some());
    content_area.append(&in_stock_check);

    // Pantry quantity fields
    let quantity_box = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
    let quantity_label = gtk::Label::new(Some("Quantity:"));
    quantity_label.set_halign(gtk::Align::Start);
    quantity_label.set_width_chars(12);
    let quantity_entry = gtk::Entry::new();
    quantity_box.append(&quantity_label);
    quantity_box.append(&quantity_entry);
    content_area.append(&quantity_box);

    // Quantity type field
    let quantity_type_box = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
    let quantity_type_label = gtk::Label::new(Some("Unit:"));
    quantity_type_label.set_halign(gtk::Align::Start);
    quantity_type_label.set_width_chars(12);
    let quantity_type_entry = gtk::Entry::new();
    quantity_type_box.append(&quantity_type_label);
    quantity_type_box.append(&quantity_type_entry);
    content_area.append(&quantity_type_box);

    // Set pantry values if they exist
    if let Some(pantry_item) = pantry_item {
        if let Some(qty) = pantry_item.quantity {
            quantity_entry.set_text(&qty.to_string());
        }
        quantity_type_entry.set_text(&pantry_item.quantity_type);
    }

    // Disable quantity/unit fields if not in stock
    let set_qty_unit_sensitive = |enabled: bool| {
        quantity_entry.set_sensitive(enabled);
        quantity_type_entry.set_sensitive(enabled);
    };
    set_qty_unit_sensitive(in_stock_check.is_active());
    let quantity_entry_clone = quantity_entry.clone();
    let quantity_type_entry_clone = quantity_type_entry.clone();
    in_stock_check.connect_toggled(move |check| {
        let enabled = check.is_active();
        quantity_entry_clone.set_sensitive(enabled);
        quantity_type_entry_clone.set_sensitive(enabled);
    });

    // Create clones for the closure
    let sender_clone = sender.clone();
    let ingredient_name_clone = ingredient_name.clone();
    let data_manager_clone = data_manager.clone();

    dialog.add_button("Cancel", gtk::ResponseType::Cancel);
    dialog.add_button("Save", gtk::ResponseType::Accept);

    dialog.connect_response(move |dialog, response| {
        if response == gtk::ResponseType::Accept {
            let new_name = name_entry.text().to_string();
            let new_plural = plural_entry.text().to_string();
            if new_name.trim().is_empty() {
                let error_dialog = gtk::MessageDialog::new(
                    None::<&gtk::Window>,
                    gtk::DialogFlags::MODAL,
                    gtk::MessageType::Error,
                    gtk::ButtonsType::Ok,
                    "Ingredient name cannot be empty!",
                );
                error_dialog.connect_response(|dialog, _| dialog.destroy());
                error_dialog.show();
                return;
            }
            let new_category = category_entry.text().to_string();
            let new_kb = if kb_entry.text().is_empty() {
                None
            } else {
                Some(kb_entry.text().to_string())
            };
            let new_tags = tags_entry
                .text()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<String>>();
            // Write translations.en.one and .other
            let mut translations = std::collections::HashMap::new();
            let en_map = cookbook_engine::TranslationForms {
                one: new_name.clone(),
                other: if !new_plural.trim().is_empty() { new_plural.clone() } else { new_name.clone() },
            };
            translations.insert("en".to_string(), en_map);
            let new_ingredient = cookbook_engine::Ingredient {
                name: new_name.clone(),
                slug: new_name.clone(), // Use name as slug for now (should be slugified)
                category: new_category,
                kb: new_kb,
                tags: Some(new_tags),
                translations: Some(translations),
            };
            let in_stock = in_stock_check.is_active();
            let quantity_text = quantity_entry.text().to_string();
            let quantity = if quantity_text.is_empty() {
                None
            } else {
                match quantity_text.parse::<f64>() {
                    Ok(val) => Some(val),
                    Err(_) => None,
                }
            };
            let quantity_type = Some(quantity_type_entry.text().to_string());
            if let Some(rc_dm) = &data_manager_clone {
                let original_name = ingredient_name_clone.clone();
                let ingredient_clone = new_ingredient.clone();
                let sender_clone2 = sender_clone.clone();
                let data_dir = rc_dm.get_data_dir().to_path_buf();
                if original_name.trim().is_empty() {
                    // ADD: create ingredient, then update pantry if needed
                    let dm_result = cookbook_engine::DataManager::new(&data_dir);
                    if let Ok(mut dm) = dm_result {
                        let create_result = dm.create_ingredient(ingredient_clone.clone());
                        let pantry_result = if in_stock {
                            dm.update_pantry_item(
                                &ingredient_clone.name,
                                quantity,
                                quantity_type.clone(),
                            )
                        } else {
                            Ok(true)
                        };
                        match (create_result, pantry_result) {
                            (Ok(_), Ok(_)) => {
                                sender_clone2.input(crate::types::AppMsg::SelectIngredient(
                                    ingredient_clone.name.clone(),
                                ));
                                sender_clone2.input(crate::types::AppMsg::ReloadPantry);
                            }
                            (Err(err), _) | (_, Err(err)) => {
                                let error_dialog = gtk::MessageDialog::new(
                                    None::<&gtk::Window>,
                                    gtk::DialogFlags::MODAL,
                                    gtk::MessageType::Error,
                                    gtk::ButtonsType::Ok,
                                    &format!("Failed to add ingredient: {}", err),
                                );
                                error_dialog.connect_response(|dialog, _| dialog.destroy());
                                error_dialog.show();
                            }
                        }
                    } else {
                        let error_dialog = gtk::MessageDialog::new(
                            None::<&gtk::Window>,
                            gtk::DialogFlags::MODAL,
                            gtk::MessageType::Error,
                            gtk::ButtonsType::Ok,
                            &format!("Failed to initialize DataManager for add ingredient."),
                        );
                        error_dialog.connect_response(|dialog, _| dialog.destroy());
                        error_dialog.show();
                    }
                } else {
                    // UPDATE: update ingredient, then update pantry if needed
                    let dm_result = cookbook_engine::DataManager::new(&data_dir);
                    if let Ok(mut dm) = dm_result {
                        let update_result =
                            dm.update_ingredient(&original_name, ingredient_clone.clone());
                        let pantry_result = if in_stock {
                            dm.update_pantry_item(
                                &ingredient_clone.name,
                                quantity,
                                quantity_type.clone(),
                            )
                        } else {
                            dm.remove_from_pantry(&ingredient_clone.name)
                        };
                        match (update_result, pantry_result) {
                            (Ok(_), Ok(_)) => {
                                sender_clone2.input(crate::types::AppMsg::SelectIngredient(
                                    ingredient_clone.name.clone(),
                                ));
                                sender_clone2.input(crate::types::AppMsg::ReloadPantry);
                            }
                            (Err(err), _) | (_, Err(err)) => {
                                let error_dialog = gtk::MessageDialog::new(
                                    None::<&gtk::Window>,
                                    gtk::DialogFlags::MODAL,
                                    gtk::MessageType::Error,
                                    gtk::ButtonsType::Ok,
                                    &format!("Failed to update ingredient: {}", err),
                                );
                                error_dialog.connect_response(|dialog, _| dialog.destroy());
                                error_dialog.show();
                            }
                        }
                    } else {
                        let error_dialog = gtk::MessageDialog::new(
                            None::<&gtk::Window>,
                            gtk::DialogFlags::MODAL,
                            gtk::MessageType::Error,
                            gtk::ButtonsType::Ok,
                            &format!("Failed to initialize DataManager for update ingredient."),
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
