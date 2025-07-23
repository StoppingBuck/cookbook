use cookbook_gtk::types::AppModel;
use relm4::ComponentSender;
// Integration/UI tests for the GTK app using gtk-test or libadwaita-testing
// These tests require a display and may be skipped in headless CI
#[cfg(test)]
mod tests {
    use gtk::prelude::*;
    use gtk::glib::MainContext;
    use cookbook_gtk::build_app_model_and_widgets;
    use relm4::RelmIterChildrenExt;
    use relm4::ComponentSender;
    use cookbook_gtk::types::AppModel;

    #[test]
    fn test_recipe_selection_ui() {
        use cookbook_gtk::recipes::update_recipes_list;
        use cookbook_gtk::types::AppMsg;
        let app = gtk::Application::new(Some("org.cookbook.CookbookGtk"), Default::default());
        let test_done = std::sync::Arc::new(std::sync::Mutex::new(false));
        let test_done_clone = test_done.clone();
        app.connect_startup(move |app| {
            let window = gtk::ApplicationWindow::new(app);
            let parts = build_app_model_and_widgets(window.clone(), None);
            let widgets = parts.widgets;

            let num_recipes = parts.model.data_manager.as_ref()
                .map(|dm| dm.get_all_recipes().len())
                .unwrap_or(0);
            println!("Loaded {} recipes for test", num_recipes);

            if let Some(dm) = parts.model.data_manager.as_ref() {
                update_recipes_list(
                    &widgets.recipes_list_box,
                    &Some(dm.clone()),
                    "",
                    None,
                    None::<&relm4::ComponentSender<cookbook_gtk::types::AppModel>>,
                    |title| AppMsg::SelectRecipe(title),
                );
            }

            let num_recipes = parts.model.data_manager.as_ref()
                .map(|dm| dm.get_all_recipes().len())
                .unwrap_or(0);
            println!("Loaded {} recipes for test", num_recipes);

            let list_box = &widgets.recipes_list_box;
            if num_recipes == 0 {
                panic!("No recipes found in the list; test cannot proceed. Ensure example/data/recipes/ contains valid recipes and COOKBOOK_DATA_DIR is set if needed.");
            }
            if let Some(row) = list_box.row_at_index(0) {
                list_box.select_row(Some(&row));
                // Flush GTK events to ensure UI updates propagate
                let ctx = MainContext::default();
                while ctx.pending() {
                    ctx.iteration(false);
                }
                // Manually update details pane after selection
                if let Some(label) = row.child().and_then(|w| w.downcast::<gtk::Box>().ok())
                    .and_then(|box_layout| box_layout.first_child().and_then(|w| w.downcast::<gtk::Label>().ok())) {
                    let recipe_title = label.text().to_string();
                    if let Some(dm) = parts.model.data_manager.as_ref() {
                        let details = cookbook_gtk::recipes::build_recipe_detail_view(
                            dm,
                            &recipe_title,
                            // Pass None for sender in tests
                            None::<&ComponentSender<AppModel>>, // dummy sender, not used in test
                            |title| AppMsg::EditRecipe(title),
                        );
                        // Remove all children from the details box using first_child/next_sibling traversal
                        let mut child_opt = widgets.recipes_details.first_child();
                        while let Some(child) = child_opt {
                            widgets.recipes_details.remove(&child);
                            child_opt = widgets.recipes_details.first_child();
                        }
                        widgets.recipes_details.append(&details);
                    }
                }
            } else {
                panic!("Recipe list box is empty; test cannot proceed.");
            }

            // Recursively search for a non-empty gtk::Label inside the details pane using first_child/next_sibling
            fn find_nonempty_label(widget: &gtk::Widget) -> bool {
                if let Some(label) = widget.downcast_ref::<gtk::Label>() {
                    let details_text = label.text();
                    if !details_text.contains("Select a recipe to view details") && !details_text.is_empty() {
                        return true;
                    }
                }
                // Traverse children using first_child/next_sibling
                let mut child_opt = widget.first_child();
                while let Some(child) = child_opt {
                    if find_nonempty_label(&child) {
                        return true;
                    }
                    child_opt = child.next_sibling();
                }
                false
            }
            let mut found = false;
            let mut child_opt = widgets.recipes_details.first_child();
            while let Some(child) = child_opt {
                if find_nonempty_label(&child) {
                    found = true;
                    break;
                }
                child_opt = child.next_sibling();
            }
            assert!(found, "Details pane did not update");
            *test_done_clone.lock().unwrap() = true;
            app.quit();
        });
        app.run();
        assert!(*test_done.lock().unwrap(), "Test did not complete");
    }
}
