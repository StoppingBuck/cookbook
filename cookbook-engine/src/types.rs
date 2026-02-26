use serde::{Deserialize, Serialize}; // For serialization and deserialization
use std::collections::HashMap; // For storing ingredients and recipes
use thiserror::Error; // For error handling

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

impl Recipe {
    /// Returns the total time required for the recipe (prep time + downtime)
    pub fn total_time(&self) -> u32 {
        self.prep_time.unwrap_or(0) + self.downtime.unwrap_or(0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PantryItem {
    pub ingredient: String, //#TODO Should it perhaps be an Ingredient?
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
