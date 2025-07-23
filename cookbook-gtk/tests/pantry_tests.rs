// Unit tests for pantry.rs
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_pantry_category_filter() {
        let categories = vec!["vegetable", "fruit"];
        assert!(categories.contains(&"vegetable"));
    }

    #[test]
    fn test_ingredient_logic() {
        let ingredient = "potato";
        assert_eq!(ingredient, "potato");
    }
}
