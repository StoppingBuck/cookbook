use cookbook_engine::DataManager;
use gtk::prelude::*;
use relm4::gtk;
use relm4::ComponentSender;
use relm4::RelmWidgetExt;
use std::collections::HashMap;
use std::rc::Rc;

/// Rebuilds the pantry list with filtered ingredients based on search text and category filters
pub fn rebuild_pantry_list<C>(
    pantry_list: &gtk::Box,
    data_manager: &Option<Rc<DataManager>>,
    search_text: &str,
    selected_categories: &[String],
    show_in_stock_only: bool,
    sender: &ComponentSender<C>,
    select_ingredient_msg: impl Fn(String) -> C::Input + Clone + 'static,
) where
    C: relm4::Component,
{
    // Clear current list
    while let Some(child) = pantry_list.first_child() {
        pantry_list.remove(&child);
    }

    if let Some(ref dm) = data_manager {
        let _pantry = dm.get_pantry(); // Prefix with underscore to avoid unused variable warning

        // Use engine method to filter ingredients
        let filtered_ingredients = dm.filter_ingredients(
            search_text,
            selected_categories,
            show_in_stock_only,
        );

        // Convert filtered ingredients to the format expected by the UI
        let mut pantry_items_by_category: HashMap<
            String,
            Vec<(String, Option<String>, Option<String>, bool)>,
        > = HashMap::new();

        for ingredient in filtered_ingredients {
            let is_in_stock = dm.is_in_pantry(&ingredient.name);

            // Get quantity information if in pantry
            let (quantity, quantity_type) = 
                if let Some(pantry_item) = dm.get_pantry_item(&ingredient.name) {
                    (
                        pantry_item.quantity.map(|q| q.to_string()),
                        Some(pantry_item.quantity_type.clone()),
                    )
                } else {
                    (None, Some(String::new()))
                };

            pantry_items_by_category
                .entry(ingredient.category.clone())
                .or_default()
                .push((
                    ingredient.name.clone(),
                    quantity,
                    quantity_type,
                    is_in_stock,
                ));
        }

        // Sort categories and ingredients
        let mut sorted_categories: Vec<String> =
            pantry_items_by_category.keys().cloned().collect();
        sorted_categories.sort();

        if pantry_items_by_category.is_empty() {
            // No items match the filters
            let no_items_label =
                gtk::Label::new(Some("No ingredients match the current filters"));
            no_items_label.set_margin_all(20);
            pantry_list.append(&no_items_label);
        } else {
            for category in sorted_categories {
                // Create category header
                let category_frame = gtk::Frame::new(Some(&category));
                category_frame.set_margin_bottom(10);

                let category_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
                category_frame.set_child(Some(&category_box));

                if let Some(items) = pantry_items_by_category.get_mut(&category) {
                    // Sort ingredients alphabetically within category
                    items.sort_by(|a, b| a.0.cmp(&b.0));

                    for (name, quantity, quantity_type, is_in_stock) in items.iter() {
                        let item_row = gtk::Box::new(gtk::Orientation::Horizontal, 5);
                        item_row.set_margin_all(5);

                        // Create the item label with quantity if available
                        let mut label_text = name.clone();
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
                        }

                        // Add checkmark for in-stock items
                        if *is_in_stock {
                            label_text = format!("{} ✅", label_text);
                        }

                        let item_label = gtk::Label::new(Some(&label_text));
                        item_label.set_halign(gtk::Align::Start);
                        item_label.set_hexpand(true);
                        item_row.append(&item_label);

                        // Make the row selectable
                        let click_gesture = gtk::GestureClick::new();
                        item_row.add_css_class("pantry-item");
                        item_row.add_controller(click_gesture.clone());

                        let sender_clone = sender.clone();
                        let name_clone = name.clone();
                        let select_msg_clone = select_ingredient_msg.clone();
                        click_gesture.connect_pressed(move |_, _, _, _| {
                            sender_clone.input(select_msg_clone(name_clone.clone()));
                        });

                        category_box.append(&item_row);
                    }
                }

                pantry_list.append(&category_frame);
            }
        }
    } else {
        // No data available
        let no_data_label = gtk::Label::new(Some("No ingredient data available"));
        no_data_label.set_margin_all(10);
        pantry_list.append(&no_data_label);
    }
}

/// Builds and returns the pantry ingredient detail view for a selected ingredient
pub fn build_ingredient_detail_view<C>(
    data_manager: &Rc<DataManager>, 
    ingredient_name: &str,
    sender: &ComponentSender<C>,
    switch_tab_msg: impl Fn(crate::Tab) -> C::Input + Clone + 'static,
    select_kb_entry_msg: impl Fn(String) -> C::Input + Clone + 'static,
    select_recipe_msg: impl Fn(String) -> C::Input + Clone + 'static,
    edit_ingredient_msg: impl Fn(String) -> C::Input + Clone + 'static
) -> gtk::Box
where
    C: relm4::Component,
{
    // Create a small details view
    let details_container = gtk::Box::new(gtk::Orientation::Vertical, 10);
    details_container.set_margin_all(10);

    if let Some(ingredient) = data_manager.get_ingredient(ingredient_name) {
        // Title with ingredient name and edit button in a horizontal box
        let title_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        title_box.set_margin_bottom(10);

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

        details_container.append(&title_box);

        // Category
        let category_label = gtk::Label::new(None);
        category_label.set_markup(&format!("<b>Category:</b> {}", ingredient.category));
        category_label.set_halign(gtk::Align::Start);
        category_label.set_margin_bottom(5);
        details_container.append(&category_label);

        // Tags (if any)
        if let Some(ref tags) = ingredient.tags {
            if !tags.is_empty() {
                let tags_label = gtk::Label::new(None);
                tags_label.set_markup("<b>Tags:</b>");
                tags_label.set_halign(gtk::Align::Start);
                tags_label.set_margin_bottom(5);
                details_container.append(&tags_label);

                let tags_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
                tags_box.set_margin_start(10);
                tags_box.set_margin_bottom(10);

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
            let kb_box = gtk::Box::new(gtk::Orientation::Vertical, 5);
            kb_box.set_margin_top(10);
            kb_box.set_margin_bottom(10);

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
                stock_label.set_margin_top(10);

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
                } else {
                    stock_label.set_markup("<b>In stock:</b> Yes (quantity unknown)");
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
                updated_label.set_margin_bottom(10);
                details_container.append(&updated_label);
            } else {
                let not_in_stock_label = gtk::Label::new(None);
                not_in_stock_label.set_markup("<b>Not in pantry</b>");
                not_in_stock_label.set_halign(gtk::Align::Start);
                not_in_stock_label.set_margin_top(10);
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
            recipes_header.set_margin_top(15);
            recipes_header.set_margin_bottom(5);
            details_container.append(&recipes_header);

            let recipes_box = gtk::Box::new(gtk::Orientation::Vertical, 5);
            recipes_box.set_margin_start(10);

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
        let not_found_label = gtk::Label::new(Some(&format!(
            "Ingredient '{}' not found",
            ingredient_name
        )));
        not_found_label.set_halign(gtk::Align::Center);
        not_found_label.set_valign(gtk::Align::Center);
        details_container.append(&not_found_label);
    }
    
    details_container
}