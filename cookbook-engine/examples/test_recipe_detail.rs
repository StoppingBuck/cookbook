use cookbook_engine::{DataManager, Recipe};
use std::path::Path;

fn main() {
    // Path to the example data directory
    let data_dir = Path::new("example/data");
    
    // Load the data manager
    match DataManager::new(data_dir) {
        Ok(manager) => {
            // Get all recipes
            let recipes = manager.get_all_recipes();
            
            println!("Found {} recipes:", recipes.len());
            
            // Print details for each recipe
            for (i, recipe) in recipes.iter().enumerate() {
                println!("\nRecipe #{}: {}", i+1, recipe.title);
                println!("  Prep time: {} min", recipe.prep_time);
                println!("  Downtime: {} min", recipe.downtime);
                println!("  Total time: {} min", recipe.total_time());
                println!("  Servings: {}", recipe.servings);
                println!("  Tags: {:?}", recipe.tags);
                
                println!("  Ingredients:");
                for ingredient in &recipe.ingredients {
                    let qty = ingredient.quantity.map_or(String::new(), |q| q.to_string());
                    let qty_type = ingredient.quantity_type.as_deref().unwrap_or("");
                    println!("    - {} {} of {}", qty, qty_type, ingredient.ingredient);
                }
                
                // Print instructions (first 100 chars)
                let preview = if recipe.instructions.len() > 100 {
                    format!("{}...", &recipe.instructions[..100])
                } else {
                    recipe.instructions.clone()
                };
                println!("  Instructions: {}", preview);
                
                // Check if instructions are empty (which would indicate a problem)
                if recipe.instructions.is_empty() {
                    println!("  WARNING: Instructions are empty!");
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to load data manager: {}", e);
        }
    }
}
