use cookbook_engine::{DataManager, KnowledgeBaseEntry};
use gtk4::{prelude::*, gdk_pixbuf::Pixbuf, Image, Label, ListBox, Orientation, ScrolledWindow};
use relm4::{
    gtk4, Component, ComponentParts, ComponentSender, SimpleComponent,
};
use std::rc::Rc;
use std::path::PathBuf;
use markdown::to_html;

#[derive(Debug)]
pub struct KbModel {
    kb_entries: Vec<KnowledgeBaseEntry>,
    data_manager: Option<Rc<DataManager>>,
    selected_entry: Option<KnowledgeBaseEntry>,
    search_term: String,
    data_dir: Option<PathBuf>,
}

#[derive(Debug)]
pub enum KbInput {
    SelectEntry(usize),
    UpdateDataManager(&'static DataManager),
    SearchChanged(String),
}

#[derive(Debug)]
pub enum KbOutput {
    Quit,
}

pub struct KnowledgeBaseComponent;

#[relm4::component(pub)]
impl SimpleComponent for KnowledgeBaseComponent {
    type Init = Option<&'static DataManager>;
    type Input = KbInput;
    type Output = KbOutput;
    type Widgets = KbWidgets;

    fn init(
        data_manager: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let kb_entries = if let Some(dm) = data_manager {
            dm.get_all_kb_entries()
        } else {
            Vec::new()
        };

        let model = KbModel {
            kb_entries: kb_entries.into_iter().cloned().collect(),
            data_manager: data_manager.map(Rc::from),
            selected_entry: None,
            search_term: String::new(),
            data_dir: None, // Would need to be set from the data manager
        };

        let widgets = view_output!();

        // Populate KB entry list
        model.populate_kb_list(&widgets);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            KbInput::SelectEntry(index) => {
                if index < self.kb_entries.len() {
                    self.selected_entry = Some(self.kb_entries[index].clone());
                    // Would update detail view
                }
            }
            KbInput::UpdateDataManager(data_manager) => {
                self.data_manager = Some(Rc::from(data_manager));
                self.kb_entries = data_manager.get_all_kb_entries().into_iter().cloned().collect();
                self.selected_entry = None;
            }
            KbInput::SearchChanged(term) => {
                self.search_term = term;
                // Filter KB entries - not implemented yet
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
                    set_placeholder_text: Some("Search knowledge base..."),
                    set_hexpand: true,
                    connect_search_changed[sender] => move |entry| {
                        sender.input(KbInput::SearchChanged(entry.text().to_string()));
                    }
                },
            },

            #[name(no_kb_label)]
            gtk4::Label {
                set_label: "No knowledge base entries available.",
                set_visible: self.kb_entries.is_empty(),
                set_margin_top: 50,
                add_css_class: "dim-label",
            },

            gtk4::Paned {
                set_orientation: gtk4::Orientation::Horizontal,
                set_hexpand: true,
                set_vexpand: true,
                set_position: 300,
                set_visible: !self.kb_entries.is_empty(),

                #[name(kb_list_box)]
                gtk4::ScrolledWindow {
                    set_hscrollbar_policy: gtk4::PolicyType::Never,
                    set_min_content_width: 200,
                    
                    #[name(kb_list)]
                    gtk4::ListBox {
                        set_selection_mode: gtk4::SelectionMode::Single,
                        connect_row_activated[sender] => move |_, row| {
                            sender.input(KbInput::SelectEntry(row.index() as usize));
                        },
                    }
                },

                gtk4::Box {
                    set_orientation: gtk4::Orientation::Vertical,
                    set_spacing: 10,
                    set_margin_start: 10,
                    set_hexpand: true,
                    set_vexpand: true,

                    #[name(kb_detail_view)]
                    gtk4::ScrolledWindow {
                        set_hexpand: true,
                        set_vexpand: true,
                        
                        #[name(kb_detail)]
                        gtk4::Box {
                            set_orientation: gtk4::Orientation::Vertical,
                            set_spacing: 15,
                            set_margin_all: 10,

                            #[name(kb_title)]
                            gtk4::Label {
                                set_markup: "<b>Select an entry</b>",
                                set_xalign: 0.0,
                                set_yalign: 0.0,
                                set_wrap: true,
                                add_css_class: "title-2",
                            },

                            #[name(kb_image_container)]
                            gtk4::Box {
                                set_orientation: gtk4::Orientation::Vertical,
                                set_halign: gtk4::Align::Center,
                                set_margin_top: 10,
                                set_margin_bottom: 10,

                                #[name(kb_image)]
                                gtk4::Image {
                                    set_size_request: (300, 200),
                                }
                            },

                            #[name(kb_content)]
                            gtk4::Label {
                                set_xalign: 0.0,
                                set_yalign: 0.0,
                                set_wrap: true,
                                set_use_markup: true,
                                set_selectable: true,
                                set_hexpand: true,
                            }
                        }
                    }
                }
            }
        }
    }
}

impl KbModel {
    fn populate_kb_list(&self, widgets: &KbWidgets) {
        // Clear existing rows
        while let Some(child) = widgets.kb_list.first_child() {
            widgets.kb_list.remove(&child);
        }

        // Add KB entries to list
        for entry in &self.kb_entries {
            let row = gtk4::ListBoxRow::new();
            let box_container = gtk4::Box::new(Orientation::Vertical, 5);
            box_container.set_margin_all(5);

            let title = gtk4::Label::new(Some(&entry.title));
            title.set_xalign(0.0);
            title.add_css_class("heading");

            let slug = gtk4::Label::new(Some(&format!("Slug: {}", entry.slug)));
            slug.set_xalign(0.0);
            slug.add_css_class("caption");

            box_container.append(&title);
            box_container.append(&slug);
            row.set_child(Some(&box_container));

            widgets.kb_list.append(&row);
        }
    }

    fn update_kb_detail(&self, widgets: &KbWidgets) {
        if let Some(entry) = &self.selected_entry {
            // Update title
            widgets.kb_title.set_markup(&format!("<b>{}</b>", entry.title));

            // Update image if available
            if let Some(image_path) = &entry.image {
                if let Some(data_dir) = &self.data_dir {
                    let full_path = data_dir.join("kb").join(image_path);
                    
                    match Pixbuf::from_file(full_path) {
                        Ok(pixbuf) => {
                            // Resize the image if needed
                            let scaled = pixbuf.scale_simple(300, 200, gtk4::gdk_pixbuf::InterpType::Bilinear);
                            widgets.kb_image.set_from_pixbuf(scaled.as_ref());
                            widgets.kb_image_container.set_visible(true);
                        }
                        Err(_) => {
                            widgets.kb_image_container.set_visible(false);
                        }
                    }
                } else {
                    widgets.kb_image_container.set_visible(false);
                }
            } else {
                widgets.kb_image_container.set_visible(false);
            }

            // Convert markdown to HTML and set content
            let html_content = to_html(&entry.content);
            // Note: GTK Label has limited HTML support; in a real app you might want to use a WebView
            widgets.kb_content.set_markup(&html_content);
        } else {
            // No entry selected
            widgets.kb_title.set_markup("<b>Select an entry</b>");
            widgets.kb_image_container.set_visible(false);
            widgets.kb_content.set_text("");
        }
    }
}
