use relm4::gtk;
use relm4::ComponentSender;

use crate::types::{AppModel, AppMsg, Tab};
use crate::kb;
use crate::pantry;
use crate::recipes;

/// Updates the UI based on the currently selected tab
pub fn update_tab_content(
    current_tab: &Tab,
    previous_tab: Option<&Tab>,
    widgets: &mut crate::types::AppWidgets,
    model: &AppModel,
    sender: &ComponentSender<AppModel>
) {
    // Set the visible stack page based on the current tab
    match current_tab {
        Tab::Recipes => widgets.main_stack.set_visible_child_name("recipes"),
        Tab::Pantry => widgets.main_stack.set_visible_child_name("pantry"),
        Tab::KnowledgeBase => widgets.main_stack.set_visible_child_name("kb"),
        Tab::Settings => widgets.main_stack.set_visible_child_name("settings"),
    }
    
    // Update content based on the current tab
    match current_tab {
        Tab::Recipes => {
            // Update recipes list and details
            recipes::update_recipes_list(
                &widgets.recipes_list_box,
                &model.data_manager,
                &model.search_text,
                &sender,
                AppMsg::SelectRecipe,
            );
            
            recipes::update_recipe_details(
                model.selected_recipe.as_deref(),
                &widgets.recipes_details,
                &model.data_manager,
                &sender,
                AppMsg::EditRecipe,
            );
        },
        Tab::Pantry => {
            // Rebuild pantry list when filters change or search text changes
            pantry::rebuild_pantry_list(
                &widgets.pantry_list,
                &model.data_manager,
                &model.search_text,
                &model.selected_pantry_categories,
                model.show_in_stock_only,
                &sender,
                AppMsg::SelectIngredient,
            );
            
            // Update pantry details based on selection
            if let Some(ingredient_name) = &model.selected_ingredient {
                pantry::update_pantry_details(
                    Some(ingredient_name.as_str()),
                    &widgets.pantry_details,
                    &model.data_manager,
                    &sender,
                );
            } else {
                pantry::update_pantry_details(
                    None,
                    &widgets.pantry_details,
                    &model.data_manager,
                    &sender,
                );
            }
        },
        Tab::KnowledgeBase => {
            // Only rebuild the KB list when first switching to the tab
            let tab_changed = previous_tab.map_or(true, |prev| prev != current_tab);
            
            if tab_changed {
                kb::update_kb_list(
                    &widgets.kb_list_box,
                    &model.data_manager,
                    &sender,
                    AppMsg::SelectKnowledgeBaseEntry,
                );
            }
            
            // Always update KB details
            if let Some(kb_slug) = &model.selected_kb_entry {
                // Update the selection in the list
                kb::select_kb_entry_in_list(&widgets.kb_list_box, kb_slug);
                
                // Update the details view
                kb::update_kb_details::<AppModel>(
                    &widgets.kb_details,
                    &model.data_manager,
                    kb_slug,
                    &model.data_dir,
                );
            } else {
                // No KB entry selected, show placeholder
                kb::show_kb_details_placeholder(&widgets.kb_details);
            }
        },
        Tab::Settings => {
            // Currently nothing to update for the settings tab
        }
    }
}
/// Updates the main stack to show the currently selected tab
pub fn update_tab_view(current_tab: &Tab, main_stack: &gtk::Stack) {
    match current_tab {
        Tab::Recipes => main_stack.set_visible_child_name("recipes"),
        Tab::Pantry => main_stack.set_visible_child_name("pantry"),
        Tab::KnowledgeBase => main_stack.set_visible_child_name("kb"),
        Tab::Settings => main_stack.set_visible_child_name("settings"),
    }
}