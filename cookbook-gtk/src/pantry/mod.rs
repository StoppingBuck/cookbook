pub mod detail;
pub mod format;
pub mod list;

// Re-export everything so existing callers work unchanged
pub use detail::{build_ingredient_detail_view, show_edit_ingredient_dialog};
#[allow(unused_imports)]
pub use format::format_quantity;
pub use list::rebuild_pantry_list;

use crate::i18n::tr;
use crate::types::AppModel;
use crate::types::AppMsg;
use crate::ui_constants::*;
use gtk::prelude::*;
use relm4::gtk;
use relm4::ComponentSender;
use relm4::RelmWidgetExt;
use std::cell::RefCell;
use std::rc::Rc;

/// Builds the Pantry tab UI and returns the main container, list container, details box, in-stock switch, and title label.
pub fn build_pantry_tab(
    model: &AppModel,
    sender: Option<ComponentSender<AppModel>>,
) -> (
    gtk::Box,                       // pantry_container
    gtk::ListBox,                   // pantry_list_container
    gtk::Box,                       // pantry_details_box
    gtk::Switch,                    // stock_filter_switch
    gtk::Label,                     // pantry_title
    Option<Box<dyn Fn(&AppModel)>>, // refresh_categories closure
) {
    // Main container for the Pantry tab
    let pantry_container = gtk::Box::new(gtk::Orientation::Vertical, SECTION_SPACING);

    // Title
    let pantry_title = gtk::Label::new(Some(&tr("Pantry")));
    pantry_title.set_markup(&format!(
        "<span size='x-large' weight='bold'>{}</span>",
        tr("Pantry")
    ));
    pantry_title.set_halign(gtk::Align::Start);
    pantry_title.set_margin_all(DEFAULT_MARGIN);

    // Filters frame
    let filters_frame = gtk::Frame::new(None);
    filters_frame.set_margin_bottom(DEFAULT_MARGIN);

    let filters_container = gtk::Box::new(gtk::Orientation::Vertical, TAG_SPACING);
    filters_container.set_margin_all(DEFAULT_MARGIN);

    // Category filters (popover multi-select)
    let category_filters_label = gtk::Label::new(Some(&tr("Categories:")));
    category_filters_label.set_halign(gtk::Align::Start);
    category_filters_label.set_margin_bottom(LIST_ROW_MARGIN);

    // Button to open popover (static, not recreated)
    let filter_button = gtk::Button::with_label(&tr("Filter Categories"));
    filter_button.set_halign(gtk::Align::Start);
    filter_button.set_tooltip_text(Some(&tr("Filter by one or more categories")));

    // Popover for category filters (static)
    let popover = gtk::Popover::new();
    popover.set_parent(&filter_button);
    // Ensure popover is closed when the parent button is finalized
    {
        let popover_weak = popover.downgrade();
        filter_button.connect_destroy(move |_| {
            if let Some(popover) = popover_weak.upgrade() {
                popover.popdown();
                popover.unparent();
            }
        });
    }
    let popover_box = gtk::Box::new(gtk::Orientation::Vertical, TAG_SPACING);
    popover_box.set_margin_all(DEFAULT_MARGIN);
    popover_box.set_vexpand(true);
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_min_content_height(180);
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    let listbox = gtk::Box::new(gtk::Orientation::Vertical, TAG_SPACING);
    scroll.set_child(Some(&listbox));
    popover_box.append(&scroll);
    popover.set_child(Some(&popover_box));

    // Function to refresh the listbox contents
    let refresh_categories = {
        let listbox = listbox.clone();
        let sender = sender.clone();
        move |categories: Vec<String>, selected_categories: Vec<String>| {
            log::debug!(
                "[refresh_categories] Called with categories: {:?}, selected: {:?}",
                categories, selected_categories
            );
            while let Some(child) = listbox.first_child() {
                listbox.remove(&child);
            }
            for category in &categories {
                let check = gtk::CheckButton::with_label(category);
                check.set_active(selected_categories.contains(category));
                let sender_clone = sender.clone();
                let category_clone = category.clone();
                check.connect_toggled(move |btn| {
                    if let Some(sender) = &sender_clone {
                        sender.input(AppMsg::TogglePantryCategory(
                            category_clone.clone(),
                            btn.is_active(),
                        ));
                    }
                });
                listbox.append(&check);
            }
        }
    };
    // Initial population
    let categories = if let Some(ref dm) = model.data_manager {
        log::debug!(
            "[build_pantry_tab] DataManager ptr: {:p}",
            Rc::as_ptr(dm)
        );
        let cats = dm.as_ref().get_unique_categories();
        log::debug!("[build_pantry_tab] Initial categories: {:?}", cats);
        cats
    } else {
        log::debug!("[build_pantry_tab] No DataManager available");
        Vec::new()
    };
    let selected_categories = model.selected_pantry_categories.clone();
    log::debug!(
        "[build_pantry_tab] Selected categories: {:?}",
        selected_categories
    );
    refresh_categories(categories.clone(), selected_categories.clone());

    // Static wrapper closure for AppWidgets
    let listbox = listbox.clone();
    let sender = sender.clone();
    let sender_for_refresh_categories = sender.clone();
    let refresh_categories_boxed = Box::new(move |model: &AppModel| {
        if let Some(ref dm) = model.data_manager {
            log::debug!(
                "[refresh_categories_boxed] DataManager ptr: {:p}",
                Rc::as_ptr(dm)
            );
            let categories = dm.as_ref().get_unique_categories();
            log::debug!(
                "[refresh_categories_boxed] Refreshed categories: {:?}",
                categories
            );
            let selected_categories = model.selected_pantry_categories.clone();
            log::debug!(
                "[refresh_categories_boxed] Selected categories: {:?}",
                selected_categories
            );
            while let Some(child) = listbox.first_child() {
                listbox.remove(&child);
            }
            for category in &categories {
                let check = gtk::CheckButton::with_label(category);
                check.set_active(selected_categories.contains(category));
                let sender_clone = sender_for_refresh_categories.clone();
                let category_clone = category.clone();
                check.connect_toggled(move |btn| {
                    if let Some(sender) = &sender_clone {
                        sender.input(AppMsg::TogglePantryCategory(
                            category_clone.clone(),
                            btn.is_active(),
                        ));
                    }
                });
                listbox.append(&check);
            }
        } else {
            log::debug!("[refresh_categories_boxed] No DataManager available");
            while let Some(child) = listbox.first_child() {
                listbox.remove(&child);
            }
        }
    });
    // Open popover on button click
    filter_button.connect_clicked(move |_| {
        popover.popup();
    });

    // Add refresh button next to filter_button
    let refresh_button = gtk::Button::new();
    let refresh_icon = gtk::Image::from_icon_name("view-refresh-symbolic");
    refresh_icon.set_pixel_size(16);
    refresh_button.set_child(Some(&refresh_icon));
    refresh_button.set_tooltip_text(Some(&tr("Refresh category list")));
    let sender_for_refresh = sender.clone();
    refresh_button.connect_clicked(move |_| {
        if let Some(sender) = &sender_for_refresh {
            sender.input(AppMsg::RefreshCategoryPopover);
        }
    });

    // Update button label to show number of selected categories
    let selected_count = model.selected_pantry_categories.len();
    if selected_count > 0 {
        filter_button.set_label(&format!("{} ({})", tr("Filter Categories"), selected_count));
    }

    // In-stock only filter
    let stock_filter_box = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
    let stock_filter_label = gtk::Label::new(Some(&tr("Show in-stock items only:")));
    stock_filter_label.set_halign(gtk::Align::Start);

    let stock_filter_switch = gtk::Switch::new();
    stock_filter_switch.set_state(model.show_in_stock_only);
    let sender_clone = sender.clone();
    stock_filter_switch.connect_state_notify(move |switch| {
        if let Some(sender) = &sender_clone {
            sender.input(AppMsg::ToggleInStockFilter(switch.state()));
        }
    });

    stock_filter_box.append(&stock_filter_label);
    stock_filter_box.append(&stock_filter_switch);

    // Add filter_button and refresh_button to a horizontal box
    let filter_buttons_box = gtk::Box::new(gtk::Orientation::Horizontal, TAG_SPACING);
    filter_buttons_box.append(&filter_button);
    filter_buttons_box.append(&refresh_button);
    filters_container.append(&category_filters_label);
    filters_container.append(&filter_buttons_box);
    filters_container.append(&stock_filter_box);

    filters_frame.set_child(Some(&filters_container));
    pantry_container.append(&pantry_title);
    pantry_container.append(&filters_frame);

    // Pantry Tab UI Structure:
    // - Pantry List Pane (middle): shows all ingredients
    // - Pantry Details Pane (right): shows details for selected ingredient
    // - Navigation Pane (left): handled by main app sidebar, not here
    // The panes are uncoupled except:
    //   - Selecting an ingredient in the List Pane updates the Details Pane
    //   - Changing tab in Navigation triggers List Pane update

    let pantry_container = gtk::Box::new(gtk::Orientation::Vertical, SECTION_SPACING);

    // Title
    let pantry_title = gtk::Label::new(Some(&tr("Pantry")));
    pantry_title.set_markup(&format!(
        "<span size='x-large' weight='bold'>{}</span>",
        tr("Pantry")
    ));
    pantry_title.set_halign(gtk::Align::Start);
    pantry_title.set_margin_all(DEFAULT_MARGIN);

    // Filters frame
    let filters_frame = gtk::Frame::new(None);
    filters_frame.set_margin_bottom(DEFAULT_MARGIN);

    let filters_container = gtk::Box::new(gtk::Orientation::Vertical, TAG_SPACING);
    filters_container.set_margin_all(DEFAULT_MARGIN);

    // Category filters (popover multi-select)
    let category_filters_label = gtk::Label::new(Some(&tr("Categories:")));
    category_filters_label.set_halign(gtk::Align::Start);
    category_filters_label.set_margin_bottom(LIST_ROW_MARGIN);

    // Button to open popover (static, not recreated)
    let filter_button = gtk::Button::with_label(&tr("Filter Categories"));
    filter_button.set_halign(gtk::Align::Start);
    filter_button.set_tooltip_text(Some(&tr("Filter by one or more categories")));

    // Popover for category filters (static)
    let popover = gtk::Popover::new();
    popover.set_parent(&filter_button);
    {
        let popover_weak = popover.downgrade();
        filter_button.connect_destroy(move |_| {
            if let Some(popover) = popover_weak.upgrade() {
                popover.popdown();
                popover.unparent();
            }
        });
    }

    let popover_box = gtk::Box::new(gtk::Orientation::Vertical, TAG_SPACING);
    popover_box.set_margin_all(DEFAULT_MARGIN);
    popover_box.set_vexpand(true);
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_min_content_height(180);
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    let listbox = gtk::Box::new(gtk::Orientation::Vertical, TAG_SPACING);
    scroll.set_child(Some(&listbox));
    popover_box.append(&scroll);
    popover.set_child(Some(&popover_box));

    // Function to refresh the listbox contents
    let refresh_categories = {
        let listbox = listbox.clone();
        let sender = sender.clone();
        move |categories: Vec<String>, selected_categories: Vec<String>| {
            while let Some(child) = listbox.first_child() {
                listbox.remove(&child);
            }
            for category in &categories {
                let check = gtk::CheckButton::with_label(category);
                check.set_active(selected_categories.contains(category));
                let sender_clone = sender.clone();
                let category_clone = category.clone();
                check.connect_toggled(move |btn| {
                    if let Some(sender) = &sender_clone {
                        sender.input(AppMsg::TogglePantryCategory(
                            category_clone.clone(),
                            btn.is_active(),
                        ));
                    }
                });
                listbox.append(&check);
            }
        }
    };
    let categories = if let Some(ref dm) = model.data_manager {
        let cats = dm.as_ref().get_unique_categories();
        cats
    } else {
        Vec::new()
    };
    let selected_categories = model.selected_pantry_categories.clone();
    refresh_categories(categories.clone(), selected_categories.clone());

    let listbox = listbox.clone();
    let sender = sender.clone();
    let sender_for_refresh_categories = sender.clone();
    let refresh_categories_boxed = Box::new(move |model: &AppModel| {
        if let Some(ref dm) = model.data_manager {
            let categories = dm.as_ref().get_unique_categories();
            let selected_categories = model.selected_pantry_categories.clone();
            while let Some(child) = listbox.first_child() {
                listbox.remove(&child);
            }
            for category in &categories {
                let check = gtk::CheckButton::with_label(category);
                check.set_active(selected_categories.contains(category));
                let sender_clone = sender_for_refresh_categories.clone();
                let category_clone = category.clone();
                check.connect_toggled(move |btn| {
                    if let Some(sender) = &sender_clone {
                        sender.input(AppMsg::TogglePantryCategory(
                            category_clone.clone(),
                            btn.is_active(),
                        ));
                    }
                });
                listbox.append(&check);
            }
        } else {
            while let Some(child) = listbox.first_child() {
                listbox.remove(&child);
            }
        }
    });
    filter_button.connect_clicked(move |_| {
        popover.popup();
    });

    let refresh_button = gtk::Button::new();
    let refresh_icon = gtk::Image::from_icon_name("view-refresh-symbolic");
    refresh_icon.set_pixel_size(16);
    refresh_button.set_child(Some(&refresh_icon));
    refresh_button.set_tooltip_text(Some(&tr("Refresh category list")));
    let sender_for_refresh = sender.clone();
    refresh_button.connect_clicked(move |_| {
        if let Some(sender) = &sender_for_refresh {
            sender.input(AppMsg::RefreshCategoryPopover);
        }
    });

    let selected_count = model.selected_pantry_categories.len();
    if selected_count > 0 {
        filter_button.set_label(&format!("{} ({})", tr("Filter Categories"), selected_count));
    }

    let stock_filter_box = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
    let stock_filter_label = gtk::Label::new(Some(&tr("Show in-stock items only:")));
    stock_filter_label.set_halign(gtk::Align::Start);

    let stock_filter_switch = gtk::Switch::new();
    stock_filter_switch.set_state(model.show_in_stock_only);
    let sender_clone = sender.clone();
    stock_filter_switch.connect_state_notify(move |switch| {
        if let Some(sender) = &sender_clone {
            sender.input(AppMsg::ToggleInStockFilter(switch.state()));
        }
    });

    stock_filter_box.append(&stock_filter_label);
    stock_filter_box.append(&stock_filter_switch);

    let filter_buttons_box = gtk::Box::new(gtk::Orientation::Horizontal, TAG_SPACING);
    filter_buttons_box.append(&filter_button);
    filter_buttons_box.append(&refresh_button);

    filters_container.append(&category_filters_label);
    filters_container.append(&filter_buttons_box);
    filters_container.append(&stock_filter_box);

    filters_frame.set_child(Some(&filters_container));
    pantry_container.append(&pantry_title);
    pantry_container.append(&filters_frame);

    // Split view: Pantry List Pane (middle), Pantry Details Pane (right)
    let pantry_content = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
    pantry_content.set_hexpand(true);
    pantry_content.set_vexpand(true);
    pantry_content.set_margin_top(DEFAULT_MARGIN);
    pantry_content.set_margin_start(DEFAULT_MARGIN);
    pantry_content.set_margin_end(DEFAULT_MARGIN);
    pantry_content.set_margin_bottom(DEFAULT_MARGIN);

    // Pantry List Pane
    let pantry_list_scroll = gtk::ScrolledWindow::new();
    pantry_list_scroll.set_hexpand(false);
    pantry_list_scroll.set_vexpand(true);
    pantry_list_scroll.set_min_content_width(300);

    let pantry_list_pane = gtk::ListBox::new();

    pantry_list_scroll.set_child(Some(&pantry_list_pane));

    // Pantry Details Pane
    let pantry_details_pane = gtk::Box::new(gtk::Orientation::Vertical, SECTION_SPACING);
    pantry_details_pane.set_hexpand(true);
    pantry_details_pane.set_vexpand(true);

    let select_label = gtk::Label::new(Some(&tr("Select an ingredient to view details")));
    select_label.set_halign(gtk::Align::Center);
    select_label.set_valign(gtk::Align::Center);
    select_label.set_hexpand(true);
    select_label.set_vexpand(true);
    pantry_details_pane.append(&select_label);

    pantry_content.append(&pantry_list_scroll);
    pantry_content.append(&pantry_details_pane);

    pantry_container.append(&pantry_content);

    // Add button for new ingredient
    let add_button = gtk::Button::with_label(&tr("Add Ingredient"));
    add_button.set_halign(gtk::Align::End);
    let sender_clone = sender.clone();
    add_button.connect_clicked(move |_| {
        if let Some(sender) = &sender_clone {
            sender.input(AppMsg::AddIngredient);
        }
    });
    pantry_container.append(&add_button);

    // --- DEBUG: Breadcrumb for ListBox selection ---
    log::debug!("[build_pantry_tab] Setting up ListBox selection handler");
    let pantry_details_box_clone = pantry_details_pane.clone();
    // let model_clone = model.clone();
    let sender_clone = sender.clone();
    let selected_ingredient = Rc::new(RefCell::new(model.selected_ingredient.clone()));
    let selected_ingredient_ref = selected_ingredient.clone();
    pantry_list_pane.connect_row_selected(move |_listbox, row_opt| {
        log::debug!("[ListBox] row_selected triggered");
        if let Some(row) = row_opt {
            // Retrieve the actual slug from row data
            if let Some(nn_slug) = unsafe { row.data::<String>("slug") } {
                let slug_ref = unsafe { nn_slug.as_ref() };
                log::debug!("[ListBox] Selected slug={:?}", slug_ref);
                // Only send SelectIngredient if the selected slug is different from the model's selected_ingredient
                if let Some(sender) = &sender_clone {
                    if selected_ingredient_ref.borrow().as_deref() != Some(slug_ref.as_str()) {
                        sender.input(AppMsg::SelectIngredient(slug_ref.clone()));
                        *selected_ingredient_ref.borrow_mut() = Some(slug_ref.clone());
                        log::debug!("[ListBox] Sent SelectIngredient message");
                    } else {
                        log::debug!(
                            "[ListBox] Ingredient already selected, not sending message"
                        );
                    }
                }
            } else {
                log::debug!("[ListBox] Selected row has no slug data");
            }
        } else {
            log::debug!("[ListBox] No row selected");
        }
    });

    (
        pantry_container,
        pantry_list_pane,
        pantry_details_pane,
        stock_filter_switch,
        pantry_title,
        Some(refresh_categories_boxed),
    )
}
