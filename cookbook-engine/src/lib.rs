use serde::{Deserialize, Serialize};            // For serialization and deserialization
use std::fs;                                    // For file operations
use std::path::{Path, PathBuf};                 // For handling file paths
use thiserror::Error;                           // For error handling
use std::collections::HashMap;                  // For storing ingredients and recipes
use log::{debug, info, warn, error};            // For logging

/*
The #[derive(...)] attribute in Rust allows you to automatically implement certain traits for your custom data types without having to write the implementation code manually. In this specific case, four important traits are being derived:

Debug enables the type to be printed with the {:?} format specifier during debugging, allowing you to see the internal state of instances during development or when troubleshooting.

Clone provides a .clone() method that creates a deep copy of the value, allowing you to duplicate instances when needed. This is useful when you need to create independent copies of your data structures.

Serialize and Deserialize are traits from the Serde library (serialization/deserialization framework) that allow the type to be converted to and from various data formats like JSON, YAML, or TOML. This is particularly important for the cookbook project since it stores data in YAML files and needs to read/write these formats.
*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ingredient {
    pub name: String,
    pub slug: String,
    pub category: String,
    pub kb: Option<String>,
    pub tags: Option<Vec<String>>,
    pub translations: Option<HashMap<String, TranslationForms>>, // language code -> forms
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationForms {
    pub one: String,
    pub other: String,
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
    
    #[serde(rename = "image")]
    pub image: Option<String>,
    
    #[serde(skip)]
    pub instructions: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PantryItem {
    pub ingredient: String,     //#TODO Should it perhaps be an Ingredient?
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

// Implementing method for Ingredient
impl Ingredient {
    
    // Reads an ingredient from a YAML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, CookbookError> {
        let content = std::fs::read_to_string(&path).map_err(|e| CookbookError::ReadError(e.to_string()))?;
        serde_yaml::from_str(&content).map_err(|e| CookbookError::ParseError(e.to_string()))
    }
    // Writes an ingredient to a YAML file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), CookbookError> {
        let yaml = serde_yaml::to_string(self)
            .map_err(|e| CookbookError::ParseError(format!("Failed to serialize ingredient: {}", e)))?;
        std::fs::write(&path, yaml)
            .map_err(|e| CookbookError::WriteError(format!("Failed to write ingredient file: {}", e)))?;
        println!("Successfully wrote to {}!", path.as_ref().display());
        Ok(())
    }
}

// Implementing method for Pantry
impl Pantry {
    // Reads a pantry from a YAML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, CookbookError> {
        let content = std::fs::read_to_string(&path).map_err(|e| CookbookError::ReadError(e.to_string()))?;
        serde_yaml::from_str(&content).map_err(|e| CookbookError::ParseError(e.to_string()))
    }
    // Writes a pantry to a YAML file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), CookbookError> {
        let yaml = serde_yaml::to_string(self)
            .map_err(|e| CookbookError::ParseError(format!("Failed to serialize pantry: {}", e)))?;
        std::fs::write(&path, yaml)
            .map_err(|e| CookbookError::WriteError(format!("Failed to write pantry file: {}", e)))?;
        println!("Successfully wrote to {}!", path.as_ref().display());
        Ok(())
    }
}

// Implementing method for Recipe
impl Recipe {
    /// Reads a recipe from a Markdown file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, CookbookError> {
        let content = fs::read_to_string(&path).map_err(|e| CookbookError::ReadError(e.to_string()))?; // Read the file content
        
        // Split frontmatter from content by finding the two "---" delimiters
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        
        // Check if we have at least three parts: before, frontmatter, and after
        if parts.len() < 3 {
            return Err(CookbookError::MarkdownError("Invalid markdown format: missing frontmatter delimiters".to_string()));
        }
        
        // Parse the YAML frontmatter (parts[1] is between the first and second ---)
        let mut recipe: Recipe = serde_yaml::from_str(parts[1].trim())
            .map_err(|e| CookbookError::ParseError(format!("Failed to parse YAML frontmatter: {}", e)))?;
        
        // Store the instructions (parts[2] is after the second ---)
        recipe.instructions = parts[2].trim().to_string();
        
        Ok(recipe)  // Return the parsed recipe
    }
    
    /// Writes a recipe to a Markdown file with YAML frontmatter
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), CookbookError> {
        // Create a copy of the recipe without instructions for YAML serialization
        // This ensures we don't include the instructions in the YAML frontmatter
        let recipe_for_yaml = Recipe {
            title: self.title.clone(),
            ingredients: self.ingredients.clone(),
            prep_time: self.prep_time,
            downtime: self.downtime,
            servings: self.servings,
            tags: self.tags.clone(),
            image: self.image.clone(),
            instructions: String::new(), // Empty string since it's excluded via #[serde(skip)]
        };
        
        // Serialize to YAML
        let yaml = serde_yaml::to_string(&recipe_for_yaml)
            .map_err(|e| CookbookError::ParseError(format!("Failed to serialize recipe: {}", e)))?;
        
        // Build the full content with frontmatter delimiters and instructions
        let content = format!("---\n{}---\n{}", yaml, self.instructions);
        
        // Write to file
        fs::write(&path, content)
            .map_err(|e| CookbookError::WriteError(format!("Failed to write recipe file: {}", e)))?;
        println!("Successfully wrote to {}!", path.as_ref().display());
        Ok(())
    }
    
    /// Returns the total time required for the recipe (prep time + downtime)
    pub fn total_time(&self) -> u32 {
        self.prep_time.unwrap_or(0) + self.downtime.unwrap_or(0)
    }
    
    /// Checks if all ingredients for this recipe are in stock (in the pantry)
    /// Returns true if all ingredients are available, false otherwise
    pub fn all_ingredients_in_stock(&self, data_manager: &DataManager) -> bool {
        self.ingredients.iter().all(|ingredient| data_manager.is_in_pantry(&ingredient.ingredient))
    }
}

// Implementing method for KnowledgeBaseEntry
impl KnowledgeBaseEntry {
    /// Reads a knowledge base entry from a Markdown file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, CookbookError> {
        let content = fs::read_to_string(&path).map_err(|e| CookbookError::ReadError(e.to_string()))?; // Read the file content
        
        // Split frontmatter from content by finding the two "---" delimiters
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        
        // Check if we have at least three parts: before, frontmatter, and after
        if parts.len() < 3 {
            return Err(CookbookError::MarkdownError("Invalid markdown format: missing frontmatter delimiters".to_string()));
        }
        
        // Parse the YAML frontmatter (parts[1] is between the first and second ---)
        let mut kb_entry: KnowledgeBaseEntry = serde_yaml::from_str(parts[1].trim())
            .map_err(|e| CookbookError::ParseError(format!("Failed to parse YAML frontmatter: {}", e)))?;
        
        // Store the content (parts[2] is after the second ---)
        kb_entry.content = parts[2].trim().to_string();
        
        Ok(kb_entry) // Return the parsed knowledge base entry
    }
}

// Main struct for managing the cookbook data
// It contains methods for loading, updating, and retrieving ingredients, recipes, and pantry items
// It also provides methods for filtering and searching through the data
#[derive(Debug)]
pub struct DataManager {
    data_dir: PathBuf,
    ingredients: HashMap<String, Ingredient>,
    recipes: Vec<Recipe>,
    pantry: Option<Pantry>,
    kb_entries: HashMap<String, KnowledgeBaseEntry>,
}

// Implementing methods for DataManager
/// The DataManager is responsible for loading, storing, retrieving, and updating cookbook data.
///
/// # Data Components
///
/// This manager handles several types of cookbook data:
/// - Ingredients: Basic food components with properties like name, category, etc.
/// - Recipes: Instructions for preparing dishes using combinations of ingredients
/// - Pantry: User's inventory of available ingredients
/// - Knowledge Base: Educational entries related to cooking techniques and ingredients
///
/// # File Structure
///
/// The DataManager expects a specific directory structure:
/// - `{data_dir}/ingredients/*.yaml` - YAML files for each ingredient
/// - `{data_dir}/recipes/*.md` - Markdown files for each recipe
/// - `{data_dir}/pantry.yaml` - YAML file containing pantry inventory
/// - `{data_dir}/kb/*.md` - Markdown files for knowledge base entries
///
/// # Main Functionality
///
/// The DataManager provides methods for:
/// - Loading data from files
/// - Retrieving specific items or collections
/// - Filtering and searching data
/// - Updating ingredients and pantry items
/// - Cross-referencing between different data types (e.g., finding recipes that use specific ingredients)
///
/// # Error Handling
///
/// Most methods return Result types that can contain CookbookError variants for handling
/// issues such as missing directories, parsing failures, or update errors.
impl DataManager {
    /// Creates a new DataManager instance, loading data from the specified directory
    /// Returns an error if the directory does not exist or if any data loading fails
    /// The data_dir parameter is the path to the directory containing the cookbook data files
    pub fn new<P: AsRef<Path>>(data_dir: P) -> Result<Self, CookbookError> {
        // Initialize Android logging if we're on Android
        #[cfg(target_os = "android")]
        {
            let _ = android_logger::init_once(
                android_logger::Config::default()
                    .with_max_level(log::LevelFilter::Debug)
                    .with_tag("CookbookEngine")
            );
        }
        
        let data_dir = data_dir.as_ref().to_path_buf(); // Convert data_dir to PathBuf
        info!("DataManager::new called with data_dir: {:?}", data_dir);
        
        // Check if the directory exists
        // If it doesn't exist, return an error
        if !data_dir.exists() {
            error!("Data directory does not exist: {:?}", data_dir);
            return Err(CookbookError::DataDirError(format!("Directory not found: {:?}", data_dir)));
        }
        
        info!("Data directory exists, creating DataManager...");
        
        // Create a new DataManager instance
        // Initialize the ingredients HashMap, recipes Vec, pantry Option, and kb_entries HashMap
        let mut manager = DataManager {
            data_dir,
            ingredients: HashMap::new(),
            recipes: Vec::new(),
            pantry: None,
            kb_entries: HashMap::new(),
        };
        
        info!("Loading data...");
        
        // Load all data
        manager.load_data()?;
        
        info!("DataManager created successfully");
        Ok(manager) // Return the DataManager instance
    }
    
    /// Returns the data directory path
    pub fn get_data_dir(&self) -> &Path {
        &self.data_dir
    }
    
    /// Loads all data from the specified directory
    pub fn load_data(&mut self) -> Result<(), CookbookError> {
        self.load_ingredients()?;
        self.load_recipes()?;
        self.load_pantry()?;
        self.load_kb_entries()?;
        
        Ok(())  // Return Ok if all data loading is successful
    }
    
    /// Loads ingredients from the ingredients directory
    /// Returns an error if the directory does not exist or if any ingredient file fails to load
    /// The ingredients directory should contain YAML files for each ingredient
    /// The ingredient files should be named with the format "ingredient_name.yaml"
    /// The ingredient_name should be the same as the name field in the Ingredient struct
    fn load_ingredients(&mut self) -> Result<(), CookbookError> {
        let ingredients_dir = self.data_dir.join("ingredients");    // Path to the ingredients directory
        // Check if the ingredients directory exists
        if !ingredients_dir.exists() {
            return Err(CookbookError::DataDirError(format!("Ingredients directory not found: {:?}", ingredients_dir))); 
        }
        
        // Read the contents of the ingredients directory
        let entries = fs::read_dir(&ingredients_dir)
            .map_err(|e| CookbookError::ListDirError(e.to_string()))?;
        
        // Iterate through each entry in the directory
        for entry in entries {
            // Get the path of the entry
            let entry = entry.map_err(|e| CookbookError::ListDirError(e.to_string()))?;
            let path = entry.path();
            
            // Check if the entry is a file and has a .yaml extension
            // If it is, load the ingredient from the file
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                let ingredient = Ingredient::from_file(&path)?;
                self.ingredients.insert(ingredient.name.clone(), ingredient);
            }
        }
        
        Ok(())  // Return Ok if all ingredients are loaded successfully
    }
    
    /// Loads recipes from the recipes directory
    /// Returns an error if the directory does not exist or if any recipe file fails to load
    /// The recipes directory should contain Markdown files for each recipe
    fn load_recipes(&mut self) -> Result<(), CookbookError> {
        let recipes_dir = self.data_dir.join("recipes"); // Path to the recipes directory
        // Check if the recipes directory exists
        // If it doesn't exist, return an error
        if !recipes_dir.exists() {
            return Err(CookbookError::DataDirError(format!("Recipes directory not found: {:?}", recipes_dir)));
        }
        
        // Use a HashMap to track recipes by title (case-insensitive)
        // This helps us avoid duplicate recipes with different capitalizations
        use std::collections::HashMap;
        let mut recipes_map: HashMap<String, Recipe> = HashMap::new();
        
        // Read the contents of the recipes directory
        let entries = fs::read_dir(&recipes_dir)
            .map_err(|e| CookbookError::ListDirError(e.to_string()))?;
        
        // Iterate through each entry in the directory
        for entry in entries {
            // Get the path of the entry
            let entry = entry.map_err(|e| CookbookError::ListDirError(e.to_string()))?;
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") { // Check if the entry is a file and has a .md extension
                match Recipe::from_file(&path) {                                                    // Load the recipe from the file
                    Ok(recipe) => {
                        // Use lowercase title as key to avoid case-sensitive duplicates
                        let title_key = recipe.title.to_lowercase();
                        recipes_map.insert(title_key, recipe);
                    },
                    Err(e) => eprintln!("Failed to load recipe {:?}: {}", path, e), // If failed, print an error message
                }
            }
        }
        
        // Convert the HashMap values into our recipes vector
        self.recipes = recipes_map.into_values().collect();

        Ok(()) // Return Ok if all recipes are loaded successfully
    }
    
    /// Loads the pantry from the pantry.yaml file
    /// Returns an error if the file does not exist or if it fails to load
    /// The pantry.yaml file should contain the pantry data in YAML format
    /// The pantry data should include the version and a list of pantry items
    /// Each pantry item should include the ingredient name, quantity, quantity type, and last updated date
    /// The pantry items should be stored in the items field of the Pantry struct
    fn load_pantry(&mut self) -> Result<(), CookbookError> {
        let pantry_path = self.data_dir.join("pantry.yaml");    // Path to the pantry.yaml file
        info!("Attempting to load pantry from: {:?}", pantry_path);
        
        // Check if the pantry.yaml file exists
        if pantry_path.exists() {
            info!("pantry.yaml file exists, attempting to load...");
            match Pantry::from_file(&pantry_path) {
                Ok(pantry) => {
                    info!("Successfully loaded pantry with {} items", pantry.items.len());
                    for item in &pantry.items {
                        info!("Pantry item: {} (qty: {:?} {})", item.ingredient, item.quantity, item.quantity_type);
                    }
                    self.pantry = Some(pantry);
                    info!("Pantry successfully stored in DataManager");
                }
                Err(e) => {
                    error!("Failed to load pantry from file: {:?}", e);
                    return Err(e);
                }
            }
        } else {
            warn!("pantry.yaml file does not exist at: {:?}", pantry_path);
            info!("Creating empty pantry since file doesn't exist");
            // Create an empty pantry if the file doesn't exist
            self.pantry = Some(Pantry {
                version: 1,
                items: Vec::new(),
            });
        }
        
        Ok(())  // Return Ok if the pantry is loaded successfully
    }
    
    /// Loads knowledge base entries from the kb directory
    /// Returns an error if the directory does not exist or if any entry file fails to load
    /// The kb directory should contain Markdown files for each knowledge base entry
    /// Each entry file should include the frontmatter with the slug, title, image, and content
    /// The slug should be a unique identifier for the entry
    /// The title should be the title of the entry
    /// The image should be the path to the image file (if any)
    /// The content should be the main content of the entry
    /// The knowledge base entries should be stored in the kb_entries HashMap
    /// The keys of the HashMap should be the slugs of the entries
    /// The values of the HashMap should be the KnowledgeBaseEntry struct
    /// The KnowledgeBaseEntry struct should include the slug, title, image, and content fields
    fn load_kb_entries(&mut self) -> Result<(), CookbookError> {
        // Path to the canonical KB directory in the engine crate
        let kb_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/kb");
        if !kb_dir.exists() {
            // KB directory doesn't exist - this is OK for applications that don't need KB functionality
            // (like the pantryman app which only manages pantry/ingredients)
            eprintln!("Warning: Knowledge Base directory not found: {:?} - continuing without KB entries", kb_dir);
            return Ok(());
        }

        // Read the contents of the kb directory
        let entries = std::fs::read_dir(&kb_dir)
            .map_err(|e| CookbookError::ListDirError(e.to_string()))?;

        // Iterate through each entry in the directory
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
    
    /// Returns all ingredients as a vector of references
    pub fn get_all_ingredients(&self) -> Vec<&Ingredient> {
        self.ingredients.values().collect()
    }
    
    /// Returns a specific ingredient from the DataManager's collection of ingredients. 
    pub fn get_ingredient(&self, name: &str) -> Option<&Ingredient> {
        self.ingredients.get(name)
    }
    
    /// Checks whether a specific ingredient exists in the user's pantry
    /// Returns true if the ingredient is found in the pantry, false otherwise
    /// The ingredient_name parameter is the name of the ingredient to check
    /// The ingredient_name should match the name field in the Ingredient struct
    pub fn is_in_pantry(&self, ingredient_name: &str) -> bool {
        debug!("Checking if '{}' is in pantry", ingredient_name);
        if let Some(pantry) = &self.pantry {
            debug!("Pantry exists with {} items", pantry.items.len());
            let result = pantry.items.iter().any(|item| item.ingredient == ingredient_name);
            debug!("'{}' in pantry: {}", ingredient_name, result);
            result
        } else {
            warn!("No pantry loaded when checking for ingredient '{}'", ingredient_name);
            false
        }
    }
    
    /// Returns a specific pantry item from the user's pantry
    /// Returns an Option containing a reference to the PantryItem if found, or None if not found
    /// The ingredient_name parameter is the name of the ingredient to check
    pub fn get_pantry_item(&self, ingredient_name: &str) -> Option<&PantryItem> {
        debug!("Getting pantry item for '{}'", ingredient_name);
        if let Some(pantry) = &self.pantry {
            debug!("Searching through {} pantry items", pantry.items.len());
            let result = pantry.items.iter().find(|item| item.ingredient == ingredient_name);
            if let Some(item) = result {
                debug!("Found pantry item: {} (qty: {:?} {})", item.ingredient, item.quantity, item.quantity_type);
            } else {
                debug!("No pantry item found for '{}'", ingredient_name);
            }
            result
        } else {
            warn!("No pantry loaded when getting pantry item for '{}'", ingredient_name);
            None
        }
    }
    
    /// Returns all recipes as a slice of references
    /// The recipes are stored in the recipes field of the DataManager struct
    /// The recipes field is a vector of Recipe structs
    pub fn get_all_recipes(&self) -> &[Recipe] {
        &self.recipes
    }

    /// Returns a specific recipe from the DataManager's collection of recipes
    /// Returns an Option containing a reference to the Recipe if found, or None if not found
    /// The title parameter is the title of the recipe to retrieve
    /// The title should match the Title field in the Recipe struct
    pub fn get_recipe(&self, title: &str) -> Option<&Recipe> {
        self.recipes.iter().find(|recipe| recipe.title == title)
    }
    
    /// Returns the pantry as an Option containing a reference to the Pantry struct
    /// The pantry field is an Option that contains the user's pantry data
    /// The pantry data is loaded from the pantry.yaml file
    pub fn get_pantry(&self) -> Option<&Pantry> {
        self.pantry.as_ref()
    }
    
    pub fn get_kb_entry(&self, slug: &str) -> Option<&KnowledgeBaseEntry> {
        self.kb_entries.get(slug)
    }
    
    pub fn get_all_kb_entries(&self) -> Vec<&KnowledgeBaseEntry> {
        self.kb_entries.values().collect()
    }

    // Add this method to the DataManager impl section
    /// Returns all ingredients that reference a specific KB entry by slug
    pub fn get_ingredients_with_kb_reference(&self, kb_slug: &str) -> Vec<&Ingredient> {
        self.ingredients
            .values()
            .filter(|ingredient| {
                // Check if the ingredient has a KB reference that matches the slug
                ingredient.kb.as_ref().map_or(false, |slug| slug == kb_slug)
            })
            .collect()
    }
    
    /// Returns all recipes that use the specified ingredient
    pub fn get_recipes_with_ingredient(&self, ingredient_name: &str) -> Vec<&Recipe> {
        let mut results: Vec<&Recipe> = self.recipes.iter()
            .filter(|recipe| {
                recipe.ingredients.iter().any(|ing| ing.ingredient == ingredient_name)
            })
            .collect();
            
        // Sort recipes alphabetically by title
        results.sort_by(|a, b| a.title.cmp(&b.title));
        
        results
    }

    /// Filters ingredients based on search text, categories, and stock status, using translations
    pub fn filter_ingredients(
        &self,
        search_text: &str,
        categories: &[String],
        in_stock_only: bool,
        lang: &str,
    ) -> Vec<&Ingredient> {
        let search_lower = search_text.to_lowercase();
        self.get_all_ingredients()
            .into_iter()
            .filter(|ingredient| {
                // Match search text against slug, name, and translations
                let mut matches_search = search_text.is_empty()
                    || ingredient.slug.to_lowercase().contains(&search_lower)
                    || ingredient.name.to_lowercase().contains(&search_lower);
                if let Some(translations) = &ingredient.translations {
                    if let Some(forms) = translations.get(lang) {
                        matches_search = matches_search
                            || forms.one.to_lowercase().contains(&search_lower)
                            || forms.other.to_lowercase().contains(&search_lower);
                    }
                    if let Some(forms) = translations.get("en") {
                        matches_search = matches_search
                            || forms.one.to_lowercase().contains(&search_lower)
                            || forms.other.to_lowercase().contains(&search_lower);
                    }
                }
                // Match category filter
                let matches_category = categories.is_empty() || categories.contains(&ingredient.category);
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
        
        let mut results: Vec<&Recipe> = self.recipes.iter()
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
            .collect();
            
        // Sort recipes alphabetically by title
        results.sort_by(|a, b| a.title.cmp(&b.title));
        
        results
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
        
        // Sort recipes alphabetically for each ingredient
        for recipes in result.values_mut() {
            recipes.sort_by(|a, b| a.title.cmp(&b.title));
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
        let pantry = self.pantry.as_mut().expect("Pantry should be loaded if not None");
        // Check if the ingredient exists in the ingredients
        if !self.ingredients.contains_key(ingredient_name) {
            return Err(CookbookError::UpdateError(
                format!("update_pantry_item: Ingredient '{}' does not exist", ingredient_name)
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
                format!("update_ingredient: Ingredient '{}' does not exist", original_name)
            ));
        }
        
        // Check if the new name conflicts with an existing ingredient (if name is changing)
        if original_name != new_ingredient.name && self.ingredients.contains_key(&new_ingredient.name) {
            return Err(CookbookError::UpdateError(
                format!("update_ingredient: Cannot rename: ingredient '{}' already exists", new_ingredient.name)
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
    
    /// Updates a recipe's properties including title, ingredients, prep time, downtime, servings, tags, and instructions
    pub fn update_recipe(&mut self, 
                       original_title: &str,
                       new_recipe: Recipe) -> Result<bool, CookbookError> {
        // Check if the original recipe exists
        if !self.recipes.iter().any(|r| r.title == original_title) {
            return Err(CookbookError::UpdateError(
                format!("update_recipe: Recipe '{}' does not exist", original_title)
            ));
        }
        
        // Check if the new title conflicts with an existing recipe (if title is changing)
        if original_title != new_recipe.title && self.recipes.iter().any(|r| r.title == new_recipe.title) {
            return Err(CookbookError::UpdateError(
                format!("update_recipe: Cannot rename: recipe '{}' already exists", new_recipe.title)
            ));
        }
        
        let recipes_dir = self.data_dir.join("recipes");
        let old_path = recipes_dir.join(format!("{}.md", original_title.replace(" ", "_")));
        let new_path = recipes_dir.join(format!("{}.md", new_recipe.title.replace(" ", "_")));
        
        // Update recipe in the recipes vector
        // First remove the old recipe
        self.recipes.retain(|r| r.title != original_title);
        
        // Add the new recipe
        self.recipes.push(new_recipe.clone());
        
        // Write the recipe to file
        new_recipe.to_file(&new_path)?;
        
        // If the title changed, remove the old file
        if original_title != new_recipe.title && old_path.exists() {
            fs::remove_file(old_path)
                .map_err(|e| CookbookError::WriteError(format!("Failed to remove old recipe file: {}", e)))?;
        }
        
        Ok(true)
    }
    
    /// Removes an ingredient from the pantry by name
    pub fn remove_from_pantry(&mut self, ingredient_name: &str) -> Result<bool, CookbookError> {
        if let Some(pantry) = self.pantry.as_mut() {
            let original_len = pantry.items.len();
            pantry.items.retain(|item| item.ingredient != ingredient_name);
            let pantry_path = self.data_dir.join("pantry.yaml");
            pantry.to_file(pantry_path)?;
            Ok(pantry.items.len() < original_len)
        } else {
            Err(CookbookError::UpdateError("No pantry loaded".to_string()))
        }
    }
    
    /// Updates an ingredient with potential changes to both ingredient properties and pantry values
    pub fn update_ingredient_with_pantry(
    &mut self,
    original_name: &str,
    new_ingredient: Ingredient,
    quantity: Option<f64>,
    quantity_type: Option<String>,
    remove_from_pantry: bool,
) -> Result<bool, CookbookError> {
    // First update the ingredient itself
    self.update_ingredient(original_name, new_ingredient.clone())?;

    if remove_from_pantry {
        // Remove from pantry if present
        if let Some(pantry) = self.pantry.as_mut() {
            pantry.items.retain(|item| item.ingredient != new_ingredient.name);
            let pantry_path = self.data_dir.join("pantry.yaml");
            pantry.to_file(pantry_path)?;
        }
    } else {
        // Add or update pantry item
        let qt = quantity_type.unwrap_or_else(|| "".to_string());
        self.update_pantry_item(&new_ingredient.name, quantity, Some(qt))?;
    }

    Ok(true)
}
    
    /// Creates a new DataManager instance with updated ingredient and pantry values
    /// This is a utility method for UIs that need to update ingredients while maintaining immutability
    /// Returns a new DataManager instance with the updated values
    pub fn create_with_updated_ingredient(
        data_dir: &Path,
        original_name: &str,
        new_ingredient: Ingredient,
        quantity: Option<f64>,
        quantity_type: Option<String>,
        remove_from_pantry: bool,
    ) -> Result<Self, CookbookError> {
        // Create a new DataManager instance
        let mut manager = DataManager::new(data_dir)?;
        // Perform the update
        manager.update_ingredient_with_pantry(original_name, new_ingredient, quantity, quantity_type, remove_from_pantry)?;
        // Return the updated manager
        Ok(manager)
    }
    
    /// Creates a new DataManager instance with an updated recipe
    /// This is a utility method for UIs that need to update recipes while maintaining immutability
    /// Returns a new DataManager instance with the updated recipe
    pub fn create_with_updated_recipe(
        data_dir: &Path,
        original_title: &str,
        new_recipe: Recipe,
    ) -> Result<Self, CookbookError> {
        // Create a new DataManager instance
        let mut manager = DataManager::new(data_dir)?;
        
        // Perform the update
        manager.update_recipe(original_title, new_recipe)?;
        
        // Return the updated manager
        Ok(manager)
    }
    
    /// Checks if all ingredients for a specific recipe are in the pantry
    /// Returns true if all ingredients are available, false otherwise
    /// The recipe_title parameter is the title of the recipe to check
    pub fn are_all_ingredients_in_pantry(&self, recipe_title: &str) -> bool {
        // First, get the recipe
        if let Some(recipe) = self.get_recipe(recipe_title) {
            // Check if all ingredients are in pantry
            recipe.ingredients.iter().all(|ingredient| self.is_in_pantry(&ingredient.ingredient))
        } else {
            false // Recipe not found
        }
    }
    
    /// Returns a sorted Vec of all unique ingredient categories.
    pub fn get_unique_categories(&self) -> Vec<String> {
        let mut categories = self
            .ingredients
            .values()
            .map(|ingredient| ingredient.category.clone())
            .collect::<std::collections::HashSet<String>>()
            .into_iter()
            .collect::<Vec<String>>();
        categories.sort();
        categories
    }
    
    /// Returns the path to a KB image in the engine's canonical KB directory.
    pub fn get_kb_image_path(&self, image_name: &str) -> Option<std::path::PathBuf> {
        // The canonical KB directory is at cookbook-engine/src/kb/
        let kb_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/kb");
        let image_path = kb_dir.join(image_name);
        if image_path.exists() {
            Some(image_path)
        } else {
            None
        }
    }
    
    /// Creates a new ingredient and writes it to the ingredients directory
    pub fn create_ingredient(&mut self, ingredient: Ingredient) -> Result<(), CookbookError> {
        if self.ingredients.contains_key(&ingredient.name) {
            return Err(CookbookError::UpdateError(format!(
                "create_ingredient: Ingredient '{}' already exists",
                ingredient.name
            )));
        }
        let ingredients_dir = self.data_dir.join("ingredients");
        let path = ingredients_dir.join(format!("{}.yaml", ingredient.name.replace(" ", "_")));
        ingredient.to_file(&path)?;
        self.ingredients.insert(ingredient.name.clone(), ingredient);
        Ok(())
    }
    
    /// Deletes an ingredient by name, and removes it from the pantry if present
    pub fn delete_ingredient(&mut self, ingredient_name: &str) -> Result<bool, CookbookError> {
        // Remove from ingredients map
        if self.ingredients.remove(ingredient_name).is_none() {
            return Err(CookbookError::UpdateError(format!("Ingredient '{}' does not exist", ingredient_name)));
        }
        // Remove ingredient file
        let ingredients_dir = self.data_dir.join("ingredients");
        let path = ingredients_dir.join(format!("{}.yaml", ingredient_name.replace(" ", "_")));
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| CookbookError::WriteError(format!("Failed to remove ingredient file: {}", e)))?;
        }
        // Remove from pantry if present
        if let Some(pantry) = self.pantry.as_mut() {
            pantry.items.retain(|item| item.ingredient != ingredient_name);
            let pantry_path = self.data_dir.join("pantry.yaml");
            pantry.to_file(pantry_path)?;
            Ok(true)
        } else {
            Ok(true)
        }
    }
    
    /// Search for an ingredient by any translation (singular or plural) in the given language, or by slug
    pub fn find_ingredient_by_name_or_translation(
        &self,
        query: &str,
        lang: &str,
    ) -> Option<&Ingredient> {
        let query_lower = query.to_lowercase();
        for ingredient in self.get_all_ingredients() {
            // Check slug
            if ingredient.slug.to_lowercase() == query_lower {
                return Some(ingredient);
            }
            // Check name
            if ingredient.name.to_lowercase() == query_lower {
                return Some(ingredient);
            }
            // Check translations
            if let Some(translations) = &ingredient.translations {
                if let Some(forms) = translations.get(lang) {
                    if forms.one.to_lowercase() == query_lower || forms.other.to_lowercase() == query_lower {
                        return Some(ingredient);
                    }
                }
                // Fallback to English
                if let Some(forms) = translations.get("en") {
                    if forms.one.to_lowercase() == query_lower || forms.other.to_lowercase() == query_lower {
                        return Some(ingredient);
                    }
                }
            }
        }
        None
    }
    
    /// Returns the display name for an ingredient in the given language and quantity (for pluralization)
    pub fn ingredient_display_name(
        ingredient: &Ingredient,
        lang: &str,
        quantity: Option<f64>,
    ) -> String {
        if let Some(translations) = &ingredient.translations {
            if let Some(forms) = translations.get(lang) {
                let n = quantity.unwrap_or(1.0);
                if (n - 1.0).abs() < f64::EPSILON {
                    return forms.one.clone();
                } else {
                    return forms.other.clone();
                }
            }
            // fallback to English
            if let Some(forms) = translations.get("en") {
                let n = quantity.unwrap_or(1.0);
                if (n - 1.0).abs() < f64::EPSILON {
                    return forms.one.clone();
                } else {
                    return forms.other.clone();
                }
            }
        }
        // fallback to name or slug
        if !ingredient.name.is_empty() {
            ingredient.name.clone()
        } else {
            ingredient.slug.clone()
        }
    }
    
    /// Returns the display name for a recipe ingredient in the correct language and plural form
    pub fn recipe_ingredient_display_name(
        &self,
        recipe_ingredient: &RecipeIngredient,
        lang: &str,
    ) -> String {
        if let Some(ingredient) = self.get_ingredient(&recipe_ingredient.ingredient) {
            let qty = recipe_ingredient.quantity;
            Self::ingredient_display_name(ingredient, lang, qty)
        } else {
            // fallback to whatever is in the recipe
            recipe_ingredient.ingredient.clone()
        }
    }
}
