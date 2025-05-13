use cookbook_engine::{DataManager, Pantry, PantryItem};
use gtk4::{prelude::*, Button, Label, ListBox, Orientation, ScrolledWindow};
use relm4::{
    gtk4, Component, ComponentParts, ComponentSender, SimpleComponent,
};
use std::rc::Rc;
use chrono::NaiveDate;

#[derive(Debug)]
pub struct PantryModel {
    pantry: Option<Pantry>,
    data_manager: Option<Rc<DataManager>>,
    selected_item: Option<PantryItem>,
    search_term: String,
}

#[derive(Debug)]
pub enum PantryInput {
    SelectItem(usize),
    UpdateDataManager(&'static DataManager),
    SearchChanged(String),
    AddItem,
    EditItem,
    DeleteItem,
}

#[derive(Debug)]
pub enum PantryOutput {
    Quit,
}

pub struct PantryComponent;

#[relm4::component(pub)]
impl SimpleComponent for PantryComponent {
    type Init = Option<&'static DataManager>;
    type Input = PantryInput;
    type Output = PantryOutput;
    type Widgets = PantryWidgets;

    fn init(
        data_manager: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let pantry = data_manager.and_then(|dm| dm.get_pantry().cloned());

        let model = PantryModel {
            pantry,
            data_manager: data_manager.map(Rc::from),
            selected_item: None,
            search_term: String::new(),
        };

        let widgets = view_output!();

        // Populate pantry list
        model.populate_pantry_list(&widgets);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            PantryInput::SelectItem(index) => {
                if let Some(pantry) = &self.pantry {
                    if index < pantry.items.len() {
                        self.selected_item = Some(pantry.items[index].clone());
                        // Update detail view
                    }
                }
            }
            PantryInput::UpdateDataManager(data_manager) => {
                self.data_manager = Some(Rc::from(data_manager));
                self.pantry = data_manager.get_pantry().cloned();
                self.selected_item = None;
            }
            PantryInput::SearchChanged(term) => {
                self.search_term = term;
                // Apply filter - not implemented yet
            }
            PantryInput::AddItem => {
                // Not implemented yet - would open a dialog to add new pantry item
            }
            PantryInput::EditItem => {
                // Not implemented yet - would open a dialog to edit the selected item
            }
            PantryInput::DeleteItem => {
                // Not implemented yet - would delete the selected item
            }
        }
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
                    set_placeholder_text: Some("Search pantry items..."),
                    set_hexpand: true,
                    connect_search_changed[sender] => move |entry| {
                        sender.input(PantryInput::SearchChanged(entry.text().to_string()));
                    }
                },
                
                gtk4::Button {
                    set_label: "Add Item",
                    set_icon_name: "list-add",
                    connect_clicked[sender] => move |_| {
                        sender.input(PantryInput::AddItem);
                    }
                }
            },

            #[name(no_pantry_label)]
            gtk4::Label {
                set_label: "No pantry data available.",
                set_visible: self.pantry.is_none(),
                set_margin_top: 50,
                add_css_class: "dim-label",
            },

            gtk4::Paned {
                set_orientation: gtk4::Orientation::Horizontal,
                set_hexpand: true,
                set_vexpand: true,
                set_position: 300,
                set_visible: self.pantry.is_some(),

                #[name(item_list_box)]
                gtk4::ScrolledWindow {
                    set_hscrollbar_policy: gtk4::PolicyType::Never,
                    set_min_content_width: 200,
                    
                    #[name(item_list)]
                    gtk4::ListBox {
                        set_selection_mode: gtk4::SelectionMode::Single,
                        connect_row_activated[sender] => move |_, row| {
                            sender.input(PantryInput::SelectItem(row.index() as usize));
                        },
                    }
                },

                gtk4::Box {
                    set_orientation: gtk4::Orientation::Vertical,
                    set_spacing: 10,
                    set_margin_start: 10,
                    set_hexpand: true,
                    set_vexpand: true,

                    #[name(detail_view)]
                    gtk4::ScrolledWindow {
                        set_hexpand: true,
                        set_vexpand: true,
                        
                        #[name(item_detail)]
                        gtk4::Box {
                            set_orientation: gtk4::Orientation::Vertical,
                            set_spacing: 15,
                            set_margin_all: 10,

                            #[name(item_name)]
                            gtk4::Label {
                                set_markup: "<b>Select an item</b>",
                                set_xalign: 0.0,
                                set_yalign: 0.0,
                                set_wrap: true,
                                add_css_class: "title-2",
                            },

                            #[name(item_quantity)]
                            gtk4::Label {
                                set_xalign: 0.0,
                                set_yalign: 0.0,
                            },

                            #[name(last_updated)]
                            gtk4::Label {
                                set_xalign: 0.0,
                                set_yalign: 0.0,
                                add_css_class: "dim-label",
                            },

                            #[name(ingredient_info)]
                            gtk4::Box {
                                set_orientation: gtk4::Orientation::Vertical,
                                set_spacing: 10,
                                set_margin_top: 20,

                                #[name(category_label)]
                                gtk4::Label {
                                    set_xalign: 0.0,
                                    set_yalign: 0.0,
                                },

                                #[name(tags_box)]
                                gtk4::FlowBox {
                                    set_selection_mode: gtk4::SelectionMode::None,
                                    set_homogeneous: false,
                                    set_row_spacing: 5,
                                    set_column_spacing: 5,
                                    set_max_children_per_line: 10,
                                },
                            },

                            gtk4::Box {
                                set_orientation: gtk4::Orientation::Horizontal,
                                set_spacing: 10,
                                set_margin_top: 20,
                                set_halign: gtk4::Align::End,

                                gtk4::Button {
                                    set_label: "Edit",
                                    connect_clicked[sender] => move |_| {
                                        sender.input(PantryInput::EditItem);
                                    }
                                },

                                gtk4::Button {
                                    set_label: "Delete",
                                    add_css_class: "destructive-action",
                                    connect_clicked[sender] => move |_| {
                                        sender.input(PantryInput::DeleteItem);
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

impl PantryModel {
    fn populate_pantry_list(&self, widgets: &PantryWidgets) {
        // Clear existing rows
        while let Some(child) = widgets.item_list.first_child() {
            widgets.item_list.remove(&child);
        }

        // If we have pantry data, populate the list
        if let Some(pantry) = &self.pantry {
            for item in &pantry.items {
                let row = gtk4::ListBoxRow::new();
                let box_container = gtk4::Box::new(Orientation::Vertical, 5);
                box_container.set_margin_all(5);

                let title = gtk4::Label::new(Some(&item.ingredient));
                title.set_xalign(0.0);
                title.add_css_class("heading");

                // Quantity details
                let quantity_text = match (&item.quantity, &item.quantity_type) {
                    (Some(qty), Some(qty_type)) => format!("{} {}", qty, qty_type),
                    (Some(qty), None) => format!("{}", qty),
                    _ => "Unknown quantity".to_string(),
                };

                let quantity = gtk4::Label::new(Some(&quantity_text));
                quantity.set_xalign(0.0);
                quantity.add_css_class("caption");

                box_container.append(&title);
                box_container.append(&quantity);
                row.set_child(Some(&box_container));

                widgets.item_list.append(&row);
            }
        }
    }

    fn update_item_detail(&self, widgets: &PantryWidgets) {
        if let Some(item) = &self.selected_item {
            // Update name
            widgets.item_name.set_markup(&format!("<b>{}</b>", item.ingredient));

            // Update quantity
            let quantity_text = match (&item.quantity, &item.quantity_type) {
                (Some(qty), Some(qty_type)) => format!("Quantity: {} {}", qty, qty_type),
                (Some(qty), None) => format!("Quantity: {}", qty),
                _ => "Quantity: Unknown".to_string(),
            };
            widgets.item_quantity.set_text(&quantity_text);

            // Format date
            let date_text = match NaiveDate::parse_from_str(&item.last_updated, "%Y-%m-%d") {
                Ok(date) => format!("Last updated: {}", date.format("%B %d, %Y")),
                Err(_) => format!("Last updated: {}", item.last_updated),
            };
            widgets.last_updated.set_text(&date_text);

            // Update ingredient info based on data manager
            if let Some(dm) = &self.data_manager {
                if let Some(ingredient) = dm.get_ingredient(&item.ingredient) {
                    // Set category
                    widgets.category_label.set_text(&format!("Category: {}", ingredient.category));

                    // Update tags
                    while let Some(child) = widgets.tags_box.first_child() {
                        widgets.tags_box.remove(&child);
                    }

                    for tag in &ingredient.tags {
                        let button = Button::new();
                        button.add_css_class("tag-button");
                        button.set_label(tag);
                        widgets.tags_box.append(&button);
                    }
                } else {
                    widgets.category_label.set_text("Category: Unknown");
                    
                    while let Some(child) = widgets.tags_box.first_child() {
                        widgets.tags_box.remove(&child);
                    }
                }
            } else {
                widgets.category_label.set_text("");
                
                while let Some(child) = widgets.tags_box.first_child() {
                    widgets.tags_box.remove(&child);
                }
            }
        } else {
            // No item selected
            widgets.item_name.set_markup("<b>Select an item</b>");
            widgets.item_quantity.set_text("");
            widgets.last_updated.set_text("");
            widgets.category_label.set_text("");
            
            while let Some(child) = widgets.tags_box.first_child() {
                widgets.tags_box.remove(&child);
            }
        }
    }
}
