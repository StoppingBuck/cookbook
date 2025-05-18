use cookbook_engine::DataManager;
use gtk::prelude::*;
use relm4::gtk;
use relm4::ComponentSender;
use relm4::RelmWidgetExt;
use std::rc::Rc;
use regex::Regex;
use relm4::gtk::glib;

use crate::types::AppModel;
use crate::types::AppMsg;
use crate::ui_constants::*;
use crate::utils;

/// Updates the KB entry list based on search text and other filters
pub fn update_kb_list<C>(
    kb_list_box: &gtk::ListBox,
    data_manager: &Option<Rc<DataManager>>,
    sender: &ComponentSender<C>,
    select_kb_entry_msg: impl Fn(String) -> C::Input + Clone + 'static,
) where
    C: relm4::Component,
{
    // Clear the KB list
    utils::clear_list_box(&kb_list_box);


    if let Some(ref dm) = data_manager {
        let entries = dm.get_all_kb_entries();
        if !entries.is_empty() {
            // Sort entries by title for better usability
            let mut sorted_entries = entries.clone();
            sorted_entries.sort_by(|a, b| a.title.cmp(&b.title));

            for entry in sorted_entries {
                let row = gtk::ListBoxRow::new();
                let title_label = gtk::Label::new(Some(&entry.title));
                title_label.set_halign(gtk::Align::Start);
                title_label.set_margin_start(5);
                title_label.set_margin_end(5);
                title_label.set_margin_top(5);
                title_label.set_margin_bottom(LIST_ROW_MARGIN);
                row.set_child(Some(&title_label));

                kb_list_box.append(&row);

                // Store the slug in the row's data to make retrieval easier
                row.set_widget_name(&entry.slug);
            }

            // Set up row selection handler
            let sender_clone = sender.clone();
            let select_msg = select_kb_entry_msg.clone();
            kb_list_box.connect_row_selected(move |_, row_opt| {
                if let Some(row) = row_opt {
                    let slug = row.widget_name().to_string();
                    sender_clone.input(select_msg(slug));
                }
            });
        } else {
            let no_entries_row = gtk::ListBoxRow::new();
            let no_entries_label = gtk::Label::new(Some("No KB entries available"));
            no_entries_label.set_margin_all(DEFAULT_MARGIN);
            no_entries_row.set_child(Some(&no_entries_label));
            kb_list_box.append(&no_entries_row);
        }
    } else {
        let no_data_row = gtk::ListBoxRow::new();
        let no_data_label = gtk::Label::new(Some("Failed to load KB data"));
        no_data_label.set_margin_all(DEFAULT_MARGIN);
        no_data_row.set_child(Some(&no_data_label));
        kb_list_box.append(&no_data_row);
    }
}

/// Converts a subset of Markdown to Pango markup for GTK labels.
/// Supports headings, bold, italic, unordered lists, and links.
fn markdown_to_pango(md: &str) -> String {
    let mut out = String::new();
    let bold_re = Regex::new(r"\*\*(.+?)\*\*").unwrap();
    let italic_re = Regex::new(r"\*(.+?)\*").unwrap();
    let link_re = Regex::new(r"\[(.+?)\]\((.+?)\)").unwrap();
    for line in md.lines() {
        let trimmed = line.trim();
        let mut pango_line = String::new();
        // Headings
        if trimmed.starts_with("### ") {
            pango_line.push_str(&format!("<span size='large' weight='bold'>{}</span>", &trimmed[4..]));
        } else if trimmed.starts_with("## ") {
            pango_line.push_str(&format!("<span size='x-large' weight='bold'>{}</span>", &trimmed[3..]));
        } else if trimmed.starts_with("# ") {
            pango_line.push_str(&format!("<span size='xx-large' weight='bold'>{}</span>", &trimmed[2..]));
        } else if trimmed.starts_with("* ") || trimmed.starts_with("- ") {
            // Unordered list
            pango_line.push_str(&format!("• {}", &trimmed[2..]));
        } else {
            pango_line.push_str(trimmed);
        }
        // Inline: links, bold, italic (order: links -> bold -> italic)
        let pango_line = link_re.replace_all(&pango_line, |caps: &regex::Captures| {
            // Render as underlined text with URL in parentheses
            format!("<u>{}</u> ({})", &caps[1], &caps[2])
        });
        let pango_line = bold_re.replace_all(&pango_line, |caps: &regex::Captures| {
            format!("<b>{}</b>", &caps[1])
        });
        let pango_line = italic_re.replace_all(&pango_line, |caps: &regex::Captures| {
            // Avoid matching inside bold
            if caps[1].contains("<b>") { caps[0].to_string() } else { format!("<i>{}</i>", &caps[1]) }
        });
        out.push_str(&pango_line);
        out.push('\n');
    }
    out.trim().to_string()
}

pub fn build_kb_detail_view(
    data_manager: &Rc<DataManager>,
    kb_slug: &str,
) -> gtk::ScrolledWindow {
    let kb_details_scroll = gtk::ScrolledWindow::new();
    kb_details_scroll.set_hexpand(true);
    kb_details_scroll.set_vexpand(true); // Allow vertical expansion

    // Find the selected KB entry
    if let Some(kb_entry) = data_manager.get_kb_entry(kb_slug) {
        let details_container = gtk::Box::new(gtk::Orientation::Vertical, SECTION_SPACING);
        details_container.set_margin_all(DEFAULT_MARGIN);
        details_container.set_vexpand(true); // Allow vertical expansion

        // Title
        let title = gtk::Label::new(None);
        title.set_markup(&format!(
            "<span size='x-large' weight='bold'>{}</span>",
            kb_entry.title
        ));
        title.set_halign(gtk::Align::Start);
        title.set_margin_bottom(DEFAULT_MARGIN);
        details_container.append(&title);

        // Image (if available)
        if let Some(image_name) = &kb_entry.image {
            if let Some(image_path) = data_manager.get_kb_image_path(image_name) {
                if image_path.exists() {
                    match gtk::gdk_pixbuf::Pixbuf::from_file(&image_path) {
                        Ok(pixbuf) => {
                            let aspect = pixbuf.width() as f32 / pixbuf.height() as f32;
                            let image = gtk::Image::from_pixbuf(Some(&pixbuf));
                            image.set_hexpand(true);
                            image.set_vexpand(true);
                            // Use GtkAspectFrame to make the image scale with the window, preserving aspect ratio
                            let aspect_frame = gtk::AspectFrame::new(0.5, 0.0, aspect, false);
                            aspect_frame.set_hexpand(true);
                            aspect_frame.set_vexpand(true);
                            aspect_frame.set_halign(gtk::Align::Fill);
                            aspect_frame.set_valign(gtk::Align::Fill); // Allow vertical expansion
                            aspect_frame.set_child(Some(&image));
                            aspect_frame.set_margin_bottom(HEADER_MARGIN);
                            details_container.append(&aspect_frame);

                            // Print debug widget sizes only once using RefCell<bool>
                            use std::cell::RefCell;
                            let printed = std::rc::Rc::new(RefCell::new(false));
                            let details_container_clone = details_container.clone();
                            let aspect_frame_clone = aspect_frame.clone();
                            let image_clone = image.clone();
                            let printed_clone = printed.clone();
                            aspect_frame.add_tick_callback(move |_, _| {
                                let mut printed = printed_clone.borrow_mut();
                                if !*printed {
                                    let cont_alloc = details_container_clone.allocation();
                                    let alloc = aspect_frame_clone.allocation();
                                    let img_alloc = image_clone.allocation();
                                    let window = aspect_frame_clone.root();
                                    let (win_w, win_h) = if let Some(window) = window.and_downcast::<gtk::Window>() {
                                        let alloc = window.allocation();
                                        (alloc.width(), alloc.height())
                                    } else { (0, 0) };
                                    // Only print if all allocations are nonzero
                                    if win_w > 0 && win_h > 0 && cont_alloc.width() > 0 && cont_alloc.height() > 0 && alloc.width() > 0 && alloc.height() > 0 && img_alloc.width() > 0 && img_alloc.height() > 0 {
                                        println!("[KB DEBUG] Window size: {}x{} | details_container: {}x{} | aspect_frame: {}x{} | image: {}x{}", win_w, win_h, cont_alloc.width(), cont_alloc.height(), alloc.width(), alloc.height(), img_alloc.width(), img_alloc.height());
                                        *printed = true;
                                        return glib::ControlFlow::Break;
                                    }
                                }
                                glib::ControlFlow::Continue
                            });
                        }
                        Err(_) => {
                            let missing_label = gtk::Label::new(Some("Image not available"));
                            missing_label.set_halign(gtk::Align::Center);
                            missing_label.set_margin_bottom(HEADER_MARGIN);
                            details_container.append(&missing_label);
                        }
                    }
                } else {
                    let missing_label = gtk::Label::new(Some("Image not available"));
                    missing_label.set_halign(gtk::Align::Center);
                    missing_label.set_margin_bottom(HEADER_MARGIN);
                    details_container.append(&missing_label);
                }
            } else {
                let missing_label = gtk::Label::new(Some("Image not available"));
                missing_label.set_halign(gtk::Align::Center);
                missing_label.set_margin_bottom(HEADER_MARGIN);
                details_container.append(&missing_label);
            }
        }

        // Related ingredients section (if any)
        let related_ingredients = data_manager.get_ingredients_with_kb_reference(kb_slug);
        if !related_ingredients.is_empty() {
            let related_label = gtk::Label::new(None);
            related_label.set_markup("<span weight='bold'>Related Ingredients:</span>");
            related_label.set_halign(gtk::Align::Start);
            related_label.set_margin_top(LIST_ROW_MARGIN);
            related_label.set_margin_bottom(LIST_ROW_MARGIN);
            details_container.append(&related_label);

            let ingredients_box = gtk::Box::new(gtk::Orientation::Horizontal, TAG_SPACING);
            ingredients_box.set_margin_start(DEFAULT_MARGIN);
            ingredients_box.set_margin_bottom(DEFAULT_MARGIN);

            for ingredient in related_ingredients {
                let ingredient_chip = gtk::Button::with_label(&ingredient.name);
                ingredient_chip.add_css_class("tag");
                ingredients_box.append(&ingredient_chip);
            }

            details_container.append(&ingredients_box);
            details_container.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        }

        // Content (rendered as markdown to pango)
        let pango_markup = markdown_to_pango(&kb_entry.content);
        let content_text = gtk::Label::new(None);
        // Heuristic: fallback to plain text if markup looks broken
        let open_spans = pango_markup.matches("<span").count();
        let close_spans = pango_markup.matches("</span>").count();
        if open_spans == close_spans && !pango_markup.contains("</span>\n\n</span>") {
            content_text.set_markup(&pango_markup);
        } else {
            content_text.set_text(&kb_entry.content);
        }
        content_text.set_halign(gtk::Align::Start);
        content_text.set_wrap(true);
        content_text.set_xalign(0.0);
        content_text.set_margin_top(DEFAULT_MARGIN);
        details_container.append(&content_text);

        kb_details_scroll.set_child(Some(&details_container));
    } else {
        // KB entry not found
        let not_found_label = gtk::Label::new(Some(&format!(
            "Knowledge Base entry '{}' not found",
            kb_slug
        )));
        not_found_label.set_halign(gtk::Align::Center);
        not_found_label.set_valign(gtk::Align::Center);
        kb_details_scroll.set_child(Some(&not_found_label));
    }

    kb_details_scroll
}

/// Updates the KB details view with the selected entry
pub fn update_kb_details<C>(
    kb_details: &gtk::Box,
    data_manager: &Option<Rc<DataManager>>,
    kb_slug: &str,
) where
    C: relm4::Component,
{
    // Clear previous content
    utils::clear_box(kb_details);

    if let Some(ref dm) = data_manager {
        let kb_details_scroll = build_kb_detail_view(dm, kb_slug);
        kb_details.append(&kb_details_scroll);
    } else {
        // Data manager not available
        let error_label =
            gtk::Label::new(Some("Unable to load KB entry: data manager not available"));
        error_label.set_halign(gtk::Align::Center);
        error_label.set_valign(gtk::Align::Center);
        kb_details.append(&error_label);
    }
}

/// Shows a placeholder when no KB entry is selected
pub fn show_kb_details_placeholder(kb_details: &gtk::Box) {
    // Clear previous content
    utils::clear_box(&kb_details);


    let select_label = gtk::Label::new(Some("Select an item to view details"));
    select_label.set_halign(gtk::Align::Center);
    select_label.set_valign(gtk::Align::Center);
    kb_details.append(&select_label);
}

/// Helper function to select the correct KB entry in the list box
#[allow(dead_code)]
pub fn select_kb_entry_in_list(kb_list_box: &gtk::ListBox, kb_slug: &str) {
    // First try to find by widget name (which should contain the slug)
    let mut i = 0;
    while let Some(row) = kb_list_box.row_at_index(i) {
        i += 1;
        if row.widget_name() == kb_slug {
            kb_list_box.select_row(Some(&row));
            return;
        }
    }

    // If that fails, try with the label text (backward compatibility)
    i = 0;
    while let Some(row) = kb_list_box.row_at_index(i) {
        i += 1;
        if let Some(child) = row.child() {
            if let Some(label) = child.downcast_ref::<gtk::Label>() {
                if label.text() == kb_slug {
                    kb_list_box.select_row(Some(&row));
                    return;
                }
            }
        }
    }
}

pub fn build_kb_tab(
    model: &AppModel,
    sender: &ComponentSender<AppModel>,
) -> (gtk::Box, gtk::ListBox, gtk::Box, gtk::Label) {
    // Main container
    let kb_container = gtk::Box::new(gtk::Orientation::Vertical, SECTION_SPACING);

    // Title
    let kb_title = gtk::Label::new(Some("Knowledge Base"));
    kb_title.set_markup("<span size='x-large' weight='bold'>Knowledge Base</span>");
    kb_title.set_halign(gtk::Align::Start);
    kb_title.set_margin_all(DEFAULT_MARGIN);
    kb_container.append(&kb_title);

    // Split view
    let kb_content = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
    kb_content.set_hexpand(true);
    kb_content.set_vexpand(true);

    // List
    let kb_list_scroll = gtk::ScrolledWindow::new();
    kb_list_scroll.set_hexpand(false);
    kb_list_scroll.set_vexpand(true);
    kb_list_scroll.set_min_content_width(250);

    let kb_list_box = gtk::ListBox::new();
    kb_list_box.set_selection_mode(gtk::SelectionMode::Single);

    // Populate list
    if let Some(ref dm) = model.data_manager {
        let entries = dm.get_all_kb_entries();
        if !entries.is_empty() {
            let mut sorted_entries = entries.clone();
            sorted_entries.sort_by(|a, b| a.title.cmp(&b.title));
            for entry in sorted_entries {
                let row = gtk::ListBoxRow::new();
                let title_label = gtk::Label::new(Some(&entry.title));
                title_label.set_halign(gtk::Align::Start);
                title_label.set_margin_start(LIST_ROW_MARGIN);
                title_label.set_margin_end(LIST_ROW_MARGIN);
                title_label.set_margin_top(LIST_ROW_MARGIN);
                title_label.set_margin_bottom(LIST_ROW_MARGIN);
                row.set_child(Some(&title_label));
                // Store the slug in the row's widget name for retrieval
                row.set_widget_name(&entry.slug);
                kb_list_box.append(&row);
            }

            // Add this after populating the list:
            let sender_clone = sender.clone();
            kb_list_box.connect_row_selected(move |_, row_opt| {
                if let Some(row) = row_opt {
                    let slug = row.widget_name().to_string();
                    sender_clone.input(AppMsg::SelectKnowledgeBaseEntry(slug));
                }
            });
        } else {
            let no_entries_row = gtk::ListBoxRow::new();
            let no_entries_label = gtk::Label::new(Some("No KB entries available"));
            no_entries_label.set_margin_all(DEFAULT_MARGIN);
            no_entries_row.set_child(Some(&no_entries_label));
            kb_list_box.append(&no_entries_row);
        }
    } else {
        let no_data_row = gtk::ListBoxRow::new();
        let no_data_label = gtk::Label::new(Some("Failed to load KB data"));
        no_data_label.set_margin_all(DEFAULT_MARGIN);
        no_data_row.set_child(Some(&no_data_label));
        kb_list_box.append(&no_data_row);
    }

    kb_list_scroll.set_child(Some(&kb_list_box));

    // Details
    let kb_details = gtk::Box::new(gtk::Orientation::Vertical, SECTION_SPACING);
    kb_details.set_hexpand(true);
    kb_details.set_vexpand(true);

    let kb_label = gtk::Label::new(Some("Select an item to view details"));
    kb_label.set_halign(gtk::Align::Center);
    kb_label.set_valign(gtk::Align::Center);
    kb_label.set_hexpand(true);
    kb_label.set_vexpand(true);

    kb_details.append(&kb_label);

    kb_content.append(&kb_list_scroll);
    kb_content.append(&kb_details);

    kb_container.append(&kb_content);

    (kb_container, kb_list_box, kb_details, kb_label)
}
