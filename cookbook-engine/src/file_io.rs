use crate::types::*;
use std::fs;
use std::path::Path;
use log::info;

// Implementing method for Ingredient
impl Ingredient {
    // Reads an ingredient from a YAML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, CookbookError> {
        let content =
            fs::read_to_string(&path).map_err(|e| CookbookError::ReadError(e.to_string()))?; // Read the file content
        serde_yaml::from_str(&content).map_err(|e| CookbookError::ParseError(e.to_string()))
        // Parse the YAML content
    }

    // Writes an ingredient to a YAML file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), CookbookError> {
        let yaml = serde_yaml::to_string(self).map_err(|e| {
            CookbookError::ParseError(format!("Failed to serialize ingredient: {}", e))
        })?; // Serialize the ingredient to YAML

        fs::write(&path, yaml).map_err(|e| {
            CookbookError::WriteError(format!("Failed to write ingredient file: {}", e))
        })?; // Write the YAML content to the file
        info!("Successfully wrote to {}", path.as_ref().display());
        Ok(()) // Return Ok if successful
    }
}

// Implementing method for Pantry
impl Pantry {
    // Reads a pantry from a YAML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, CookbookError> {
        let content =
            fs::read_to_string(&path).map_err(|e| CookbookError::ReadError(e.to_string()))?; // Read the file content
        serde_yaml::from_str(&content).map_err(|e| CookbookError::ParseError(e.to_string()))
        // Parse the YAML content
    }

    // Writes a pantry to a YAML file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), CookbookError> {
        let yaml = serde_yaml::to_string(self)
            .map_err(|e| CookbookError::ParseError(format!("Failed to serialize pantry: {}", e)))?; // Serialize the pantry to YAML

        fs::write(&path, yaml).map_err(|e| {
            CookbookError::WriteError(format!("Failed to write pantry file: {}", e))
        })?; // Write the YAML content to the file
        info!("Successfully wrote to {}", path.as_ref().display());
        Ok(()) // Return Ok if successful
    }
}

// Implementing method for Recipe
impl Recipe {
    /// Reads a recipe from a Markdown file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, CookbookError> {
        let content =
            fs::read_to_string(&path).map_err(|e| CookbookError::ReadError(e.to_string()))?; // Read the file content

        // Split frontmatter from content by finding the two "---" delimiters
        let parts: Vec<&str> = content.splitn(3, "---").collect();

        // Check if we have at least three parts: before, frontmatter, and after
        if parts.len() < 3 {
            return Err(CookbookError::MarkdownError(
                "Invalid markdown format: missing frontmatter delimiters".to_string(),
            ));
        }

        // Parse the YAML frontmatter (parts[1] is between the first and second ---)
        let mut recipe: Recipe = serde_yaml::from_str(parts[1].trim()).map_err(|e| {
            CookbookError::ParseError(format!("Failed to parse YAML frontmatter: {}", e))
        })?;

        // Store the instructions (parts[2] is after the second ---)
        recipe.instructions = parts[2].trim().to_string();

        Ok(recipe) // Return the parsed recipe
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
        fs::write(&path, content).map_err(|e| {
            CookbookError::WriteError(format!("Failed to write recipe file: {}", e))
        })?;
        info!("Successfully wrote to {}", path.as_ref().display());
        Ok(())
    }
}

// Implementing method for KnowledgeBaseEntry
impl KnowledgeBaseEntry {
    /// Reads a knowledge base entry from a Markdown file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, CookbookError> {
        let content =
            fs::read_to_string(&path).map_err(|e| CookbookError::ReadError(e.to_string()))?; // Read the file content

        // Split frontmatter from content by finding the two "---" delimiters
        let parts: Vec<&str> = content.splitn(3, "---").collect();

        // Check if we have at least three parts: before, frontmatter, and after
        if parts.len() < 3 {
            return Err(CookbookError::MarkdownError(
                "Invalid markdown format: missing frontmatter delimiters".to_string(),
            ));
        }

        // Parse the YAML frontmatter (parts[1] is between the first and second ---)
        let mut kb_entry: KnowledgeBaseEntry =
            serde_yaml::from_str(parts[1].trim()).map_err(|e| {
                CookbookError::ParseError(format!("Failed to parse YAML frontmatter: {}", e))
            })?;

        // Store the content (parts[2] is after the second ---)
        kb_entry.content = parts[2].trim().to_string();

        Ok(kb_entry) // Return the parsed knowledge base entry
    }
}
