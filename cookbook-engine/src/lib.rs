use serde::{Deserialize, Serialize};            // For serialization and deserialization
use std::fs;                                    // For file operations
use std::path::{Path, PathBuf};                 // For handling file paths
use thiserror::Error;                           // For error handling
use std::collections::HashMap;                  // For storing ingredients and recipes

/*
The #[derive(...)] attribute in Rust allows you to automatically implement certain traits for your custom data types without having to write the implementation code manually. In this specific case, four important traits are being derived:

Debug enables the type to be printed with the {:?} format specifier during debugging, allowing you to see the internal state of instances during development or when troubleshooting.

Clone provides a .clone() method that creates a deep copy of the value, allowing you to duplicate instances when needed. This is useful when you need to create independent copies of your data structures.

Serialize and Deserialize are traits from the Serde library (serialization/deserialization framework) that allow the type to be converted to and from various data formats like JSON, YAML, or TOML. This is particularly important for the cookbook project since it stores data in YAML files and needs to read/write these formats.
*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ingredient {
    pub name: String,
    pub category: String,
    pub kb: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeIngredient {
    pub ingredient: String,
    pub quantity: Option<f64>,
    pub quantity_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    #[serde(rename = "Title")]
    pub title: String,
    
    #[serde(rename = "Ingredients")]
    pub ingredients: Vec<RecipeIngredient>,
    
    #[serde(rename = "PrepTime")]
    pub prep_time: Option<u32>,
    
    #[serde(rename = "Downtime")]
    pub downtime: Option<u32>,
    
    #[serde(rename = "Servings")]
    pub servings: Option<u32>,
    
    #[serde(rename = "Tags")]
    pub tags: Option<Vec<String>>,
    
    #[serde(skip)]
    pub instructions: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PantryItem {
    pub ingredient: String,
    pub quantity: Option<f64>,
    #[serde(default)]
    pub quantity_type: String,
    pub last_updated: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pantry {
    pub version: u8,
    pub items: Vec<PantryItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KnowledgeBaseEntry {
    pub slug: String,
    pub title: String,
    pub image: Option<String>,
    #[serde(skip)]
    pub content: String,
}

#[derive(Debug, Error)]
pub enum CookbookError {
    #[error("Failed to read file: {0}")]
    ReadError(String),

    #[error("Failed to parse YAML: {0}")]
    ParseError(String),
    
    #[error("Failed to parse Markdown: {0}")]
    MarkdownError(String),
    
    #[error("Data directory not found: {0}")]
    DataDirError(String),
    
    #[error("Failed to list directory: {0}")]
    ListDirError(String),
    
    #[error("Failed to write file: {0}")]
    WriteError(String),
    
    #[error("Failed to update ingredient: {0}")]
    UpdateError(String),
}

impl Ingredient {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, CookbookError> {
        let content = fs::read_to_string(&path).map_err(|e| CookbookError::ReadError(e.to_string()))?;
        serde_yaml::from_str(&content).map_err(|e| CookbookError::ParseError(e.to_string()))
    }
    
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), CookbookError> {
        let yaml = serde_yaml::to_string(self)
            .map_err(|e| CookbookError::ParseError(format!("Failed to serialize ingredient: {}", e)))?;
        
        fs::write(&path, yaml)
            .map_err(|e| CookbookError::WriteError(format!("Failed to write ingredient file: {}", e)))?;
            
        Ok(())
    }
}

impl Pantry {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, CookbookError> {
        let content = fs::read_to_string(&path).map_err(|e| CookbookError::ReadError(e.to_string()))?;
        serde_yaml::from_str(&content).map_err(|e| CookbookError::ParseError(e.to_string()))
    }
    
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), CookbookError> {
        let yaml = serde_yaml::to_string(self)
            .map_err(|e| CookbookError::ParseError(format!("Failed to serialize pantry: {}", e)))?;
        
        fs::write(&path, yaml)
            .map_err(|e| CookbookError::WriteError(format!("Failed to write pantry file: {}", e)))?;
            
        Ok(())
    }
}

impl Recipe {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, CookbookError> {
        let content = fs::read_to_string(&path).map_err(|e| CookbookError::ReadError(e.to_string()))?;
        
        // Split frontmatter from content by finding the two "---" delimiters
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        
        if parts.len() < 3 {
            return Err(CookbookError::MarkdownError("Invalid markdown format: missing frontmatter delimiters".to_string()));
        }
        
        // Parse the YAML frontmatter (parts[1] is between the first and second ---)
        let mut recipe: Recipe = serde_yaml::from_str(parts[1].trim())
            .map_err(|e| CookbookError::ParseError(format!("Failed to parse YAML frontmatter: {}", e)))?;
        
        // Store the instructions (parts[2] is after the second ---)
        recipe.instructions = parts[2].trim().to_string();
        
        Ok(recipe)
    }
    
    pub fn total_time(&self) -> u32 {
        self.prep_time.unwrap_or(0) + self.downtime.unwrap_or(0)
    }
}

impl KnowledgeBaseEntry {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, CookbookError> {
        let content = fs::read_to_string(&path).map_err(|e| CookbookError::ReadError(e.to_string()))?;
        
        // Split frontmatter from content by finding the two "---" delimiters
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        
        if parts.len() < 3 {
            return Err(CookbookError::MarkdownError("Invalid markdown format: missing frontmatter delimiters".to_string()));
        }
        
        // Parse the YAML frontmatter (parts[1] is between the first and second ---)
        let mut kb_entry: KnowledgeBaseEntry = serde_yaml::from_str(parts[1].trim())
            .map_err(|e| CookbookError::ParseError(format!("Failed to parse YAML frontmatter: {}", e)))?;
        
        // Store the content (parts[2] is after the second ---)
        kb_entry.content = parts[2].trim().to_string();
        
        Ok(kb_entry)
    }
}

#[derive(Debug)]
pub struct DataManager {
    data_dir: PathBuf,
    ingredients: HashMap<String, Ingredient>,
    recipes: Vec<Recipe>,
    pantry: Option<Pantry>,
    kb_entries: HashMap<String, KnowledgeBaseEntry>,
}

impl DataManager {
    pub fn new<P: AsRef<Path>>(data_dir: P) -> Result<Self, CookbookError> {
        let data_dir = data_dir.as_ref().to_path_buf();
        if !data_dir.exists() {
            return Err(CookbookError::DataDirError(format!("Directory not found: {:?}", data_dir)));
        }
        
        let mut manager = DataManager {
            data_dir,
            ingredients: HashMap::new(),
            recipes: Vec::new(),
            pantry: None,
            kb_entries: HashMap::new(),
        };
        
        // Load all data
        manager.load_data()?;
        
        Ok(manager)
    }
    
    pub fn get_data_dir(&self) -> &Path {
        &self.data_dir
    }
    
    pub fn load_data(&mut self) -> Result<(), CookbookError> {
        self.load_ingredients()?;
        self.load_recipes()?;
        self.load_pantry()?;
        self.load_kb_entries()?;
        
        Ok(())
    }
    
    fn load_ingredients(&mut self) -> Result<(), CookbookError> {
        let ingredients_dir = self.data_dir.join("ingredients");
        if !ingredients_dir.exists() {
            return Err(CookbookError::DataDirError(format!("Ingredients directory not found: {:?}", ingredients_dir)));
        }
        
        let entries = fs::read_dir(&ingredients_dir)
            .map_err(|e| CookbookError::ListDirError(e.to_string()))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| CookbookError::ListDirError(e.to_string()))?;
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                let ingredient = Ingredient::from_file(&path)?;
                self.ingredients.insert(ingredient.name.clone(), ingredient);
            }
        }
        
        Ok(())
    }
    
    fn load_recipes(&mut self) -> Result<(), CookbookError> {
        let recipes_dir = self.data_dir.join("recipes");
        if !recipes_dir.exists() {
            return Err(CookbookError::DataDirError(format!("Recipes directory not found: {:?}", recipes_dir)));
        }
        
        let entries = fs::read_dir(&recipes_dir)
            .map_err(|e| CookbookError::ListDirError(e.to_string()))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| CookbookError::ListDirError(e.to_string()))?;
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                match Recipe::from_file(&path) {
                    Ok(recipe) => self.recipes.push(recipe),
                    Err(e) => eprintln!("Failed to load recipe {:?}: {}", path, e),
                }
            }
        }
        
        Ok(())
    }
    
    fn load_pantry(&mut self) -> Result<(), CookbookError> {
        let pantry_path = self.data_dir.join("pantry.yaml");
        if pantry_path.exists() {
            self.pantry = Some(Pantry::from_file(&pantry_path)?);
        }
        
        Ok(())
    }
    
    fn load_kb_entries(&mut self) -> Result<(), CookbookError> {
        let kb_dir = self.data_dir.join("kb");
        if !kb_dir.exists() {
            return Err(CookbookError::DataDirError(format!("Knowledge Base directory not found: {:?}", kb_dir)));
        }
        
        let entries = fs::read_dir(&kb_dir)
            .map_err(|e| CookbookError::ListDirError(e.to_string()))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| CookbookError::ListDirError(e.to_string()))?;
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                match KnowledgeBaseEntry::from_file(&path) {
                    Ok(kb_entry) => {
                        self.kb_entries.insert(kb_entry.slug.clone(), kb_entry);
                    },
                    Err(e) => eprintln!("Failed to load KB entry {:?}: {}", path, e),
                }
            }
        }
        
        Ok(())
    }
    
    pub fn get_all_ingredients(&self) -> Vec<&Ingredient> {
        self.ingredients.values().collect()
    }
    
    pub fn get_ingredient(&self, name: &str) -> Option<&Ingredient> {
        self.ingredients.get(name)
    }
    
    pub fn is_in_pantry(&self, ingredient_name: &str) -> bool {
        if let Some(pantry) = &self.pantry {
            pantry.items.iter().any(|item| item.ingredient == ingredient_name)
        } else {
            false
        }
    }
    
    pub fn get_pantry_item(&self, ingredient_name: &str) -> Option<&PantryItem> {
        if let Some(pantry) = &self.pantry {
            pantry.items.iter().find(|item| item.ingredient == ingredient_name)
        } else {
            None
        }
    }
    
    pub fn get_all_recipes(&self) -> &[Recipe] {
        &self.recipes
    }

    pub fn get_recipe(&self, title: &str) -> Option<&Recipe> {
        self.recipes.iter().find(|recipe| recipe.title == title)
    }
    
    pub fn get_pantry(&self) -> Option<&Pantry> {
        self.pantry.as_ref()
    }
    
    pub fn get_kb_entry(&self, slug: &str) -> Option<&KnowledgeBaseEntry> {
        self.kb_entries.get(slug)
    }
    
    pub fn get_all_kb_entries(&self) -> Vec<&KnowledgeBaseEntry> {
        self.kb_entries.values().collect()
    }
    
    /// Returns all recipes that use the specified ingredient
    pub fn get_recipes_with_ingredient(&self, ingredient_name: &str) -> Vec<&Recipe> {
        self.recipes.iter()
            .filter(|recipe| {
                recipe.ingredients.iter().any(|ing| ing.ingredient == ingredient_name)
            })
            .collect()
    }

    /// Filters ingredients based on search text, categories, and stock status
    pub fn filter_ingredients(&self, 
                             search_text: &str, 
                             categories: &[String], 
                             in_stock_only: bool) -> Vec<&Ingredient> {
        self.get_all_ingredients()
            .into_iter()
            .filter(|ingredient| {
                // Match search text
                let matches_search = search_text.is_empty() || 
                    ingredient.name.to_lowercase().contains(&search_text.to_lowercase());
                    
                // Match category filter
                let matches_category = categories.is_empty() || 
                    categories.contains(&ingredient.category);
                    
                // Match stock status
                let matches_stock = !in_stock_only || self.is_in_pantry(&ingredient.name);
                
                matches_search && matches_category && matches_stock
            })
            .collect()
    }

    /// Returns all available ingredient categories
    pub fn get_all_ingredient_categories(&self) -> Vec<String> {
        let mut categories = self.get_all_ingredients()
            .iter()
            .map(|ingredient| ingredient.category.clone())
            .collect::<std::collections::HashSet<String>>()
            .into_iter()
            .collect::<Vec<String>>();
        
        categories.sort();
        categories
    }

    /// Groups ingredients by their category
    pub fn get_ingredients_by_category(&self) -> HashMap<String, Vec<&Ingredient>> {
        let mut result = HashMap::new();
        
        for ingredient in self.get_all_ingredients() {
            result
                .entry(ingredient.category.clone())
                .or_insert_with(Vec::new)
                .push(ingredient);
        }
        
        // Sort ingredients within each category
        for (_category, ingredients) in &mut result {
            ingredients.sort_by(|a, b| a.name.cmp(&b.name));
        }
        
        result
    }
    
    /// Groups pantry items by ingredient category
    pub fn get_pantry_items_by_category(&self) -> HashMap<String, Vec<(&Ingredient, Option<&PantryItem>)>> {
        let mut result = HashMap::new();
        
        for ingredient in self.get_all_ingredients() {
            // Find the matching pantry item, if any
            let pantry_item = self.get_pantry_item(&ingredient.name);
            
            result
                .entry(ingredient.category.clone())
                .or_insert_with(Vec::new)
                .push((ingredient, pantry_item));
        }
        
        // Sort ingredients within each category
        for (_category, items) in &mut result {
            items.sort_by(|a, b| a.0.name.cmp(&b.0.name));
        }
        
        result
    }
    
    /// Search for recipes matching a text query
    pub fn search_recipes(&self, query: &str) -> Vec<&Recipe> {
        let query_lower = query.to_lowercase();
        
        self.recipes.iter()
            .filter(|recipe| {
                // Search in title
                recipe.title.to_lowercase().contains(&query_lower) ||
                // Search in ingredients
                recipe.ingredients.iter().any(|ing| 
                    ing.ingredient.to_lowercase().contains(&query_lower)
                ) ||
                // Search in instructions
                recipe.instructions.to_lowercase().contains(&query_lower) ||
                // Search in tags
                recipe.tags.as_ref().map_or(false, |tags| 
                    tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
                )
            })
            .collect()
    }
    
    /// Returns a map of ingredient names and the recipes that use them
    pub fn get_ingredient_usage(&self) -> HashMap<String, Vec<&Recipe>> {
        let mut result = HashMap::new();
        
        // Collect all unique ingredient names
        for ingredient in self.get_all_ingredients() {
            result.entry(ingredient.name.clone()).or_insert_with(Vec::new);
        }
        
        // Match recipes to ingredients
        for recipe in &self.recipes {
            for ing in &recipe.ingredients {
                if let Some(recipes) = result.get_mut(&ing.ingredient) {
                    recipes.push(recipe);
                }
            }
        }
        
        result
    }
    
    /// Get KB entry for a specific ingredient
    pub fn get_kb_entry_for_ingredient(&self, ingredient_name: &str) -> Option<&KnowledgeBaseEntry> {
        if let Some(ingredient) = self.get_ingredient(ingredient_name) {
            if let Some(kb_slug) = &ingredient.kb {
                return self.get_kb_entry(kb_slug);
            }
        }
        None
    }
    
    /// Updates an ingredient in the pantry with new quantity and quantity_type values
    pub fn update_pantry_item(&mut self, 
                             ingredient_name: &str,
                             quantity: Option<f64>,
                             quantity_type: Option<String>) -> Result<bool, CookbookError> {
        // Make sure we have a pantry loaded
        if self.pantry.is_none() {
            return Err(CookbookError::UpdateError(format!("No pantry loaded")));
        }
        
        let pantry = self.pantry.as_mut().unwrap();
        
        // Check if the ingredient exists in the ingredients
        if !self.ingredients.contains_key(ingredient_name) {
            return Err(CookbookError::UpdateError(
                format!("Ingredient '{}' does not exist", ingredient_name)
            ));
        }
        
        // Get the current date for the last_updated field
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        
        // Find the pantry item if it exists
        let pantry_item_index = pantry.items.iter().position(|item| item.ingredient == ingredient_name);
        
        if let Some(index) = pantry_item_index {
            // Update the existing pantry item
            pantry.items[index].quantity = quantity;
            pantry.items[index].quantity_type = quantity_type.unwrap_or_default();
            pantry.items[index].last_updated = today;
        } else {
            // Create a new pantry item
            let new_item = PantryItem {
                ingredient: ingredient_name.to_string(),
                quantity,
                quantity_type: quantity_type.unwrap_or_default(),
                last_updated: today,
            };
            
            pantry.items.push(new_item);
        }
        
        // Save the updated pantry to file
        let pantry_path = self.data_dir.join("pantry.yaml");
        pantry.to_file(pantry_path)?;
        
        Ok(true)
    }
    
    /// Updates an ingredient's properties (name, category, kb, tags)
    pub fn update_ingredient(&mut self, 
                            original_name: &str,
                            new_ingredient: Ingredient) -> Result<bool, CookbookError> {
        // Check if the original ingredient exists
        if !self.ingredients.contains_key(original_name) {
            return Err(CookbookError::UpdateError(
                format!("Ingredient '{}' does not exist", original_name)
            ));
        }
        
        // Check if the new name conflicts with an existing ingredient (if name is changing)
        if original_name != new_ingredient.name && self.ingredients.contains_key(&new_ingredient.name) {
            return Err(CookbookError::UpdateError(
                format!("Cannot rename: ingredient '{}' already exists", new_ingredient.name)
            ));
        }
        
        let ingredients_dir = self.data_dir.join("ingredients");
        let old_path = ingredients_dir.join(format!("{}.yaml", original_name.replace(" ", "_")));
        let new_path = ingredients_dir.join(format!("{}.yaml", new_ingredient.name.replace(" ", "_")));
        
        // Handle name changes
        if original_name != new_ingredient.name {
            // Update any pantry reference
            if let Some(pantry) = self.pantry.as_mut() {
                for item in &mut pantry.items {
                    if item.ingredient == original_name {
                        item.ingredient = new_ingredient.name.clone();
                    }
                }
                
                // Save the updated pantry
                let pantry_path = self.data_dir.join("pantry.yaml");
                pantry.to_file(pantry_path)?;
            }
            
            // Remove the old ingredient from our HashMap
            self.ingredients.remove(original_name);
        }
        
        // Update/add the ingredient in our HashMap
        self.ingredients.insert(new_ingredient.name.clone(), new_ingredient.clone());
        
        // Write the ingredient to file
        new_ingredient.to_file(&new_path)?;
        
        // If the name changed, remove the old file
        if original_name != new_ingredient.name && old_path.exists() {
            fs::remove_file(old_path)
                .map_err(|e| CookbookError::WriteError(format!("Failed to remove old ingredient file: {}", e)))?;
        }
        
        Ok(true)
    }
    
    /// Updates an ingredient with potential changes to both ingredient properties and pantry values
    pub fn update_ingredient_with_pantry(&mut self,
                                       original_name: &str,
                                       new_ingredient: Ingredient,
                                       quantity: Option<f64>,
                                       quantity_type: Option<String>) -> Result<bool, CookbookError> {
        // First update the ingredient itself
        self.update_ingredient(original_name, new_ingredient.clone())?;
        
        // Then update the pantry if necessary
        // We'll always pass the quantity_type (empty string if None)
        if quantity.is_some() || quantity_type.is_some() {
            self.update_pantry_item(&new_ingredient.name, quantity, quantity_type)?;
        }
        
        Ok(true)
    }
}
