// Unit tests for types.rs
#[cfg(test)]
mod tests {
    use super::*;
    use cookbook_gtk::types::{AppMsg, Tab};
    #[test]
    fn test_appmsg_enum() {
        let msg = AppMsg::SwitchTab(Tab::Recipes);
        match msg {
            AppMsg::SwitchTab(tab) => assert_eq!(tab, Tab::Recipes),
            _ => panic!("Wrong variant"),
        }
    }
}
