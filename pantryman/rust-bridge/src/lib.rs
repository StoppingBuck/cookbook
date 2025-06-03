use jni::objects::{JClass, JString};
use jni::sys::{jstring, jboolean, jlong};
use jni::JNIEnv;
use cookbook_engine::{DataManager, Ingredient};
use std::path::Path;

#[cfg(target_os = "android")]
use android_logger::{Config, FilterBuilder};
use log::{debug, info, warn, error};

// Initialize Android logging
#[cfg(target_os = "android")]
fn init_logging() {
    android_logger::init_once(
        Config::default()
            .with_max_level(log::LevelFilter::Debug)
            .with_tag("PantrymanRust")
    );
}

#[cfg(not(target_os = "android"))]
fn init_logging() {
    // Do nothing on non-Android platforms
}

// Helper function to convert Java string to Rust string
fn jstring_to_string(env: &mut JNIEnv, jstr: &JString) -> Result<String, Box<dyn std::error::Error>> {
    let rust_string: String = env.get_string(jstr)?.into();
    Ok(rust_string)
}

// Helper function to convert Rust string to Java string
fn string_to_jstring(env: &mut JNIEnv, rust_string: String) -> Result<jstring, Box<dyn std::error::Error>> {
    let java_string = env.new_string(rust_string)?;
    Ok(java_string.into_raw())
}

// Initialize DataManager
#[no_mangle]
pub extern "system" fn Java_com_example_pantryman_CookbookEngine_createDataManager(
    mut env: JNIEnv,
    _class: JClass,
    data_dir_path: JString,
) -> jlong {
    // Initialize logging first
    init_logging();
    
    let data_dir = match jstring_to_string(&mut env, &data_dir_path) {
        Ok(path) => path,
        Err(e) => {
            error!("Failed to convert JString to String: {:?}", e);
            return 0; // Return null pointer on error
        },
    };
    
    info!("Attempting to create DataManager with path: {}", data_dir);
    
    // Let's also check if the directory exists and what's in it
    let path = Path::new(&data_dir);
    if !path.exists() {
        error!("Data directory does not exist: {}", data_dir);
        return 0;
    }
    
    info!("Data directory exists, checking contents...");
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                info!("Found file/dir: {:?}", entry.path());
            }
        }
    }
    
    match DataManager::new(path) {
        Ok(manager) => {
            info!("DataManager created successfully");
            Box::into_raw(Box::new(manager)) as jlong
        },
        Err(e) => {
            error!("Failed to create DataManager: {:?}", e);
            0 // Return null pointer on error
        },
    }
}

// Destroy DataManager
#[no_mangle]
pub extern "system" fn Java_com_example_pantryman_CookbookEngine_destroyDataManager(
    mut _env: JNIEnv,
    _class: JClass,
    manager_ptr: jlong,
) {
    if manager_ptr != 0 {
        unsafe {
            let _manager = Box::from_raw(manager_ptr as *mut DataManager);
            // Box will be dropped automatically
        }
    }
}

// Get all ingredients as JSON
#[no_mangle]
pub extern "system" fn Java_com_example_pantryman_CookbookEngine_getAllIngredientsJson(
    mut env: JNIEnv,
    _class: JClass,
    manager_ptr: jlong,
) -> jstring {
    if manager_ptr == 0 {
        return string_to_jstring(&mut env, "[]".to_string()).unwrap_or(std::ptr::null_mut());
    }
    
    let manager = unsafe { &*(manager_ptr as *const DataManager) };
    let ingredients = manager.get_all_ingredients();
    
    let mut ingredient_data = Vec::new();
    for ingredient in ingredients {
        let pantry_item = manager.get_pantry_item(&ingredient.name);
        let is_in_stock = manager.is_in_pantry(&ingredient.name);
        
        let (quantity, quantity_type) = if let Some(item) = pantry_item {
            (item.quantity, Some(item.quantity_type.clone()))
        } else {
            (None, None)
        };
        
        let ingredient_json = serde_json::json!({
            "name": ingredient.name,
            "slug": ingredient.slug,
            "category": ingredient.category,
            "kb": ingredient.kb,
            "tags": ingredient.tags,
            "isInStock": is_in_stock,
            "quantity": quantity,
            "quantityType": quantity_type
        });
        
        ingredient_data.push(ingredient_json);
    }
    
    match serde_json::to_string(&ingredient_data) {
        Ok(json) => string_to_jstring(&mut env, json).unwrap_or(std::ptr::null_mut()),
        Err(_) => string_to_jstring(&mut env, "[]".to_string()).unwrap_or(std::ptr::null_mut()),
    }
}

// Get ingredients by category as JSON
#[no_mangle]
pub extern "system" fn Java_com_example_pantryman_CookbookEngine_getIngredientsByCategoryJson(
    mut env: JNIEnv,
    _class: JClass,
    manager_ptr: jlong,
) -> jstring {
    if manager_ptr == 0 {
        return string_to_jstring(&mut env, "{}".to_string()).unwrap_or(std::ptr::null_mut());
    }
    
    let manager = unsafe { &*(manager_ptr as *const DataManager) };
    let ingredients_by_category = manager.get_ingredients_by_category();
    
    let mut result = serde_json::Map::new();
    
    for (category, ingredients) in ingredients_by_category {
        let mut category_ingredients = Vec::new();
        for ingredient in ingredients {
            let pantry_item = manager.get_pantry_item(&ingredient.name);
            let is_in_stock = manager.is_in_pantry(&ingredient.name);
            
            let (quantity, quantity_type) = if let Some(item) = pantry_item {
                (item.quantity, Some(item.quantity_type.clone()))
            } else {
                (None, None)
            };
            
            let ingredient_json = serde_json::json!({
                "name": ingredient.name,
                "slug": ingredient.slug,
                "category": ingredient.category,
                "kb": ingredient.kb,
                "tags": ingredient.tags,
                "isInStock": is_in_stock,
                "quantity": quantity,
                "quantityType": quantity_type
            });
            
            category_ingredients.push(ingredient_json);
        }
        result.insert(category, serde_json::Value::Array(category_ingredients));
    }
    
    match serde_json::to_string(&result) {
        Ok(json) => string_to_jstring(&mut env, json).unwrap_or(std::ptr::null_mut()),
        Err(_) => string_to_jstring(&mut env, "{}".to_string()).unwrap_or(std::ptr::null_mut()),
    }
}

// Update pantry status for an ingredient
#[no_mangle]
pub extern "system" fn Java_com_example_pantryman_CookbookEngine_updatePantryStatus(
    mut env: JNIEnv,
    _class: JClass,
    manager_ptr: jlong,
    ingredient_name: JString,
    add_to_pantry: jboolean,
    quantity: jlong,
    quantity_type: JString,
) -> jboolean {
    if manager_ptr == 0 {
        return 0; // false
    }
    
    let ingredient_name_str = match jstring_to_string(&mut env, &ingredient_name) {
        Ok(name) => name,
        Err(_) => return 0,
    };
    
    let quantity_type_str = match jstring_to_string(&mut env, &quantity_type) {
        Ok(qty_type) => qty_type,
        Err(_) => String::new(),
    };
    
    let manager = unsafe { &mut *(manager_ptr as *mut DataManager) };
    
    if add_to_pantry != 0 {
        // Add to pantry
        let qty = if quantity > 0 { Some(quantity as f64) } else { None };
        let qty_type = if quantity_type_str.is_empty() { None } else { Some(quantity_type_str) };
        
        match manager.update_pantry_item(&ingredient_name_str, qty, qty_type) {
            Ok(_) => 1, // true
            Err(_) => 0, // false
        }
    } else {
        // Remove from pantry
        match manager.remove_from_pantry(&ingredient_name_str) {
            Ok(_) => 1, // true
            Err(_) => 0, // false
        }
    }
}

// Create a new ingredient
#[no_mangle]
pub extern "system" fn Java_com_example_pantryman_CookbookEngine_createIngredient(
    mut env: JNIEnv,
    _class: JClass,
    manager_ptr: jlong,
    name: JString,
    category: JString,
    kb_slug: JString,
    tags_json: JString,
) -> jboolean {
    if manager_ptr == 0 {
        return 0;
    }
    
    let name_str = match jstring_to_string(&mut env, &name) {
        Ok(n) => n,
        Err(_) => return 0,
    };
    
    let category_str = match jstring_to_string(&mut env, &category) {
        Ok(c) => c,
        Err(_) => return 0,
    };
    
    let kb_str = match jstring_to_string(&mut env, &kb_slug) {
        Ok(kb) => if kb.is_empty() { None } else { Some(kb) },
        Err(_) => None,
    };
    
    let tags_str = match jstring_to_string(&mut env, &tags_json) {
        Ok(t) => t,
        Err(_) => "[]".to_string(),
    };
    
    let tags: Option<Vec<String>> = serde_json::from_str(&tags_str).ok();
    
    let ingredient = Ingredient {
        name: name_str.clone(),
        slug: name_str.replace(" ", "_").to_lowercase(),
        category: category_str,
        kb: kb_str,
        tags,
        translations: None,
    };
    
    let manager = unsafe { &mut *(manager_ptr as *mut DataManager) };
    
    match manager.create_ingredient(ingredient) {
        Ok(_) => 1, // true
        Err(_) => 0, // false
    }
}

// Update an existing ingredient
#[no_mangle]
pub extern "system" fn Java_com_example_pantryman_CookbookEngine_updateIngredient(
    mut env: JNIEnv,
    _class: JClass,
    manager_ptr: jlong,
    original_name: JString,
    new_name: JString,
    category: JString,
    kb_slug: JString,
    tags_json: JString,
) -> jboolean {
    if manager_ptr == 0 {
        return 0;
    }
    
    let original_name_str = match jstring_to_string(&mut env, &original_name) {
        Ok(n) => n,
        Err(_) => return 0,
    };
    
    let new_name_str = match jstring_to_string(&mut env, &new_name) {
        Ok(n) => n,
        Err(_) => return 0,
    };
    
    let category_str = match jstring_to_string(&mut env, &category) {
        Ok(c) => c,
        Err(_) => return 0,
    };
    
    let kb_str = match jstring_to_string(&mut env, &kb_slug) {
        Ok(kb) => if kb.is_empty() { None } else { Some(kb) },
        Err(_) => None,
    };
    
    let tags_str = match jstring_to_string(&mut env, &tags_json) {
        Ok(t) => t,
        Err(_) => "[]".to_string(),
    };
    
    let tags: Option<Vec<String>> = serde_json::from_str(&tags_str).ok();
    
    let ingredient = Ingredient {
        name: new_name_str.clone(),
        slug: new_name_str.replace(" ", "_").to_lowercase(),
        category: category_str,
        kb: kb_str,
        tags,
        translations: None,
    };
    
    let manager = unsafe { &mut *(manager_ptr as *mut DataManager) };
    
    match manager.update_ingredient(&original_name_str, ingredient) {
        Ok(_) => 1, // true
        Err(_) => 0, // false
    }
}

// Delete an ingredient
#[no_mangle]
pub extern "system" fn Java_com_example_pantryman_CookbookEngine_deleteIngredient(
    mut env: JNIEnv,
    _class: JClass,
    manager_ptr: jlong,
    ingredient_name: JString,
) -> jboolean {
    if manager_ptr == 0 {
        return 0;
    }
    
    let ingredient_name_str = match jstring_to_string(&mut env, &ingredient_name) {
        Ok(name) => name,
        Err(_) => return 0,
    };
    
    let manager = unsafe { &mut *(manager_ptr as *mut DataManager) };
    
    match manager.delete_ingredient(&ingredient_name_str) {
        Ok(_) => 1, // true
        Err(_) => 0, // false
    }
}

// Get all categories
#[no_mangle]
pub extern "system" fn Java_com_example_pantryman_CookbookEngine_getAllCategories(
    mut env: JNIEnv,
    _class: JClass,
    manager_ptr: jlong,
) -> jstring {
    if manager_ptr == 0 {
        return string_to_jstring(&mut env, "[]".to_string()).unwrap_or(std::ptr::null_mut());
    }
    
    let manager = unsafe { &*(manager_ptr as *const DataManager) };
    let categories = manager.get_unique_categories();
    
    match serde_json::to_string(&categories) {
        Ok(json) => string_to_jstring(&mut env, json).unwrap_or(std::ptr::null_mut()),
        Err(_) => string_to_jstring(&mut env, "[]".to_string()).unwrap_or(std::ptr::null_mut()),
    }
}
