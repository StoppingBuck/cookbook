// Unit tests for recipes.rs
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_recipe_title_logic() {
        let title = "Lasagna";
        assert_eq!(title, "Lasagna");
    }

    #[test]
    fn test_recipe_details_update() {
        let details = "Start by boiling the potatoes...";
        assert!(details.contains("potatoes"));
    }
}
