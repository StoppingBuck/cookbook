use crate::i18n::tr;
use crate::types::AppModel;
use crate::types::AppMsg;
use crate::ui_constants::*;
use crate::utils;
use cookbook_engine::DataManager;
use gtk::prelude::*;
use relm4::gtk;
use relm4::ComponentSender;
use relm4::RelmIterChildrenExt;
use relm4::RelmWidgetExt;
use std::rc::Rc;

/// Rebuilds the Pantry List Pane with filtered ingredients based on search text and category filters
pub fn rebuild_pantry_list<C>(
    pantry_list: &gtk::ListBox,
    search_text: &str,
    selected_categories: &[String],
    show_in_stock_only: bool,
    select_ingredient_msg: impl Fn(String) -> <C as relm4::Component>::Input + Clone + 'static,
    model: &AppModel,
    sender: Option<ComponentSender<C>>,
    pantry_details_box: Option<&gtk::Box>,
    mut pantry_row_map: Option<&mut std::collections::HashMap<String, gtk::ListBoxRow>>, // NEW
) where
    C: relm4::Component + 'static,
{
    // Clear current list
    while let Some(child) = pantry_list.first_child() {
        pantry_list.remove(&child);
    }

    let lang = "en"; // TODO: get from model/user_settings
    let dm = match &model.data_manager {
        Some(dm) => dm,
        None => {
            let row = gtk::ListBoxRow::new();
            let no_data_label = gtk::Label::new(Some(&tr("No ingredient data available")));
            no_data_label.set_margin_all(DEFAULT_MARGIN);
            row.set_child(Some(&no_data_label));
            pantry_list.append(&row);
            return;
        }
    };

    let filtered_ingredients =
        dm.filter_ingredients(search_text, selected_categories, show_in_stock_only, lang);
    let mut pantry_items_by_category: std::collections::HashMap<
        String,
        Vec<(String, String, Option<String>, Option<String>, bool)>,
    > = std::collections::HashMap::new();

    for ingredient in &filtered_ingredients {
        let is_in_stock = dm.is_in_pantry(&ingredient.name);
        let pantry_item = dm.get_pantry_item(&ingredient.name);
        let quantity = pantry_item.and_then(|item| item.quantity);
        let quantity_type = pantry_item.and_then(|item| {
            if item.quantity_type.is_empty() { None } else { Some(item.quantity_type.clone()) }
        });
        // Pluralization logic
        let use_plural = if is_in_stock {
            match (quantity, quantity_type.as_deref()) {
                (Some(q), Some(unit)) if !unit.is_empty() => q > 0.0,
                (Some(q), None) | (Some(q), Some(_)) => q > 1.0,
                _ => false,
            }
        } else {
            false
        };
        let display_name = if let Some(translations) = &ingredient.translations {
            if let Some(forms) = translations.get(lang) {
                if use_plural {
                    forms.other.clone()
                } else {
                    forms.one.clone()
                }
            } else {
                ingredient.name.clone()
            }
        } else {
            ingredient.name.clone()
        };
        let slug = ingredient.slug.clone();
        pantry_items_by_category
            .entry(ingredient.category.clone())
            .or_default()
            .push((
                display_name,
                slug,
                quantity.map(|q| q.to_string()),
                quantity_type,
                is_in_stock,
            ));
    }

    let mut sorted_categories: Vec<String> = pantry_items_by_category.keys().cloned().collect();
    sorted_categories.sort();

    if pantry_items_by_category.is_empty() {
        let row = gtk::ListBoxRow::new();
        let no_items_label = gtk::Label::new(Some(&tr("No ingredients match the current filters")));
        no_items_label.set_margin_all(20);
        row.set_child(Some(&no_items_label));
        pantry_list.append(&row);
    } else {
        for category in sorted_categories {
            let category_row = gtk::ListBoxRow::new();
            let category_label = gtk::Label::new(None);
            category_label.set_markup(&format!("<b>{}</b>", category));
            category_label.set_halign(gtk::Align::Start);
            category_label.set_margin_top(DEFAULT_MARGIN);
            category_label.set_margin_bottom(LIST_ROW_MARGIN);
            category_row.set_child(Some(&category_label));
            pantry_list.append(&category_row);

            if let Some(items) = pantry_items_by_category.get_mut(&category) {
                items.sort_by(|a, b| a.0.cmp(&b.0));
                for (name, slug, quantity, quantity_type, is_in_stock) in items.iter() {
                    let row = gtk::ListBoxRow::new();
                    let item_row = gtk::Box::new(gtk::Orientation::Horizontal, TAG_SPACING);
                    item_row.set_margin_all(LIST_ROW_MARGIN);
                    item_row.add_css_class("pantry-item");

                    let mut label_text = name.clone();
                    if *is_in_stock {
                        if let Some(q) = quantity {
                            if let Some(t) = quantity_type.as_ref() {
                                if t.is_empty() {
                                    label_text = format!("{} ({})", name, q);
                                } else {
                                    label_text = format!("{} ({} {})", name, q, t);
                                }
                            } else {
                                label_text = format!("{} ({})", name, q);
                            }
                            label_text = format!("{} ✅", label_text);
                        } else {
                            label_text = format!("{} ❓", name);
                        }
                    }

                    let item_label = gtk::Label::new(Some(&label_text));
                    item_label.set_halign(gtk::Align::Start);
                    item_label.set_hexpand(true);
                    item_row.append(&item_label);

                    let chevron = gtk::Image::from_icon_name("go-next-symbolic");
                    chevron.set_pixel_size(16);
                    chevron.set_valign(gtk::Align::Center);
                    item_row.append(&chevron);

                    let click_gesture = gtk::GestureClick::new();
                    item_row.add_controller(click_gesture.clone());
                    let sender_clone = sender.clone();
                    let slug_clone = slug.clone();
                    let select_msg_clone = select_ingredient_msg.clone();
                    let pantry_details_box = pantry_details_box.cloned();
                    let dm_rc = dm.clone();
                    click_gesture.connect_pressed(move |_, _, _, _| {
                        if let Some(sender) = &sender_clone {
                            sender.input(select_msg_clone(slug_clone.clone()));
                        } else if let Some(details_box) = pantry_details_box.as_ref() {
                            // Directly update details pane in test mode
                            while let Some(child) = details_box.first_child() {
                                details_box.remove(&child);
                            }
                            // Use build_ingredient_detail_view with dummy sender
                            use relm4::ComponentSender;
                            let dummy_sender: &ComponentSender<AppModel> =
                                unsafe { std::mem::zeroed() };
                            let detail_view = crate::pantry::build_ingredient_detail_view(
                                &dm_rc,
                                &slug_clone,
                                dummy_sender,
                                |_| AppMsg::SwitchTab(crate::Tab::Pantry),
                                |_| AppMsg::SelectKnowledgeBaseEntry(String::new()),
                                |_| AppMsg::SelectRecipe(String::new()),
                                |_| AppMsg::EditIngredient(String::new()),
                                |_| AppMsg::DeleteIngredient(String::new()),
                            );
                            details_box.append(&detail_view);
                        }
                    });

                    row.set_child(Some(&item_row));
                    // Store the actual slug as row data for selection
                    unsafe {
                        row.set_data::<String>("slug", slug.clone());
                    }
                    // Populate pantry_row_map if provided
                    if let Some(ref mut map) = pantry_row_map {
                        map.insert(slug.clone(), row.clone());
                    }
                    pantry_list.append(&row);
                }
            }
        }
    }
    if filtered_ingredients.is_empty() {
        let row = gtk::ListBoxRow::new();
        let no_data_label = gtk::Label::new(Some(&tr("No ingredient data available")));
        no_data_label.set_margin_all(DEFAULT_MARGIN);
        row.set_child(Some(&no_data_label));
        pantry_list.append(&row);
    }
}

/// Builds and returns the Pantry Details Pane for a selected ingredient
pub fn build_ingredient_detail_view<C>(
    data_manager: &Rc<DataManager>,
    ingredient_id: &str, // now this is always a slug
    sender: &ComponentSender<C>,
    switch_tab_msg: impl Fn(crate::Tab) -> C::Input + Clone + 'static,
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
                sender_clone.input(switch_tab(crate::Tab::KnowledgeBase));
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
                    sender_clone.input(switch_tab(crate::Tab::Recipes));
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

/// Builds the Pantry tab UI and returns the main container, list container, details box, in-stock switch, and title label.
pub fn build_pantry_tab(
    model: &AppModel,
    sender: Option<ComponentSender<AppModel>>,
) -> (
    gtk::Box,                       // pantry_container
    gtk::ListBox,                   // pantry_list_container
    gtk::Box,                       // pantry_details_box
    gtk::Switch,                    // stock_filter_switch
    gtk::Label,                     // pantry_title
    Option<Box<dyn Fn(&AppModel)>>, // refresh_categories closure
) {
    // Main container for the Pantry tab
    let pantry_container = gtk::Box::new(gtk::Orientation::Vertical, SECTION_SPACING);

    // Title
    let pantry_title = gtk::Label::new(Some(&tr("Pantry")));
    pantry_title.set_markup(&format!(
        "<span size='x-large' weight='bold'>{}</span>",
        tr("Pantry")
    ));
    pantry_title.set_halign(gtk::Align::Start);
    pantry_title.set_margin_all(DEFAULT_MARGIN);

    // Filters frame
    let filters_frame = gtk::Frame::new(None);
    filters_frame.set_margin_bottom(DEFAULT_MARGIN);

    let filters_container = gtk::Box::new(gtk::Orientation::Vertical, TAG_SPACING);
    filters_container.set_margin_all(DEFAULT_MARGIN);

    // Category filters (popover multi-select)
    let category_filters_label = gtk::Label::new(Some(&tr("Categories:")));
    category_filters_label.set_halign(gtk::Align::Start);
    category_filters_label.set_margin_bottom(LIST_ROW_MARGIN);

    // Button to open popover (static, not recreated)
    let filter_button = gtk::Button::with_label(&tr("Filter Categories"));
    filter_button.set_halign(gtk::Align::Start);
    filter_button.set_tooltip_text(Some(&tr("Filter by one or more categories")));

    // Popover for category filters (static)
    let popover = gtk::Popover::new();
    popover.set_parent(&filter_button);
    // Ensure popover is closed when the parent button is finalized
    {
        let popover_weak = popover.downgrade();
        filter_button.connect_destroy(move |_| {
            if let Some(popover) = popover_weak.upgrade() {
                popover.popdown();
                popover.unparent();
            }
        });
    }
    let popover_box = gtk::Box::new(gtk::Orientation::Vertical, TAG_SPACING);
    popover_box.set_margin_all(DEFAULT_MARGIN);
    popover_box.set_vexpand(true);
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_min_content_height(180);
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    let listbox = gtk::Box::new(gtk::Orientation::Vertical, TAG_SPACING);
    scroll.set_child(Some(&listbox));
    popover_box.append(&scroll);
    popover.set_child(Some(&popover_box));

    // Function to refresh the listbox contents
    let refresh_categories = {
        let listbox = listbox.clone();
        let sender = sender.clone();
        move |categories: Vec<String>, selected_categories: Vec<String>| {
            println!(
                "DEBUG: [refresh_categories] Called with categories: {:?}, selected: {:?}",
                categories, selected_categories
            );
            while let Some(child) = listbox.first_child() {
                listbox.remove(&child);
            }
            for category in &categories {
                let check = gtk::CheckButton::with_label(category);
                check.set_active(selected_categories.contains(category));
                let sender_clone = sender.clone();
                let category_clone = category.clone();
                check.connect_toggled(move |btn| {
                    if let Some(sender) = &sender_clone {
                        sender.input(AppMsg::TogglePantryCategory(
                            category_clone.clone(),
                            btn.is_active(),
                        ));
                    }
                });
                listbox.append(&check);
            }
        }
    };
    // Initial population
    let categories = if let Some(ref dm) = model.data_manager {
        println!(
            "DEBUG: [build_pantry_tab] DataManager ptr: {:p}",
            Rc::as_ptr(dm)
        );
        let cats = dm.as_ref().get_unique_categories();
        println!("DEBUG: [build_pantry_tab] Initial categories: {:?}", cats);
        cats
    } else {
        println!("DEBUG: [build_pantry_tab] No DataManager available");
        Vec::new()
    };
    let selected_categories = model.selected_pantry_categories.clone();
    println!(
        "DEBUG: [build_pantry_tab] Selected categories: {:?}",
        selected_categories
    );
    refresh_categories(categories.clone(), selected_categories.clone());

    // Static wrapper closure for AppWidgets
    let listbox = listbox.clone();
    let sender = sender.clone();
    let sender_for_refresh_categories = sender.clone();
    let refresh_categories_boxed = Box::new(move |model: &AppModel| {
        if let Some(ref dm) = model.data_manager {
            println!(
                "DEBUG: [refresh_categories_boxed] DataManager ptr: {:p}",
                Rc::as_ptr(dm)
            );
            let categories = dm.as_ref().get_unique_categories();
            println!(
                "DEBUG: [refresh_categories_boxed] Refreshed categories: {:?}",
                categories
            );
            let selected_categories = model.selected_pantry_categories.clone();
            println!(
                "DEBUG: [refresh_categories_boxed] Selected categories: {:?}",
                selected_categories
            );
            while let Some(child) = listbox.first_child() {
                listbox.remove(&child);
            }
            for category in &categories {
                let check = gtk::CheckButton::with_label(category);
                check.set_active(selected_categories.contains(category));
                let sender_clone = sender_for_refresh_categories.clone();
                let category_clone = category.clone();
                check.connect_toggled(move |btn| {
                    if let Some(sender) = &sender_clone {
                        sender.input(AppMsg::TogglePantryCategory(
                            category_clone.clone(),
                            btn.is_active(),
                        ));
                    }
                });
                listbox.append(&check);
            }
        } else {
            println!("DEBUG: [refresh_categories_boxed] No DataManager available");
            while let Some(child) = listbox.first_child() {
                listbox.remove(&child);
            }
        }
    });
    // Open popover on button click
    filter_button.connect_clicked(move |_| {
        popover.popup();
    });

    // Add refresh button next to filter_button
    let refresh_button = gtk::Button::new();
    let refresh_icon = gtk::Image::from_icon_name("view-refresh-symbolic");
    refresh_icon.set_pixel_size(16);
    refresh_button.set_child(Some(&refresh_icon));
    refresh_button.set_tooltip_text(Some(&tr("Refresh category list")));
    let sender_for_refresh = sender.clone();
    refresh_button.connect_clicked(move |_| {
        if let Some(sender) = &sender_for_refresh {
            sender.input(AppMsg::RefreshCategoryPopover);
        }
    });

    // Update button label to show number of selected categories
    let selected_count = model.selected_pantry_categories.len();
    if selected_count > 0 {
        filter_button.set_label(&format!("{} ({})", tr("Filter Categories"), selected_count));
    }

    // In-stock only filter
    let stock_filter_box = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
    let stock_filter_label = gtk::Label::new(Some(&tr("Show in-stock items only:")));
    stock_filter_label.set_halign(gtk::Align::Start);

    let stock_filter_switch = gtk::Switch::new();
    stock_filter_switch.set_state(model.show_in_stock_only);
    let sender_clone = sender.clone();
    stock_filter_switch.connect_state_notify(move |switch| {
        if let Some(sender) = &sender_clone {
            sender.input(AppMsg::ToggleInStockFilter(switch.state()));
        }
    });

    stock_filter_box.append(&stock_filter_label);
    stock_filter_box.append(&stock_filter_switch);

    // Add filter_button and refresh_button to a horizontal box
    let filter_buttons_box = gtk::Box::new(gtk::Orientation::Horizontal, TAG_SPACING);
    filter_buttons_box.append(&filter_button);
    filter_buttons_box.append(&refresh_button);
    filters_container.append(&category_filters_label);
    filters_container.append(&filter_buttons_box);
    filters_container.append(&stock_filter_box);

    filters_frame.set_child(Some(&filters_container));
    pantry_container.append(&pantry_title);
    pantry_container.append(&filters_frame);

    // Pantry Tab UI Structure:
    // - Pantry List Pane (middle): shows all ingredients
    // - Pantry Details Pane (right): shows details for selected ingredient
    // - Navigation Pane (left): handled by main app sidebar, not here
    // The panes are uncoupled except:
    //   - Selecting an ingredient in the List Pane updates the Details Pane
    //   - Changing tab in Navigation triggers List Pane update

    let pantry_container = gtk::Box::new(gtk::Orientation::Vertical, SECTION_SPACING);

    // Title
    let pantry_title = gtk::Label::new(Some(&tr("Pantry")));
    pantry_title.set_markup(&format!(
        "<span size='x-large' weight='bold'>{}</span>",
        tr("Pantry")
    ));
    pantry_title.set_halign(gtk::Align::Start);
    pantry_title.set_margin_all(DEFAULT_MARGIN);

    // Filters frame
    let filters_frame = gtk::Frame::new(None);
    filters_frame.set_margin_bottom(DEFAULT_MARGIN);

    let filters_container = gtk::Box::new(gtk::Orientation::Vertical, TAG_SPACING);
    filters_container.set_margin_all(DEFAULT_MARGIN);

    // Category filters (popover multi-select)
    let category_filters_label = gtk::Label::new(Some(&tr("Categories:")));
    category_filters_label.set_halign(gtk::Align::Start);
    category_filters_label.set_margin_bottom(LIST_ROW_MARGIN);

    // Button to open popover (static, not recreated)
    let filter_button = gtk::Button::with_label(&tr("Filter Categories"));
    filter_button.set_halign(gtk::Align::Start);
    filter_button.set_tooltip_text(Some(&tr("Filter by one or more categories")));

    // Popover for category filters (static)
    let popover = gtk::Popover::new();
    popover.set_parent(&filter_button);
    {
        let popover_weak = popover.downgrade();
        filter_button.connect_destroy(move |_| {
            if let Some(popover) = popover_weak.upgrade() {
                popover.popdown();
                popover.unparent();
            }
        });
    }

    let popover_box = gtk::Box::new(gtk::Orientation::Vertical, TAG_SPACING);
    popover_box.set_margin_all(DEFAULT_MARGIN);
    popover_box.set_vexpand(true);
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_min_content_height(180);
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    let listbox = gtk::Box::new(gtk::Orientation::Vertical, TAG_SPACING);
    scroll.set_child(Some(&listbox));
    popover_box.append(&scroll);
    popover.set_child(Some(&popover_box));

    // Function to refresh the listbox contents
    let refresh_categories = {
        let listbox = listbox.clone();
        let sender = sender.clone();
        move |categories: Vec<String>, selected_categories: Vec<String>| {
            while let Some(child) = listbox.first_child() {
                listbox.remove(&child);
            }
            for category in &categories {
                let check = gtk::CheckButton::with_label(category);
                check.set_active(selected_categories.contains(category));
                let sender_clone = sender.clone();
                let category_clone = category.clone();
                check.connect_toggled(move |btn| {
                    if let Some(sender) = &sender_clone {
                        sender.input(AppMsg::TogglePantryCategory(
                            category_clone.clone(),
                            btn.is_active(),
                        ));
                    }
                });
                listbox.append(&check);
            }
        }
    };
    let categories = if let Some(ref dm) = model.data_manager {
        let cats = dm.as_ref().get_unique_categories();
        cats
    } else {
        Vec::new()
    };
    let selected_categories = model.selected_pantry_categories.clone();
    refresh_categories(categories.clone(), selected_categories.clone());

    let listbox = listbox.clone();
    let sender = sender.clone();
    let sender_for_refresh_categories = sender.clone();
    let refresh_categories_boxed = Box::new(move |model: &AppModel| {
        if let Some(ref dm) = model.data_manager {
            let categories = dm.as_ref().get_unique_categories();
            let selected_categories = model.selected_pantry_categories.clone();
            while let Some(child) = listbox.first_child() {
                listbox.remove(&child);
            }
            for category in &categories {
                let check = gtk::CheckButton::with_label(category);
                check.set_active(selected_categories.contains(category));
                let sender_clone = sender_for_refresh_categories.clone();
                let category_clone = category.clone();
                check.connect_toggled(move |btn| {
                    if let Some(sender) = &sender_clone {
                        sender.input(AppMsg::TogglePantryCategory(
                            category_clone.clone(),
                            btn.is_active(),
                        ));
                    }
                });
                listbox.append(&check);
            }
        } else {
            while let Some(child) = listbox.first_child() {
                listbox.remove(&child);
            }
        }
    });
    filter_button.connect_clicked(move |_| {
        popover.popup();
    });

    let refresh_button = gtk::Button::new();
    let refresh_icon = gtk::Image::from_icon_name("view-refresh-symbolic");
    refresh_icon.set_pixel_size(16);
    refresh_button.set_child(Some(&refresh_icon));
    refresh_button.set_tooltip_text(Some(&tr("Refresh category list")));
    let sender_for_refresh = sender.clone();
    refresh_button.connect_clicked(move |_| {
        if let Some(sender) = &sender_for_refresh {
            sender.input(AppMsg::RefreshCategoryPopover);
        }
    });

    let selected_count = model.selected_pantry_categories.len();
    if selected_count > 0 {
        filter_button.set_label(&format!("{} ({})", tr("Filter Categories"), selected_count));
    }

    let stock_filter_box = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
    let stock_filter_label = gtk::Label::new(Some(&tr("Show in-stock items only:")));
    stock_filter_label.set_halign(gtk::Align::Start);

    let stock_filter_switch = gtk::Switch::new();
    stock_filter_switch.set_state(model.show_in_stock_only);
    let sender_clone = sender.clone();
    stock_filter_switch.connect_state_notify(move |switch| {
        if let Some(sender) = &sender_clone {
            sender.input(AppMsg::ToggleInStockFilter(switch.state()));
        }
    });

    stock_filter_box.append(&stock_filter_label);
    stock_filter_box.append(&stock_filter_switch);

    let filter_buttons_box = gtk::Box::new(gtk::Orientation::Horizontal, TAG_SPACING);
    filter_buttons_box.append(&filter_button);
    filter_buttons_box.append(&refresh_button);

    filters_container.append(&category_filters_label);
    filters_container.append(&filter_buttons_box);
    filters_container.append(&stock_filter_box);

    filters_frame.set_child(Some(&filters_container));
    pantry_container.append(&pantry_title);
    pantry_container.append(&filters_frame);

    // Split view: Pantry List Pane (middle), Pantry Details Pane (right)
    let pantry_content = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
    pantry_content.set_hexpand(true);
    pantry_content.set_vexpand(true);
    pantry_content.set_margin_top(DEFAULT_MARGIN);
    pantry_content.set_margin_start(DEFAULT_MARGIN);
    pantry_content.set_margin_end(DEFAULT_MARGIN);
    pantry_content.set_margin_bottom(DEFAULT_MARGIN);

    // Pantry List Pane
    let pantry_list_scroll = gtk::ScrolledWindow::new();
    pantry_list_scroll.set_hexpand(false);
    pantry_list_scroll.set_vexpand(true);
    pantry_list_scroll.set_min_content_width(300);

    let pantry_list_pane = gtk::ListBox::new();

    pantry_list_scroll.set_child(Some(&pantry_list_pane));

    // Pantry Details Pane
    let pantry_details_pane = gtk::Box::new(gtk::Orientation::Vertical, SECTION_SPACING);
    pantry_details_pane.set_hexpand(true);
    pantry_details_pane.set_vexpand(true);

    let select_label = gtk::Label::new(Some(&tr("Select an ingredient to view details")));
    select_label.set_halign(gtk::Align::Center);
    select_label.set_valign(gtk::Align::Center);
    select_label.set_hexpand(true);
    select_label.set_vexpand(true);
    pantry_details_pane.append(&select_label);

    pantry_content.append(&pantry_list_scroll);
    pantry_content.append(&pantry_details_pane);

    pantry_container.append(&pantry_content);

    // Add button for new ingredient
    let add_button = gtk::Button::with_label(&tr("Add Ingredient"));
    add_button.set_halign(gtk::Align::End);
    let sender_clone = sender.clone();
    add_button.connect_clicked(move |_| {
        if let Some(sender) = &sender_clone {
            sender.input(AppMsg::AddIngredient);
        }
    });
    pantry_container.append(&add_button);

    // --- DEBUG: Breadcrumb for ListBox selection ---
    println!("DEBUG: [build_pantry_tab] Setting up ListBox selection handler");
    let pantry_details_box_clone = pantry_details_pane.clone();
    // let model_clone = model.clone();
    let sender_clone = sender.clone();
    use std::cell::RefCell;
    use std::rc::Rc;
    let selected_ingredient = Rc::new(RefCell::new(model.selected_ingredient.clone()));
    let selected_ingredient_ref = selected_ingredient.clone();
    pantry_list_pane.connect_row_selected(move |_listbox, row_opt| {
        println!("DEBUG: [ListBox] row_selected triggered");
        if let Some(row) = row_opt {
            // Retrieve the actual slug from row data
            if let Some(nn_slug) = unsafe { row.data::<String>("slug") } {
                let slug_ref = unsafe { nn_slug.as_ref() };
                println!("DEBUG: [ListBox] Selected slug={:?}", slug_ref);
                // Only send SelectIngredient if the selected slug is different from the model's selected_ingredient
                if let Some(sender) = &sender_clone {
                    if selected_ingredient_ref.borrow().as_deref() != Some(slug_ref.as_str()) {
                        sender.input(AppMsg::SelectIngredient(slug_ref.clone()));
                        *selected_ingredient_ref.borrow_mut() = Some(slug_ref.clone());
                        println!("DEBUG: [ListBox] Sent SelectIngredient message");
                    } else {
                        println!(
                            "DEBUG: [ListBox] Ingredient already selected, not sending message"
                        );
                    }
                }
            } else {
                println!("DEBUG: [ListBox] Selected row has no slug data");
            }
        } else {
            println!("DEBUG: [ListBox] No row selected");
        }
    });

    (
        pantry_container,
        pantry_list_pane,
        pantry_details_pane,
        stock_filter_switch,
        pantry_title,
        Some(refresh_categories_boxed),
    )
}
