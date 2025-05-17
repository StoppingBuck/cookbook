use cookbook_engine::DataManager;
use gtk::prelude::*;
use relm4::gtk;
use relm4::ComponentSender;
use relm4::RelmWidgetExt;
use std::rc::Rc;

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
    while let Some(child) = recipes_list_box.first_child() {
        recipes_list_box.remove(&child);
    }

    if let Some(ref dm) = data_manager {
        // Use engine method to search recipes
        let filtered_recipes = dm.search_recipes(search_text);

        if !filtered_recipes.is_empty() {
            for recipe in filtered_recipes {
                let row = gtk::ListBoxRow::new();
                let box_layout = gtk::Box::new(gtk::Orientation::Horizontal, 10);
                box_layout.set_margin_all(5);

                // Create recipe item with title and status icon
                let title_label = gtk::Label::new(Some(&recipe.title));
                title_label.set_halign(gtk::Align::Start);
                title_label.set_hexpand(true);
                box_layout.append(&title_label);

                // Add checkmark if all ingredients are in pantry
                if dm.are_all_ingredients_in_pantry(&recipe.title) {
                    let check_icon = gtk::Image::from_icon_name("emblem-ok-symbolic");
                    box_layout.append(&check_icon);
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
            no_recipes_label.set_margin_all(10);
            no_recipes_row.set_child(Some(&no_recipes_label));
            recipes_list_box.append(&no_recipes_row);
        }
    } else {
        // Data manager not available
        let no_data_row = gtk::ListBoxRow::new();
        let no_data_label = gtk::Label::new(Some("Failed to load recipe data"));
        no_data_label.set_margin_all(10);
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
        let recipe_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
        recipe_box.set_margin_all(15);
        
        // Header with title and edit button
        let header_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        
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
                let tags_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
                tags_box.set_margin_bottom(10);
                
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
        metadata_grid.set_margin_bottom(15);
        
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
        downtime_label.set_markup("<b>Downtime:</b>");
        downtime_label.set_halign(gtk::Align::Start);
        metadata_grid.attach(&downtime_label, 0, 1, 1, 1);
        
        let downtime_value = gtk::Label::new(Some(&format!("{} min", recipe.downtime.unwrap_or(0))));
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
        ingredients_header.set_margin_bottom(5);
        recipe_box.append(&ingredients_header);
        
        // Ingredients list in a frame
        let ingredients_frame = gtk::Frame::new(None);
        ingredients_frame.set_margin_bottom(15);
        
        let ingredients_list = gtk::Box::new(gtk::Orientation::Vertical, 0);
        ingredients_list.set_margin_all(10);
        
        // Check which ingredients are in pantry
        let pantry_items = data_manager.get_pantry()
            .map(|p| p.items.iter().map(|i| i.ingredient.clone()).collect::<Vec<String>>())
            .unwrap_or_default();
        
        for ingredient in &recipe.ingredients {
            let ingredient_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
            ingredient_box.set_margin_bottom(5);
            
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
        
        // Instructions section
        let instructions_header = gtk::Label::new(None);
        instructions_header.set_markup("<span size='large' weight='bold'>Instructions</span>");
        instructions_header.set_halign(gtk::Align::Start);
        instructions_header.set_margin_bottom(5);
        recipe_box.append(&instructions_header);
        
        // Instructions text
        let instructions_frame = gtk::Frame::new(None);
        let instructions_text = gtk::Label::new(Some(&recipe.instructions));
        instructions_text.set_wrap(true);
        instructions_text.set_halign(gtk::Align::Start);
        instructions_text.set_margin_all(10);
        instructions_frame.set_child(Some(&instructions_text));
        recipe_box.append(&instructions_frame);
        
        recipe_details_scroll.set_child(Some(&recipe_box));
    } else {
        // Recipe not found
        let not_found_label = gtk::Label::new(Some(&format!(
            "Recipe '{}' not found", 
            recipe_name
        )));
        not_found_label.set_halign(gtk::Align::Center);
        not_found_label.set_valign(gtk::Align::Center);
        recipe_details_scroll.set_child(Some(&not_found_label));
    }
    
    recipe_details_scroll
}