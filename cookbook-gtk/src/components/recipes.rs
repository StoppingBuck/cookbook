use cookbook_engine::{DataManager, Recipe};
use gtk4::{gio, glib, prelude::*, Button, Label, ListBox, Orientation, ScrolledWindow, Widget};
use relm4::{
    gtk4, Component, ComponentParts, ComponentSender, SimpleComponent,
};
use std::rc::Rc;

#[derive(Debug)]
pub struct RecipesModel {
    recipes: Vec<Recipe>,
    data_manager: Option<Rc<DataManager>>,
    selected_recipe: Option<Recipe>,
    search_term: String,
}

#[derive(Debug)]
pub enum RecipesInput {
    SelectRecipe(usize),
    UpdateDataManager(&'static DataManager),
    SearchChanged(String),
    AddRecipe,
    EditRecipe,
    DeleteRecipe,
    RefreshView,
    ForceRefresh,
}

#[derive(Debug)]
pub enum RecipesOutput {
    Quit,
}

pub struct RecipesComponent;

#[relm4::component(pub)]
impl SimpleComponent for RecipesComponent {
    type Init = Option<&'static DataManager>;
    type Input = RecipesInput;
    type Output = RecipesOutput;
    type Widgets = RecipesWidgets;

    fn init(
        data_manager: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let recipes = if let Some(dm) = data_manager {
            dm.get_all_recipes().to_vec()
        } else {
            Vec::new()
        };

        // Initialize with the first recipe selected if available
        let selected_recipe = if !recipes.is_empty() {
            println!("Initializing with first recipe selected: {}", recipes[0].title);
            println!("Recipe instructions: {}", recipes[0].instructions);
            Some(recipes[0].clone())
        } else {
            println!("No recipes available during initialization");
            None
        };

        let model = RecipesModel {
            recipes,
            data_manager: data_manager.map(Rc::from),
            selected_recipe,
            search_term: String::new(),
        };

        let widgets = view_output!();
        sender.input(RecipesInput::SelectRecipe(0));


        println!("Initializing recipes view with {} recipes", model.recipes.len());
        
        // Populate recipe list
        model.populate_recipe_list(&widgets);
        
        // Also update the recipe detail with the selected recipe
        model.update_recipe_detail(&widgets);

        // If we have at least one recipe, make sure it's selected in the UI
        if let Some(ref recipe) = model.selected_recipe {
            println!("Initial recipe selected: {}", recipe.title);
            // Find and select the row in the list that corresponds to this recipe
            for i in 0..widgets.recipe_list.n_items() {
                if let Some(row) = widgets.recipe_list.row_at_index(i) {
                    println!("Selecting row {} in recipe list", i);
                    widgets.recipe_list.select_row(Some(&row));
                    // Force update UI elements to show the recipe
                    widgets.recipe_detail_view.set_visible(true);
                    widgets.recipe_detail.set_visible(true);
                    break;
                }
            }
        }

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            RecipesInput::SelectRecipe(index) => {
                if index < self.recipes.len() {
                    println!("Selected recipe at index {}: {}", index, self.recipes[index].title);
                    self.selected_recipe = Some(self.recipes[index].clone());
                    println!("Recipe selected: {:?}", self.selected_recipe);
                    // View will be updated automatically by update_view
                } else {
                    println!("Recipe index {} out of bounds (max {})", index, self.recipes.len());
                }
            }
            RecipesInput::UpdateDataManager(data_manager) => {
                self.data_manager = Some(Rc::from(data_manager));
                self.recipes = data_manager.get_all_recipes().to_vec();
                self.selected_recipe = None;
                println!("Data manager updated, loaded {} recipes", self.recipes.len());
                // View will be updated automatically by update_view
            }
            RecipesInput::SearchChanged(term) => {
                self.search_term = term.clone();
                self.filter_recipes();
                println!("Search term changed to '{}', found {} matches", term, self.recipes.len());
                // View will be updated automatically by update_view
            }
            RecipesInput::AddRecipe => {
                // Not implemented yet - would open a dialog to create a new recipe
                println!("Add recipe requested (not implemented)");
            }
            RecipesInput::EditRecipe => {
                // Not implemented yet - would open a dialog to edit the selected recipe
                println!("Edit recipe requested (not implemented)");
            }
            RecipesInput::DeleteRecipe => {
                // Not implemented yet - would delete the selected recipe
                println!("Delete recipe requested (not implemented)");
            }
            RecipesInput::RefreshView => {
                // Force update of the detail view with the currently selected recipe
                println!("Refreshing recipe view");
                // No need to update self, just make sure the view will update
            }
            RecipesInput::ForceRefresh => {
                // Force a refresh of the UI
                println!("Force refreshing the recipe display");
            }
        }
    }
    
    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        println!("RecipesComponent::update_view called");
        
        // First update the recipe list if needed
        self.populate_recipe_list(widgets);
        
        // Then update the recipe detail view
        self.update_recipe_detail(widgets);
        
        // Debug: Print when view is updated and which recipe is selected
        if let Some(recipe) = &self.selected_recipe {
            println!("Updating view with selected recipe: {}", recipe.title);
            // Ensure all widgets are visible
            widgets.recipe_detail_view.set_visible(true);
            widgets.recipe_detail.set_visible(true);
            widgets.recipe_title.set_visible(true);
            
            // Force visibility on key UI elements
            widgets.recipe_meta.set_visible(true);
            widgets.ingredients_title.set_visible(true);
            widgets.ingredients_list.set_visible(true);
            widgets.instructions_title.set_visible(true);
            widgets.instructions.set_visible(true);
            widgets.tags_box.set_visible(true);
            
            // Debug check - print what the recipe title shows
            println!("Recipe title widget current text: '{}'", widgets.recipe_title.text());
        } else {
            println!("Updating view with no recipe selected");
        }
        
        // Force GTK to redraw
        widgets.recipe_detail.queue_draw();
    }

    view! {
        gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 10,
            set_margin_all: 10,
            set_hexpand: true,
            set_vexpand: true,

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 10,
                set_margin_bottom: 10,
                
                gtk4::SearchEntry {
                    set_placeholder_text: Some("Search recipes..."),
                    set_hexpand: true,
                    connect_search_changed[sender] => move |entry| {
                        sender.input(RecipesInput::SearchChanged(entry.text().to_string()));
                    }
                },
                
                gtk4::Button {
                    set_label: "Add Recipe",
                    set_icon_name: "list-add",
                    connect_clicked[sender] => move |_| {
                        sender.input(RecipesInput::AddRecipe);
                    }
                }
            },

            gtk4::Paned {
                set_orientation: gtk4::Orientation::Horizontal,
                set_hexpand: true,
                set_vexpand: true,
                set_position: 300,

                #[name(recipe_list_box)]
                gtk4::ScrolledWindow {
                    set_hscrollbar_policy: gtk4::PolicyType::Never,
                    set_min_content_width: 200,
                    
                    #[name(recipe_list)]
                    gtk4::ListBox {
                        set_selection_mode: gtk4::SelectionMode::Single,
                        connect_row_activated[sender] => move |_list, row| {
                            let index = row.index() as usize;
                            sender.input(RecipesInput::SelectRecipe(index));
                        },
                        connect_row_selected[sender] => move |_list, row_opt| {
                            if let Some(row) = row_opt {
                                let index = row.index() as usize;
                                sender.input(RecipesInput::SelectRecipe(index));
                            }
                        },
                    }
                },

                gtk4::Box {
                    set_orientation: gtk4::Orientation::Vertical,
                    set_spacing: 10,
                    set_margin_start: 10,
                    set_hexpand: true,
                    set_vexpand: true,

                    #[name(recipe_detail_view)]
                    gtk4::ScrolledWindow {
                        set_hexpand: true,
                        set_vexpand: true,
                        
                        #[name(recipe_detail)]
                        gtk4::Box {
                            set_orientation: gtk4::Orientation::Vertical,
                            set_spacing: 10,
                            set_margin_all: 10,

                            #[name(recipe_title)]
                            gtk4::Label {
                                set_markup: "<b>Select a recipe</b>",
                                set_xalign: 0.0,
                                set_yalign: 0.0,
                                set_wrap: true,
                                add_css_class: "title-2",
                            },

                            #[name(recipe_meta)]
                            gtk4::Box {
                                set_orientation: gtk4::Orientation::Horizontal,
                                set_spacing: 10,
                                
                                #[name(prep_time)]
                                gtk4::Label {
                                    set_xalign: 0.0,
                                },
                                
                                #[name(total_time)]
                                gtk4::Label {
                                    set_xalign: 0.0,
                                },
                                
                                #[name(servings)]
                                gtk4::Label {
                                    set_xalign: 0.0,
                                }
                            },

                            #[name(ingredients_title)]
                            gtk4::Label {
                                set_markup: "<b>Ingredients</b>",
                                set_xalign: 0.0,
                                set_margin_top: 10,
                            },

                            #[name(ingredients_list)]
                            gtk4::Box {
                                set_orientation: gtk4::Orientation::Vertical,
                                set_spacing: 5,
                            },

                            #[name(instructions_title)]
                            gtk4::Label {
                                set_markup: "<b>Instructions</b>",
                                set_xalign: 0.0,
                                set_margin_top: 10,
                            },

                            #[name(instructions)]
                            gtk4::Label {
                                set_xalign: 0.0,
                                set_yalign: 0.0,
                                set_wrap: true,
                                set_selectable: true,
                                set_hexpand: true,
                            },

                            #[name(tags_box)]
                            gtk4::FlowBox {
                                set_selection_mode: gtk4::SelectionMode::None,
                                set_homogeneous: false,
                                set_row_spacing: 5,
                                set_column_spacing: 5,
                                set_max_children_per_line: 10,
                                set_margin_top: 10,
                            },

                            gtk4::Box {
                                set_orientation: gtk4::Orientation::Horizontal,
                                set_spacing: 10,
                                set_margin_top: 20,
                                set_halign: gtk4::Align::End,

                                gtk4::Button {
                                    set_label: "Edit",
                                    connect_clicked[sender] => move |_| {
                                        sender.input(RecipesInput::EditRecipe);
                                    }
                                },

                                gtk4::Button {
                                    set_label: "Delete",
                                    add_css_class: "destructive-action",
                                    connect_clicked[sender] => move |_| {
                                        sender.input(RecipesInput::DeleteRecipe);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl RecipesModel {
    fn populate_recipe_list(&self, widgets: &RecipesWidgets) {
        // First print all recipes we have to debug
        for (i, recipe) in self.recipes.iter().enumerate() {
            println!("Recipe #{}: {} [instructions length: {}]", 
                i+1, recipe.title, recipe.instructions.len());
        }

        // Clear existing rows
        while let Some(child) = widgets.recipe_list.first_child() {
            widgets.recipe_list.remove(&child);
        }

        // Add recipes to list
        for (index, recipe) in self.recipes.iter().enumerate() {
            let row = gtk4::ListBoxRow::new();
            row.set_selectable(true);
            let box_container = gtk4::Box::new(Orientation::Vertical, 5);
            box_container.set_margin_all(5);

            let title = gtk4::Label::new(Some(&recipe.title));
            title.set_xalign(0.0);
            title.add_css_class("heading");

            let meta = gtk4::Label::new(Some(&format!(
                "Prep: {} min · Total: {} min · {} servings",
                recipe.prep_time,
                recipe.total_time(),
                recipe.servings
            )));
            meta.set_xalign(0.0);
            meta.add_css_class("caption");

            box_container.append(&title);
            box_container.append(&meta);
            row.set_child(Some(&box_container));

            widgets.recipe_list.append(&row);
            
            // Select the first row by default or the previously selected recipe
            if let Some(selected) = &self.selected_recipe {
                if selected.title == recipe.title {
                    println!("Selecting row for previously selected recipe: {}", recipe.title);
                    widgets.recipe_list.select_row(Some(&row));
                }
            } else if index == 0 && !self.recipes.is_empty() {
                println!("Selecting first row by default: {}", recipe.title);
                widgets.recipe_list.select_row(Some(&row));
                // Force selection of this recipe
                self.selected_recipe = Some(recipe.clone());
            }
        }
    }

    fn filter_recipes(&mut self) {
        if let Some(dm) = &self.data_manager {
            // Start with all recipes
            let mut filtered = dm.get_all_recipes().to_vec();

            // Apply search filter if there's a search term
            if !self.search_term.is_empty() {
                let term = self.search_term.to_lowercase();
                filtered.retain(|recipe| {
                    recipe.title.to_lowercase().contains(&term)
                        || recipe.tags.iter().any(|tag| tag.to_lowercase().contains(&term))
                        || recipe.instructions.to_lowercase().contains(&term)
                });
            }

            self.recipes = filtered;
        }
    }
    
    // Helper function to update UI after recipes have been filtered
    fn update_recipe_list(&self, widgets: &RecipesWidgets) {
        self.populate_recipe_list(widgets);
    }

    fn update_recipe_detail(&self, widgets: &RecipesWidgets) {
        if let Some(recipe) = &self.selected_recipe {
            // Debug print to confirm recipe details are being updated
            println!("Updating recipe detail view for: {}", recipe.title);
            println!("Recipe details - prep_time: {}, downtime: {}, instructions length: {}", 
                recipe.prep_time, recipe.downtime, recipe.instructions.len());
            
            // Show the ingredients and instructions titles
            widgets.ingredients_title.show();
            widgets.instructions_title.show();
            
            widgets.recipe_title.set_markup(&format!("<b>{}</b>", glib::markup_escape_text(&recipe.title)));

            // Update meta info
            widgets.prep_time.set_text(&format!("Prep time: {} min", recipe.prep_time));
            widgets
                .total_time
                .set_text(&format!("Total time: {} min", recipe.total_time()));
            widgets
                .servings
                .set_text(&format!("{} servings", recipe.servings));

            // Clear and update ingredients
            while let Some(child) = widgets.ingredients_list.first_child() {
                widgets.ingredients_list.remove(&child);
            }

            for ingredient in &recipe.ingredients {
                let label_text = if let (Some(qty), Some(qty_type)) =
                    (ingredient.quantity.as_ref(), &ingredient.quantity_type)
                {
                    format!("• {} {} of {}", qty, qty_type, ingredient.ingredient)
                } else {
                    format!("• {}", ingredient.ingredient)
                };

                let label = Label::new(Some(&label_text));
                label.set_xalign(0.0);
                label.set_visible(true);
                widgets.ingredients_list.append(&label);
            }
            widgets.ingredients_list.show();

            // Update instructions with better formatting for newlines
            widgets.instructions.set_text(&recipe.instructions);
            widgets.instructions.set_visible(true);
            widgets.instructions.set_margin_start(10);
            widgets.instructions.set_margin_end(10);
            widgets.instructions.set_wrap(true);
            widgets.instructions.add_css_class("recipe-instructions");

            // Clear and update tags
            while let Some(child) = widgets.tags_box.first_child() {
                widgets.tags_box.remove(&child);
            }

            for tag in &recipe.tags {
                let button = Button::new();
                button.add_css_class("tag-button");
                button.set_label(tag);
                button.set_visible(true);
                widgets.tags_box.append(&button);
            }
            widgets.tags_box.show();
            
            // Make sure all recipe detail components are visible
            widgets.recipe_meta.show();
            widgets.recipe_detail.show();
            widgets.recipe_detail_view.show();
            widgets.recipe_title.show();

            println!("Recipe detail view updated successfully for: {}", recipe.title);
        } else {
            // No recipe selected
            widgets.recipe_title.set_markup("<b>Select a recipe</b>");
            widgets.prep_time.set_text("");
            widgets.total_time.set_text("");
            widgets.servings.set_text("");

            // Clear ingredients
            while let Some(child) = widgets.ingredients_list.first_child() {
                widgets.ingredients_list.remove(&child);
            }

            // Clear instructions
            widgets.instructions.set_text("");

            // Clear tags
            while let Some(child) = widgets.tags_box.first_child() {
                widgets.tags_box.remove(&child);
            }
            
            // Hide sections that should be empty
            widgets.ingredients_title.hide();
            widgets.instructions_title.hide();
            
            // Make sure recipe detail is visible even if empty
            widgets.recipe_detail.show();
            widgets.recipe_detail_view.show();
            widgets.recipe_title.show();
        }
        
        // Force GTK to redraw the recipe detail
        widgets.recipe_detail_view.queue_draw();
    }
}
