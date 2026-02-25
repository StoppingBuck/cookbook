use crate::i18n::tr;
use crate::types::AppModel;
use crate::types::AppMsg;
use crate::ui_constants::*;
use gtk::prelude::*;
use relm4::gtk;
use relm4::ComponentSender;
use relm4::RelmWidgetExt;

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
                                |_| AppMsg::SwitchTab(crate::types::Tab::Pantry),
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
