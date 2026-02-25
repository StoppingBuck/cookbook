use cookbook_engine::*;
use std::collections::HashMap;

fn fixture_data_dir() -> std::path::PathBuf {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().unwrap().join("example/data")
}

fn setup_temp_data_dir() -> tempfile::TempDir {
    let temp_dir = tempfile::tempdir().unwrap();
    copy_dir_all(&fixture_data_dir(), temp_dir.path()).unwrap();
    temp_dir
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

// ─── Group 1: DataManager construction ───────────────────────────────────────

#[test]
fn test_new_valid_dir() {
    let result = DataManager::new(fixture_data_dir());
    assert!(result.is_ok(), "Expected Ok for a valid data dir, got: {:?}", result.err());
}

#[test]
fn test_new_missing_dir() {
    let result = DataManager::new("/nonexistent/path/that/does/not/exist");
    assert!(result.is_err(), "Expected Err for a missing directory");
}

#[test]
fn test_new_loads_all_collections() {
    let dm = DataManager::new(fixture_data_dir()).unwrap();
    assert_eq!(dm.get_all_ingredients().len(), 9, "Expected 9 ingredients");
    assert_eq!(dm.get_all_recipes().len(), 2, "Expected 2 recipes");
    assert!(dm.get_pantry().is_some(), "Expected pantry to be Some");
}

// ─── Group 2: Ingredient reads ────────────────────────────────────────────────

#[test]
fn test_get_all_ingredients_count() {
    let dm = DataManager::new(fixture_data_dir()).unwrap();
    assert_eq!(dm.get_all_ingredients().len(), 9);
}

#[test]
fn test_get_ingredient_potato() {
    let dm = DataManager::new(fixture_data_dir()).unwrap();
    let ingredient = dm.get_ingredient("potato").expect("Expected to find potato");
    assert_eq!(ingredient.name, "potato");
    assert_eq!(ingredient.slug, "potato");
    assert_eq!(ingredient.category, "vegetable");
}

#[test]
fn test_get_ingredient_missing() {
    let dm = DataManager::new(fixture_data_dir()).unwrap();
    assert!(dm.get_ingredient("nonexistent").is_none());
}

#[test]
fn test_find_by_slug() {
    let dm = DataManager::new(fixture_data_dir()).unwrap();
    let result = dm.find_ingredient_by_name_or_translation("potato", "en");
    assert!(result.is_some(), "Expected to find ingredient by slug 'potato'");
}

#[test]
fn test_find_by_translation_singular() {
    let dm = DataManager::new(fixture_data_dir()).unwrap();
    // potato has translation en.one = "potato"
    let result = dm.find_ingredient_by_name_or_translation("potato", "en");
    assert!(result.is_some(), "Expected to find ingredient by singular translation 'potato'");
    assert_eq!(result.unwrap().name, "potato");
}

#[test]
fn test_find_by_translation_plural() {
    let dm = DataManager::new(fixture_data_dir()).unwrap();
    // potato has translation en.other = "potatoes"
    let result = dm.find_ingredient_by_name_or_translation("potatoes", "en");
    assert!(result.is_some(), "Expected to find ingredient by plural translation 'potatoes'");
    assert_eq!(result.unwrap().name, "potato");
}

// ─── Group 3: Pantry reads ────────────────────────────────────────────────────

#[test]
fn test_get_pantry_some() {
    let dm = DataManager::new(fixture_data_dir()).unwrap();
    assert!(dm.get_pantry().is_some());
}

#[test]
fn test_get_pantry_item_potato() {
    let dm = DataManager::new(fixture_data_dir()).unwrap();
    let item = dm.get_pantry_item("potato").expect("Expected to find potato in pantry");
    assert_eq!(item.ingredient, "potato");
    assert_eq!(item.quantity, Some(2.0));
    assert_eq!(item.quantity_type, "kg");
}

#[test]
fn test_is_in_pantry_true() {
    let dm = DataManager::new(fixture_data_dir()).unwrap();
    assert!(dm.is_in_pantry("potato"));
}

#[test]
fn test_is_in_pantry_false() {
    let dm = DataManager::new(fixture_data_dir()).unwrap();
    assert!(!dm.is_in_pantry("flour"));
}

// ─── Group 4: Recipe reads ────────────────────────────────────────────────────

#[test]
fn test_get_all_recipes_count() {
    let dm = DataManager::new(fixture_data_dir()).unwrap();
    assert_eq!(dm.get_all_recipes().len(), 2);
}

#[test]
fn test_get_recipe_lasagna() {
    let dm = DataManager::new(fixture_data_dir()).unwrap();
    let recipe = dm.get_recipe("Lasagna").expect("Expected to find Lasagna");
    assert_eq!(recipe.title, "Lasagna");
    assert_eq!(recipe.prep_time, Some(30));
    assert_eq!(recipe.downtime, Some(60));
    assert_eq!(recipe.servings, Some(2));
}

#[test]
fn test_get_recipe_missing() {
    let dm = DataManager::new(fixture_data_dir()).unwrap();
    assert!(dm.get_recipe("Nonexistent").is_none());
}

#[test]
fn test_recipe_total_time() {
    let dm = DataManager::new(fixture_data_dir()).unwrap();
    let recipe = dm.get_recipe("Lasagna").expect("Expected to find Lasagna");
    assert_eq!(recipe.total_time(), 90);
}

#[test]
fn test_search_recipes_by_title_case_insensitive() {
    let dm = DataManager::new(fixture_data_dir()).unwrap();
    let results = dm.search_recipes("lasagna");
    assert!(
        results.iter().any(|r| r.title == "Lasagna"),
        "Expected 'Lasagna' in search results for 'lasagna'"
    );
}

#[test]
fn test_search_recipes_by_ingredient() {
    let dm = DataManager::new(fixture_data_dir()).unwrap();
    let results = dm.search_recipes("potato");
    assert!(
        !results.is_empty(),
        "Expected at least one recipe with potato ingredient"
    );
}

// ─── Group 5: Filtering ───────────────────────────────────────────────────────

#[test]
fn test_filter_by_search_text() {
    let dm = DataManager::new(fixture_data_dir()).unwrap();
    let results = dm.filter_ingredients("pot", &[], false, "en");
    assert!(
        results.iter().any(|i| i.name == "potato"),
        "Expected 'potato' in filter results for 'pot'"
    );
}

#[test]
fn test_filter_by_category() {
    let dm = DataManager::new(fixture_data_dir()).unwrap();
    let results = dm.filter_ingredients("", &["vegetable".to_string()], false, "en");
    assert!(!results.is_empty(), "Expected at least one vegetable ingredient");
    for ingredient in &results {
        assert_eq!(
            ingredient.category, "vegetable",
            "Expected all results to have category 'vegetable', but got '{}' for '{}'",
            ingredient.category, ingredient.name
        );
    }
}

#[test]
fn test_filter_in_stock_only() {
    let dm = DataManager::new(fixture_data_dir()).unwrap();
    // Ingredients in both ingredients dir AND pantry:
    // tomato, egg, butter, potato, salt, pepper = 6 items
    let results = dm.filter_ingredients("", &[], true, "en");
    assert_eq!(
        results.len(),
        6,
        "Expected exactly 6 in-stock ingredients, got: {:?}",
        results.iter().map(|i| &i.name).collect::<Vec<_>>()
    );
    for ingredient in &results {
        assert!(
            dm.is_in_pantry(&ingredient.name),
            "Expected '{}' to be in pantry",
            ingredient.name
        );
    }
}

#[test]
fn test_get_all_ingredient_categories_sorted() {
    let dm = DataManager::new(fixture_data_dir()).unwrap();
    let categories = dm.get_all_ingredient_categories();
    assert!(!categories.is_empty(), "Expected at least one category");
    assert!(
        categories.contains(&"vegetable".to_string()),
        "Expected categories to contain 'vegetable'"
    );
    // Verify it is sorted
    let mut sorted = categories.clone();
    sorted.sort();
    assert_eq!(categories, sorted, "Expected categories to be sorted");
}

// ─── Group 6: Mutations (use temp dir) ───────────────────────────────────────

#[test]
fn test_update_pantry_item_persists() {
    let temp_dir = setup_temp_data_dir();
    let temp_path = temp_dir.path().to_path_buf();

    {
        let mut dm = DataManager::new(&temp_path).unwrap();
        dm.update_pantry_item("potato", Some(5.0), Some("lbs".to_string())).unwrap();
    }

    // Reload and verify
    let dm2 = DataManager::new(&temp_path).unwrap();
    let item = dm2.get_pantry_item("potato").expect("Expected potato in pantry after update");
    assert_eq!(item.quantity, Some(5.0), "Expected quantity to be 5.0 after update");
    assert_eq!(item.quantity_type, "lbs", "Expected quantity_type to be 'lbs' after update");
}

#[test]
fn test_remove_from_pantry_persists() {
    let temp_dir = setup_temp_data_dir();
    let temp_path = temp_dir.path().to_path_buf();

    {
        let mut dm = DataManager::new(&temp_path).unwrap();
        assert!(dm.is_in_pantry("potato"), "Potato should be in pantry before removal");
        dm.remove_from_pantry("potato").unwrap();
    }

    // Reload and verify
    let dm2 = DataManager::new(&temp_path).unwrap();
    assert!(!dm2.is_in_pantry("potato"), "Expected potato to be removed from pantry");
}

#[test]
fn test_create_ingredient_persists() {
    let temp_dir = setup_temp_data_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let new_ingredient = Ingredient {
        name: "garlic".to_string(),
        slug: "garlic".to_string(),
        category: "vegetable".to_string(),
        kb: None,
        tags: Some(vec!["vegetable".to_string()]),
        translations: None,
    };

    {
        let mut dm = DataManager::new(&temp_path).unwrap();
        dm.create_ingredient(new_ingredient).unwrap();
    }

    // Reload and verify
    let dm2 = DataManager::new(&temp_path).unwrap();
    let found = dm2.get_ingredient("garlic");
    assert!(found.is_some(), "Expected 'garlic' to be found after create");
    assert_eq!(found.unwrap().name, "garlic");
    assert_eq!(found.unwrap().category, "vegetable");
}

#[test]
fn test_update_ingredient_renames_file() {
    let temp_dir = setup_temp_data_dir();
    let temp_path = temp_dir.path().to_path_buf();

    let renamed = Ingredient {
        name: "sweet_potato".to_string(),
        slug: "sweet_potato".to_string(),
        category: "vegetable".to_string(),
        kb: None,
        tags: None,
        translations: None,
    };

    {
        let mut dm = DataManager::new(&temp_path).unwrap();
        assert!(dm.get_ingredient("potato").is_some(), "potato should exist before rename");
        dm.update_ingredient("potato", renamed).unwrap();
    }

    // Reload and verify
    let dm2 = DataManager::new(&temp_path).unwrap();
    assert!(
        dm2.get_ingredient("potato").is_none(),
        "Expected 'potato' to be gone after rename"
    );
    assert!(
        dm2.get_ingredient("sweet_potato").is_some(),
        "Expected 'sweet_potato' to exist after rename"
    );
}

#[test]
fn test_delete_ingredient_removes_from_pantry() {
    let temp_dir = setup_temp_data_dir();
    let temp_path = temp_dir.path().to_path_buf();

    {
        let mut dm = DataManager::new(&temp_path).unwrap();
        assert!(dm.is_in_pantry("potato"), "potato should be in pantry before deletion");
        dm.delete_ingredient("potato").unwrap();
    }

    // Reload and verify
    let dm2 = DataManager::new(&temp_path).unwrap();
    assert!(
        dm2.get_ingredient("potato").is_none(),
        "Expected 'potato' to be gone from ingredients after deletion"
    );
    assert!(
        !dm2.is_in_pantry("potato"),
        "Expected 'potato' to be gone from pantry after ingredient deletion"
    );
}

// ─── Group 7: Display logic ───────────────────────────────────────────────────

#[test]
fn test_ingredient_display_name_singular() {
    let mut translations = HashMap::new();
    translations.insert(
        "en".to_string(),
        TranslationForms {
            one: "potato".to_string(),
            other: "potatoes".to_string(),
        },
    );
    let ingredient = Ingredient {
        name: "potato".to_string(),
        slug: "potato".to_string(),
        category: "vegetable".to_string(),
        kb: None,
        tags: None,
        translations: Some(translations),
    };
    let display = DataManager::ingredient_display_name(&ingredient, "en", Some(1.0));
    assert_eq!(display, "potato");
}

#[test]
fn test_ingredient_display_name_plural() {
    let mut translations = HashMap::new();
    translations.insert(
        "en".to_string(),
        TranslationForms {
            one: "potato".to_string(),
            other: "potatoes".to_string(),
        },
    );
    let ingredient = Ingredient {
        name: "potato".to_string(),
        slug: "potato".to_string(),
        category: "vegetable".to_string(),
        kb: None,
        tags: None,
        translations: Some(translations),
    };
    let display = DataManager::ingredient_display_name(&ingredient, "en", Some(2.0));
    assert_eq!(display, "potatoes");
}

#[test]
fn test_ingredient_display_name_fallback() {
    let ingredient = Ingredient {
        name: "mystery_herb".to_string(),
        slug: "mystery_herb".to_string(),
        category: "herb".to_string(),
        kb: None,
        tags: None,
        translations: None,
    };
    let display = DataManager::ingredient_display_name(&ingredient, "en", Some(1.0));
    assert_eq!(display, "mystery_herb");
}

#[test]
fn test_are_all_ingredients_in_pantry_true() {
    let dm = DataManager::new(fixture_data_dir()).unwrap();
    // Lasagna uses potato and tomato, both are in pantry
    let result = dm.are_all_ingredients_in_pantry("Lasagna");
    assert!(result, "Expected all Lasagna ingredients to be in pantry");
}

#[test]
fn test_are_all_ingredients_in_pantry_false() {
    let dm = DataManager::new(fixture_data_dir()).unwrap();
    // Spaghetti Aglio e Olio uses spaghetti, garlic, olive oil, etc. — none are in pantry
    // (salt and pepper are in pantry, but spaghetti, garlic, etc. are not)
    let result = dm.are_all_ingredients_in_pantry("Spaghetti Aglio e Olio");
    assert!(!result, "Expected not all Spaghetti Aglio e Olio ingredients to be in pantry");
}

// ─── Group 8: YAML round-trips ────────────────────────────────────────────────

#[test]
fn test_ingredient_yaml_roundtrip() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test_ingredient.yaml");

    let mut translations = HashMap::new();
    translations.insert(
        "en".to_string(),
        TranslationForms {
            one: "carrot".to_string(),
            other: "carrots".to_string(),
        },
    );
    let original = Ingredient {
        name: "carrot".to_string(),
        slug: "carrot".to_string(),
        category: "vegetable".to_string(),
        kb: Some("carrot".to_string()),
        tags: Some(vec!["root".to_string(), "vegetable".to_string()]),
        translations: Some(translations),
    };

    original.to_file(&path).unwrap();
    let loaded = Ingredient::from_file(&path).unwrap();

    assert_eq!(loaded.name, original.name);
    assert_eq!(loaded.slug, original.slug);
    assert_eq!(loaded.category, original.category);
    assert_eq!(loaded.kb, original.kb);
    assert_eq!(loaded.tags, original.tags);
    let loaded_translations = loaded.translations.expect("Expected translations");
    let orig_translations = original.translations.expect("Expected translations");
    let loaded_en = loaded_translations.get("en").expect("Expected 'en' translation");
    let orig_en = orig_translations.get("en").expect("Expected 'en' translation");
    assert_eq!(loaded_en.one, orig_en.one);
    assert_eq!(loaded_en.other, orig_en.other);
}

#[test]
fn test_pantry_yaml_roundtrip() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test_pantry.yaml");

    let original = Pantry {
        version: 1,
        items: vec![
            PantryItem {
                ingredient: "carrot".to_string(),
                quantity: Some(3.0),
                quantity_type: "kg".to_string(),
                last_updated: "2026-01-01".to_string(),
            },
            PantryItem {
                ingredient: "onion".to_string(),
                quantity: None,
                quantity_type: "".to_string(),
                last_updated: "2026-01-02".to_string(),
            },
        ],
    };

    original.to_file(&path).unwrap();
    let loaded = Pantry::from_file(&path).unwrap();

    assert_eq!(loaded.version, original.version);
    assert_eq!(loaded.items.len(), original.items.len());

    let carrot = loaded.items.iter().find(|i| i.ingredient == "carrot").expect("Expected carrot");
    assert_eq!(carrot.quantity, Some(3.0));
    assert_eq!(carrot.quantity_type, "kg");
    assert_eq!(carrot.last_updated, "2026-01-01");

    let onion = loaded.items.iter().find(|i| i.ingredient == "onion").expect("Expected onion");
    assert_eq!(onion.quantity, None);
    assert_eq!(onion.last_updated, "2026-01-02");
}

#[test]
fn test_recipe_roundtrip() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("Test_Recipe.md");

    let original = Recipe {
        title: "Test Recipe".to_string(),
        ingredients: vec![
            RecipeIngredient {
                ingredient: "flour".to_string(),
                quantity: Some(200.0),
                quantity_type: Some("g".to_string()),
            },
            RecipeIngredient {
                ingredient: "egg".to_string(),
                quantity: Some(2.0),
                quantity_type: None,
            },
        ],
        prep_time: Some(15),
        downtime: Some(45),
        servings: Some(4),
        tags: Some(vec!["baking".to_string(), "dessert".to_string()]),
        image: None,
        instructions: "Mix flour and eggs. Bake for 45 minutes.".to_string(),
    };

    original.to_file(&path).unwrap();
    let loaded = Recipe::from_file(&path).unwrap();

    assert_eq!(loaded.title, original.title);
    assert_eq!(loaded.prep_time, original.prep_time);
    assert_eq!(loaded.downtime, original.downtime);
    assert_eq!(loaded.servings, original.servings);
    assert_eq!(loaded.tags, original.tags);
    assert_eq!(loaded.instructions, original.instructions);
    assert_eq!(loaded.ingredients.len(), original.ingredients.len());

    let flour_ing = loaded
        .ingredients
        .iter()
        .find(|i| i.ingredient == "flour")
        .expect("Expected flour ingredient");
    assert_eq!(flour_ing.quantity, Some(200.0));
    assert_eq!(flour_ing.quantity_type.as_deref(), Some("g"));

    let egg_ing = loaded
        .ingredients
        .iter()
        .find(|i| i.ingredient == "egg")
        .expect("Expected egg ingredient");
    assert_eq!(egg_ing.quantity, Some(2.0));
}
