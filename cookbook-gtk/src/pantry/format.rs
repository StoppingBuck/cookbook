/// Formats a quantity with its unit type as a display string.
pub fn format_quantity(quantity: Option<f64>, quantity_type: &str) -> String {
    match quantity {
        Some(q) => {
            if quantity_type.is_empty() {
                format!("{}", q)
            } else {
                format!("{} {}", q, quantity_type)
            }
        }
        None => String::new(),
    }
}
