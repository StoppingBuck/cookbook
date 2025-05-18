use gtk::prelude::*;
use std::fs;
use std::path::Path;

pub fn clear_box(container: &gtk::Box) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

pub fn clear_list_box(container: &gtk::ListBox) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

pub fn validate_and_create_data_dir<P: AsRef<Path>>(data_dir: P) {
    let data_dir = data_dir.as_ref();
    let ingredients = data_dir.join("ingredients");
    let recipes = data_dir.join("recipes");
    let recipes_img = recipes.join("img");
    let pantry = data_dir.join("pantry.yaml");

    if !ingredients.exists() {
        let _ = fs::create_dir_all(&ingredients);
    }
    if !recipes.exists() {
        let _ = fs::create_dir_all(&recipes);
    }
    if !recipes_img.exists() {
        let _ = fs::create_dir_all(&recipes_img);
    }
    if !pantry.exists() {
        // Create a minimal pantry.yaml if missing
        let content = "version: 1\nitems: []\n";
        let _ = fs::write(&pantry, content);
    }
}