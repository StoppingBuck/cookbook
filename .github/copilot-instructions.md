> This file exists to help GitHub Copilot understand the structure, purpose and intended behavior of the `cookbook` project. It may also help human collaborators get oriented faster.

# Cookbook Project - AI Agent Instructions

## Architecture Overview
- **Multi-crate structure:**
  - `cookbook-engine`: Rust core library, all business logic, file-based YAML/Markdown data, no GUI.
  - `cookbook-gtk`: GTK4 desktop frontend (Rust, Relm4, gtk4-rs), uses `cookbook-engine` via direct Rust API.
  - `pantryman`: Android app (Kotlin, Gradle, JNI bridge to Rust), uses `cookbook-engine` via FFI.
  - `example/data/`: Sample data for dev/testing, always safe to use for bootstrapping.

## Data Flow & Conventions
- **No database:** All persistent data is in text files (YAML for pantry/ingredients, Markdown+YAML frontmatter for recipes/KB).
- **Data directory structure:**
  - `ingredients/` (YAML per ingredient)
  - `pantry.yaml` (YAML, references ingredient names)
  - `recipes/` (Markdown w/ YAML frontmatter)
  - `kb/` (Markdown w/ YAML frontmatter, images)
- **Versioning:** Use `version:` field in global YAML files for future migrations.
- **Never rely on file name matching for KB links—always use `slug` fields.**

## Developer Workflows
  - Example: `./dev.sh gtk`, `./dev.sh android-build`, `./dev.sh check`, etc.
 **Cookbook-GTK compile-check loop:**
  - When editing any code in `cookbook-gtk`, always run `./dev.sh gtk-compile` immediately after each edit.
  - If there are any compilation errors, address all errors before running `./dev.sh gtk-compile` again.
  - Only stop once there are no more compilation errors.
  - This loop is the default workflow for cookbook-gtk development.

## Integration Points
- **FFI boundary:** Pantryman Android app uses JNI to call Rust code in `cookbook-engine` (see `pantryman/rust-bridge`).
- **GTK frontend:** Direct Rust API calls to `cookbook-engine`.
- **Shared types:** Use `cookbook-shared` for cross-crate types/utilities.

## Project-Specific Patterns
- **Separation of concerns:** Business logic in engine, UI in frontend. Never mix.
- **Data migration:** Plan for versioning in YAML files, but do not implement unless required.
- **Error handling:** Always propagate errors to UI; show dialogs in GTK, toasts/dialogs in Android.
- **Naming:**
  - Rust: snake_case for files, structs, functions.
  - Qt/C++: PascalCase.
  - Consistent domain types: `Recipe`, `Ingredient`, `PantryItem`, etc.

## Examples
- **Ingredient YAML:**
  ```yaml
  name: potato
  category: vegetable
  kb: potato
  tags: [vegetable, starch]
  ```
- **Pantry YAML:**
  ```yaml
  version: 1
  items:
    - ingredient: potato
      quantity: 2
      quantity_type: kg
      last_updated: 2025-05-10
  ```
- **Recipe Markdown:**
  ```markdown
  ---
  Title: Lasagna
  Ingredients:
    - ingredient: potato
      quantity: 2
      quantity_type: kg
  PrepTime: 30
  Downtime: 60
  Servings: 4
  Tags: [dinner, pasta]
  ---
  Start by boiling the potatoes...
  ```

## Special Agent Notes
- If a process hangs, user may kill it with CTRL+C—ignore this in output analysis.
- Never replace entire files via terminal (no EOF markers); only patch regions.
- Reference README.md for platform-specific setup and troubleshooting.

---