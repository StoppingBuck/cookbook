# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common Commands

Use `./dev.sh` for all common tasks:

```bash
./dev.sh gtk              # Build and run GTK desktop app
./dev.sh gtk-compile      # Compile GTK app without running
./dev.sh gtk-test         # Run all tests for cookbook-gtk
./dev.sh check            # Run cargo check on all Rust components
./dev.sh test             # Run all tests
./dev.sh clean            # Clean build artifacts
./dev.sh android          # Build, install, run, and stream logs for Pantryman
./dev.sh engine-test     # Run cookbook-engine tests (fast, no display needed)
./dev.sh gtk-test-headless  # Run GTK tests headlessly (requires xvfb-run)
```

For verbose GTK log output: `RUST_LOG=debug ./dev.sh gtk`

**Cookbook-GTK compile loop:** After every edit to `cookbook-gtk`, run `./dev.sh gtk-compile` immediately. Fix all errors before running it again.

**Single test:** `cargo test -p cookbook-gtk <test_name>`

**Cargo check (Rust workspace only):** `cargo check` — note that `pantryman/rust-bridge` is excluded from the workspace and must be checked separately.

## Architecture

This is a cross-platform recipe and pantry management application with three components:

### `cookbook-engine/` (Rust library)
The core business logic layer. Exposes a `DataManager` struct that handles all file I/O and domain logic. No GUI code here — ever.

Key types: `Ingredient`, `PantryItem`, `Recipe`, `KbEntry`, `DataManager`.

```
cookbook-engine/src/
  lib.rs          — module declarations and re-exports
  types.rs        — all data types (Ingredient, Recipe, PantryItem, Pantry, KbEntry, etc.)
  file_io.rs      — file read/write impls (from_file/to_file)
  data_manager.rs — DataManager struct and all business logic methods
```

### `cookbook-gtk/` (Rust + GTK4 desktop app)
Frontend built with [Relm4](https://relm4.org/) (GTK4 bindings). The `AppModel` (in `lib.rs`) implements `SimpleComponent` and holds all app state. Modules map to tabs/views: `recipes.rs`, `pantry.rs`, `kb.rs`, `settings.rs`, `sidebar.rs`. Message passing uses the `AppMsg` enum in `types.rs`.

Data dir defaults to `example/data/` during development; overridden via the `COOKBOOK_DATA_DIR` env var or user settings stored at `~/.config/cookbook-gtk/user_settings.toml`.

### `pantryman/` (Kotlin Android app)
Mobile companion app. Calls `cookbook-engine` via JNI through `pantryman/rust-bridge/` (a separate `cdylib` crate excluded from the main workspace). Build the Rust bridge with `cargo ndk` before building the Android app.

## Data Format

All data is stored as plain files — no database:

- `ingredients/<name>.yaml` — one file per ingredient
- `pantry.yaml` — pantry state referencing ingredient slugs
- `recipes/<Name>.md` — Markdown with YAML frontmatter
- `kb/<slug>.md` — knowledge base articles

**Never match KB entries by filename — always use the `slug` field.**

Ingredient YAML:
```yaml
name: potato
slug: potato
category: vegetable
kb: potato
tags: [vegetable, starch]
```

Pantry YAML:
```yaml
version: 1
items:
  - ingredient: potato
    quantity: 2
    quantity_type: kg
    last_updated: 2025-05-10
```

Recipe Markdown frontmatter:
```yaml
Title: Lasagna
Ingredients:
  - ingredient: potato
    quantity: 2
    quantity_type: kg
PrepTime: 30
Downtime: 60
Servings: 4
Tags: [dinner, pasta]
```

## Key Conventions

- **Strict separation:** Business logic in `cookbook-engine`, UI in frontends. Never mix.
- **Rust naming:** `snake_case` for files, structs, and functions.
- **Android:** Idiomatic Kotlin with Jetpack libraries; FFI calls go through `pantryman/rust-bridge`.
- **Error handling:** Propagate errors to UI — dialogs in GTK, toasts/dialogs in Android.
- **Data migration:** Use `version:` fields in YAML for future migrations; don't implement until required.
- **Test data:** `example/data/` is always safe to use for testing and bootstrapping.
