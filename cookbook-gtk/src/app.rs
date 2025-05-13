use cookbook_engine::DataManager;
use gtk4::{glib, CssProvider};
use gtk4::prelude::*;
use relm4::{
    adw, gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller,
    SimpleComponent,
};
use std::path::PathBuf;

use crate::components::recipes::RecipesComponent;
use crate::components::pantry::PantryComponent;
use crate::components::kb::KnowledgeBaseComponent;
use crate::components::settings::SettingsComponent;
use crate::components::debug_hook::install_debug_hook;
use crate::config::{AppConfig, Theme};
use crate::ui::about::show_about_dialog;
use crate::ui::help::show_help_dialog;

#[derive(Debug)]
pub struct AppModel {
    pub config: AppConfig,
    pub data_manager: Option<DataManager>,
    pub data_dir: PathBuf,
    pub current_view: AppView,
    pub recipes_component: Controller<RecipesComponent>,
    pub pantry_component: Controller<PantryComponent>,
    pub kb_component: Controller<KnowledgeBaseComponent>,
    pub settings_component: Controller<SettingsComponent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppView {
    Recipes,
    Pantry,
    KnowledgeBase,
    Settings,
}

#[derive(Debug)]
pub enum AppInput {
    SwitchView(AppView),
    ShowAbout,
    ShowHelp,
    ThemeChanged(Theme),
    DataDirChanged(PathBuf),
    Quit,
}

#[derive(Debug)]
pub enum AppOutput {
    Quit,
}

pub struct CookbookApp;

#[relm4::component(pub)]
impl SimpleComponent for CookbookApp {
    type Init = PathBuf;
    type Input = AppInput;
    type Output = AppOutput;
    type Widgets = AppWidgets;

    fn init(
        data_dir: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // Load config
        let config = AppConfig::load();

        // Setup CSS
        let provider = CssProvider::new();
        provider.load_from_data(include_str!("../assets/style.css"));
        StyleContext::add_provider_for_display(
            &gtk4::gdk::Display::default().expect("Could not get default display"),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        
        // Install debug hook
        install_debug_hook();

        // Initialize data manager
        let data_manager = match DataManager::new(&data_dir) {
            Ok(manager) => Some(manager),
            Err(e) => {
                eprintln!("Failed to initialize data manager: {}", e);
                None
            }
        };

        // Initialize components
        let recipes_component = RecipesComponent::builder()
            .launch(data_manager.as_ref())
            .forward(sender.input_sender(), |_| AppInput::Quit);

        let pantry_component = PantryComponent::builder()
            .launch(data_manager.as_ref())
            .forward(sender.input_sender(), |_| AppInput::Quit);

        let kb_component = KnowledgeBaseComponent::builder()
            .launch(data_manager.as_ref())
            .forward(sender.input_sender(), |_| AppInput::Quit);

        let settings_component = SettingsComponent::builder()
            .launch((config.clone(), data_dir.clone()))
            .forward(sender.input_sender(), |msg| match msg {
                crate::components::settings::SettingsOutput::ThemeChanged(theme) => {
                    AppInput::ThemeChanged(theme)
                }
                crate::components::settings::SettingsOutput::DataDirChanged(path) => {
                    AppInput::DataDirChanged(path)
                }
            });

        let model = AppModel {
            config,
            data_manager,
            data_dir,
            current_view: AppView::Recipes,
            recipes_component,
            pantry_component,
            kb_component,
            settings_component,
        };

        let widgets = view_output!();

        // Set initial view
        model.show_current_view(&widgets);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, widgets: &mut Self::Widgets) {
        match msg {
            AppInput::SwitchView(view) => {
                self.current_view = view;
                self.show_current_view(widgets);
            }
            AppInput::ShowAbout => {
                show_about_dialog();
            }
            AppInput::ShowHelp => {
                show_help_dialog();
            }
            AppInput::ThemeChanged(theme) => {
                self.config.theme = theme;
                self.config.save();
                adw::StyleManager::default().set_color_scheme(match theme {
                    Theme::Light => adw::ColorScheme::ForceLight,
                    Theme::Dark => adw::ColorScheme::ForceDark,
                    Theme::System => adw::ColorScheme::Default,
                });
            }
            AppInput::DataDirChanged(path) => {
                self.data_dir = path.clone();
                self.data_manager = match DataManager::new(&path) {
                    Ok(manager) => Some(manager),
                    Err(e) => {
                        eprintln!("Failed to initialize data manager with new path: {}", e);
                        None
                    }
                };

                // Update components with new data manager
                if let Some(ref manager) = self.data_manager {
                    self.recipes_component.emit(crate::components::recipes::RecipesInput::UpdateDataManager(manager));
                    self.pantry_component.emit(crate::components::pantry::PantryInput::UpdateDataManager(manager));
                    self.kb_component.emit(crate::components::kb::KbInput::UpdateDataManager(manager));
                }
            }
            AppInput::Quit => {
                sender.output(AppOutput::Quit).unwrap();
            }
        }
    }

    fn shutdown(&mut self, _sender: ComponentSender<Self>) {
        // Save any unsaved data if needed
        self.config.save();
    }

    view! {
        gtk4::ApplicationWindow {
            set_title: Some("Cookbook"),
            set_default_width: 1024,
            set_default_height: 768,

            gtk4::Box {
                set_orientation: gtk4::Orientation::Vertical,

                #[name(header_bar)]
                gtk4::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget = &gtk4::Label {
                        set_markup: "<b>Cookbook</b>",
                    },
                },

                #[name(main_box)]
                gtk4::Box {
                    set_orientation: gtk4::Orientation::Horizontal,
                    set_spacing: 10,
                    set_margin_all: 10,

                    #[name(sidebar)]
                    gtk4::Box {
                        set_orientation: gtk4::Orientation::Vertical,
                        set_spacing: 5,
                        set_margin_all: 5,
                        set_width_request: 200,

                        gtk4::Label {
                            set_markup: "<b>Navigation</b>",
                            set_xalign: 0.0,
                            set_margin_bottom: 10,
                        },

                        #[name(recipes_button)]
                        gtk4::Button {
                            set_label: "Recipes",
                            connect_clicked[sender] => move |_| {
                                sender.input(AppInput::SwitchView(AppView::Recipes));
                            },
                        },

                        #[name(pantry_button)]
                        gtk4::Button {
                            set_label: "Pantry",
                            connect_clicked[sender] => move |_| {
                                sender.input(AppInput::SwitchView(AppView::Pantry));
                            },
                        },

                        #[name(kb_button)]
                        gtk4::Button {
                            set_label: "Knowledge Base",
                            connect_clicked[sender] => move |_| {
                                sender.input(AppInput::SwitchView(AppView::KnowledgeBase));
                            },
                        },

                        gtk4::Separator {
                            set_margin_top: 10,
                            set_margin_bottom: 10,
                        },

                        #[name(settings_button)]
                        gtk4::Button {
                            set_label: "Settings",
                            connect_clicked[sender] => move |_| {
                                sender.input(AppInput::SwitchView(AppView::Settings));
                            },
                        },

                        gtk4::Button {
                            set_label: "Help",
                            connect_clicked[sender] => move |_| {
                                sender.input(AppInput::ShowHelp);
                            },
                        },

                        gtk4::Button {
                            set_label: "About",
                            connect_clicked[sender] => move |_| {
                                sender.input(AppInput::ShowAbout);
                            },
                        },
                    },

                    gtk4::Separator {
                        set_orientation: gtk4::Orientation::Vertical,
                    },

                    #[name(content_area)]
                    gtk4::Box {
                        set_hexpand: true,
                        set_vexpand: true,
                    },
                }
            }
        }
    }
}

impl AppModel {
    fn show_current_view(&self, widgets: &AppWidgets) {
        // Remove all children from content area
        while let Some(child) = widgets.content_area.first_child() {
            widgets.content_area.remove(&child);
        }

        // Set active sidebar button
        for (view, button) in [
            (AppView::Recipes, &widgets.recipes_button),
            (AppView::Pantry, &widgets.pantry_button),
            (AppView::KnowledgeBase, &widgets.kb_button),
            (AppView::Settings, &widgets.settings_button),
        ] {
            button.set_css_classes(&if self.current_view == view {
                vec!["suggested-action"]
            } else {
                vec![]
            });
        }

        // Add corresponding component's root widget to content area
        match self.current_view {
            AppView::Recipes => {
                widgets
                    .content_area
                    .append(self.recipes_component.widget());
                
                // Force a refresh of the recipe view to ensure details are displayed
                if let Some(ref manager) = self.data_manager {
                    println!("Refreshing recipe view");
                    self.recipes_component.emit(crate::components::recipes::RecipesInput::RefreshView);
                }
            }
            AppView::Pantry => {
                widgets
                    .content_area
                    .append(self.pantry_component.widget());
            }
            AppView::KnowledgeBase => {
                widgets
                    .content_area
                    .append(self.kb_component.widget());
            }
            AppView::Settings => {
                widgets
                    .content_area
                    .append(self.settings_component.widget());
            }
        }
    }
}
