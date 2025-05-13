use gtk4::prelude::*;
use relm4::{
    gtk4, Component, ComponentParts, ComponentSender, SimpleComponent,
};
use std::path::PathBuf;

use crate::config::{AppConfig, Theme};

#[derive(Debug)]
pub struct SettingsModel {
    config: AppConfig,
    data_dir: PathBuf,
}

#[derive(Debug)]
pub enum SettingsInput {
    ThemeChanged(Theme),
    FontSizeChanged(f32),
    DataDirChanged(PathBuf),
}

#[derive(Debug)]
pub enum SettingsOutput {
    ThemeChanged(Theme),
    DataDirChanged(PathBuf),
}

pub struct SettingsComponent;

#[relm4::component(pub)]
impl SimpleComponent for SettingsComponent {
    type Init = (AppConfig, PathBuf);
    type Input = SettingsInput;
    type Output = SettingsOutput;
    type Widgets = SettingsWidgets;

    fn init(
        (config, data_dir): Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = SettingsModel {
            config,
            data_dir,
        };

        let widgets = view_output!();

        // Initialize the dropdowns
        widgets.theme_dropdown.set_active(match model.config.theme {
            Theme::Light => 0,
            Theme::Dark => 1,
            Theme::System => 2,
        });

        // Set font scale
        widgets.font_scale.set_value(model.config.font_scale as f64);

        // Set data directory
        widgets.data_dir_label.set_text(&model.data_dir.to_string_lossy());

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            SettingsInput::ThemeChanged(theme) => {
                self.config.theme = theme;
                sender.output(SettingsOutput::ThemeChanged(theme)).unwrap();
            }
            SettingsInput::FontSizeChanged(scale) => {
                self.config.font_scale = scale;
                // Would update font size in the application
            }
            SettingsInput::DataDirChanged(path) => {
                self.data_dir = path.clone();
                if let Some(path_str) = path.to_str() {
                    self.config.data_dir = Some(path_str.to_string());
                }
                sender.output(SettingsOutput::DataDirChanged(path)).unwrap();
            }
        }
    }

    view! {
        gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 20,
            set_margin_all: 20,
            set_hexpand: true,
            set_vexpand: true,

            gtk4::Label {
                set_markup: "<b>Settings</b>",
                set_xalign: 0.0,
                add_css_class: "title-2",
            },

            // Theme setting
            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 10,
                set_margin_top: 10,

                gtk4::Label {
                    set_text: "Theme:",
                    set_width_chars: 15,
                    set_xalign: 0.0,
                },

                #[name(theme_dropdown)]
                gtk4::DropDown::from_strings(&["Light", "Dark", "System"]) {
                    set_hexpand: true,
                    connect_selected_notify[sender] => move |dropdown| {
                        let theme = match dropdown.selected() {
                            0 => Theme::Light,
                            1 => Theme::Dark,
                            _ => Theme::System,
                        };
                        sender.input(SettingsInput::ThemeChanged(theme));
                    }
                }
            },

            // Font size setting
            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 10,
                set_margin_top: 10,

                gtk4::Label {
                    set_text: "Font scale:",
                    set_width_chars: 15,
                    set_xalign: 0.0,
                },

                #[name(font_scale)]
                gtk4::SpinButton::with_range(0.5, 2.0, 0.1) {
                    set_hexpand: true,
                    connect_value_changed[sender] => move |spin_button| {
                        sender.input(SettingsInput::FontSizeChanged(spin_button.value() as f32));
                    }
                }
            },

            // Data directory setting
            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 10,
                set_margin_top: 10,

                gtk4::Label {
                    set_text: "Data directory:",
                    set_width_chars: 15,
                    set_xalign: 0.0,
                },

                gtk4::Box {
                    set_orientation: gtk4::Orientation::Vertical,
                    set_hexpand: true,

                    #[name(data_dir_label)]
                    gtk4::Label {
                        set_xalign: 0.0,
                        set_ellipsize: gtk4::pango::EllipsizeMode::Middle,
                        set_hexpand: true,
                    },

                    gtk4::Button {
                        set_label: "Change data directory",
                        set_margin_top: 5,
                        set_halign: gtk4::Align::Start,
                        connect_clicked[sender, root] => move |_| {
                            // Create a file chooser dialog
                            let dialog = gtk4::FileChooserDialog::new(
                                Some("Select Data Directory"),
                                Some(root.upcast_ref::<gtk4::Window>()),
                                gtk4::FileChooserAction::SelectFolder,
                                &[("Cancel", gtk4::ResponseType::Cancel), ("Open", gtk4::ResponseType::Accept)]
                            );
                            
                            // Show the dialog
                            dialog.connect_response(move |dialog, response| {
                                if response == gtk4::ResponseType::Accept {
                                    if let Some(file) = dialog.file() {
                                        if let Some(path) = file.path() {
                                            sender.input(SettingsInput::DataDirChanged(path));
                                        }
                                    }
                                }
                                dialog.destroy();
                            });
                            
                            dialog.show();
                        }
                    }
                }
            },

            // Language setting (placeholder, would be implemented later)
            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 10,
                set_margin_top: 10,

                gtk4::Label {
                    set_text: "Language:",
                    set_width_chars: 15,
                    set_xalign: 0.0,
                },

                gtk4::DropDown::from_strings(&["English"]) {
                    set_hexpand: true,
                    set_sensitive: false, // Disabled for now
                }
            },

            gtk4::Box {
                set_orientation: gtk4::Orientation::Vertical,
                set_hexpand: true,
                set_vexpand: true,
            },

            // Save button - not needed with automatic saving
            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 10,
                set_halign: gtk4::Align::End,

                gtk4::Button {
                    set_label: "Reset to defaults",
                    add_css_class: "destructive-action"
                    // Not implemented yet
                },

                gtk4::Button {
                    set_label: "Save settings",
                    add_css_class: "suggested-action",
                    // Not implemented yet - settings are saved automatically
                }
            }
        }
    }
}
