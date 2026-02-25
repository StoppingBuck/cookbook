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

/// Updates the Recipe List Pane with filtered recipes based on search text and other filters
pub fn update_recipes_list<C>(
    recipes_list_box: &gtk::ListBox,
    data_manager: &Option<Rc<DataManager>>,
    search_text: &str,
    selected_recipe: Option<&String>,
    sender: Option<&ComponentSender<C>>,
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
                row.set_selectable(true);
                row.add_css_class("recipe-row");
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
                    let in_stock_label = gtk::Label::new(Some("(in stock)"));
                    in_stock_label.add_css_class("in-stock-label");
                    in_stock_label.set_halign(gtk::Align::End);
                    box_layout.append(&in_stock_label);
                }

                row.set_child(Some(&box_layout));

                // Add click handler
                let sender_clone = sender.cloned();
                let recipe_title = recipe.title.clone();
                let select_msg = select_recipe_msg.clone();

                let click_gesture = gtk::GestureClick::new();
                row.add_controller(click_gesture.clone());
                click_gesture.connect_pressed(move |_, _, _, _| {
                    if let Some(sender) = &sender_clone {
                        sender.input(select_msg(recipe_title.clone()));
                    }
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

/// Refreshes the recipes list and details view (for ReloadRecipes)
pub fn refresh_recipes_ui(
    model: &AppModel,
    widgets: &mut crate::types::AppWidgets,
    sender: &ComponentSender<AppModel>,
) {
    update_recipes_list(
        &widgets.recipes_list_box,
        &model.data_manager,
        &model.search_text,
        model.selected_recipe.as_ref(),
        Some(sender),
        AppMsg::SelectRecipe,
    );
    super::detail::update_recipe_details(
        model.selected_recipe.as_deref(),
        &widgets.recipes_details,
        &model.data_manager,
        Some(sender),
        AppMsg::EditRecipe,
    );
}
