use relm4::gtk;
use crate::types::Tab;

/// Updates the main stack to show the currently selected tab
pub fn update_tab_view(current_tab: &Tab, main_stack: &gtk::Stack) {
    match current_tab {
        Tab::Recipes => main_stack.set_visible_child_name("recipes"),
        Tab::Pantry => main_stack.set_visible_child_name("pantry"),
        Tab::KnowledgeBase => main_stack.set_visible_child_name("kb"),
        Tab::Settings => main_stack.set_visible_child_name("settings"),
    }
}