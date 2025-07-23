# TESTING.md

## Cookbook-GTK Test Suite Overview

This document summarizes all automated tests for the `cookbook-gtk` frontend, including unit, integration, and UI tests. It is kept up-to-date as tests are added or changed.

---

### Table of Contents
- [Unit Tests](#unit-tests)
- [Integration Tests](#integration-tests)
- [UI Integration Tests](#ui-integration-tests)
- [Test Data](#test-data)
- [How to Run Tests](#how-to-run-tests)

---

## Unit Tests
Unit tests cover isolated logic for each module. They are fast and do not require a GUI.

- **dialogs_tests.rs**
  - `test_about_dialog_logic`: Verifies about dialog logic and content.
  - `test_error_dialog_logic`: Checks error dialog creation and error propagation.
- **i18n_tests.rs**
  - `test_set_language`: Ensures language setting logic works.
  - `test_language_fallback`: Verifies fallback to default language.
- **kb_tests.rs**
  - `test_kb_details_placeholder`: Checks placeholder logic for KB details.
  - `test_kb_slug_logic`: Validates KB slug resolution.
- **pantry_tests.rs**
  - `test_ingredient_logic`: Tests ingredient filtering and display logic.
  - `test_pantry_category_filter`: Verifies category filter logic for pantry items.
- **recipes_tests.rs**
  - `test_recipe_details_update`: Ensures recipe details update correctly.
  - `test_recipe_title_logic`: Checks recipe title handling and display.
- **settings_tests.rs**
  - `test_data_dir_logic`: Validates data directory setup and migration.
  - `test_settings_tab_build`: Tests settings tab UI construction.
- **sidebar_tests.rs**
  - `test_sidebar_button_logic`: Verifies sidebar button event handling.
- **tabs_tests.rs**
  - `test_tab_switching`: Ensures tab switching logic works.
- **types_tests.rs**
  - `test_appmsg_enum`: Checks AppMsg enum variants and message passing.
- **ui_constants_tests.rs**
  - `test_default_margin`: Validates UI margin constants.
- **user_settings_tests.rs**
  - `test_user_settings_load_default`: Tests loading default user settings.
  - `test_user_settings_load_custom_dir`: Verifies loading settings from a custom directory.
- **utils_tests.rs**
  - `test_validate_and_create_data_dir`: Ensures data directory creation and validation logic.

---

## Integration Tests
Integration tests cover interactions between multiple modules and components.

- **app_tests.rs**
  - (Currently no tests; placeholder for future app-level integration tests.)

---

## UI Integration Tests
UI integration tests simulate user interactions and verify end-to-end UI behavior. These require a display and may be skipped in headless CI.

- **ui_integration_tests.rs**
  - `test_recipe_selection_ui`: Simulates selecting a recipe in the UI, manually updates the details pane, and asserts that the details are displayed correctly. Uses GTK event flushing and direct widget population for robust testability.

---

## Test Data
All tests use sample data from `example/data/`:
- `ingredients/`: Ingredient YAML files
- `pantry.yaml`: Pantry state
- `recipes/`: Recipe Markdown files

---

## How to Run Tests
Run all tests (including UI integration tests) with:

```bash
./dev.sh gtk-test
```

This will build and execute all unit, integration, and UI tests for the GTK frontend. If any test fails, fix the errors and rerun until all tests pass.

---

## Maintenance Note
**Agent Note:**
This file must be kept up-to-date as new tests are added, removed, or changed. Update this summary whenever you modify the test suite.
