# TODO

Open issues, known bugs, and refactoring ideas across the project.

---

## Bugs

- **Quantity matching is incomplete** (`cookbook-gtk`, recipes tab): The recipe-to-pantry matching only checks that *some* quantity exists for an ingredient, not whether it's enough for the recipe. Needs actual quantity comparison (e.g., recipe needs 500g, pantry has 200g → show as insufficient).

- **Language hardcoded to `"en"` in pantry list** (`cookbook-gtk/src/pantry.rs` line 33): `rebuild_pantry_list` always passes `"en"` to `filter_ingredients` instead of reading from `model.user_settings`. This means translated ingredient names are never used for filtering/display regardless of the user's language setting.

- **Content URI not converted to file path on Android** (`pantryman/SettingsActivity.kt`): When the user picks a directory with the Android file picker, the returned content URI is not properly resolved to an absolute file path. The app silently falls back to the default directory. External storage (SD cards, USB drives) cannot be set as the data directory until this is fixed.

- **`finalize()` deprecated in Kotlin** (`pantryman/CookbookEngine.kt`): Uses the deprecated `finalize()` method to free the Rust pointer. Should use a lifecycle-aware mechanism (e.g., `Closeable`/`AutoCloseable`, or tie cleanup to an Android `ViewModel`).

- **`onActivityResult()` deprecated on Android** (`pantryman/MainActivity.kt`): Should be migrated to `registerForActivityResult()` with `ActivityResultContracts`.

---

## Engine (`cookbook-engine`)

- **No unit tests**: The entire `DataManager` API has zero unit tests. This is the most critical testing gap. Priority areas:
  - YAML read/write round-trips for ingredients and pantry
  - Recipe Markdown parsing (frontmatter + body)
  - `filter_ingredients` with search, category, and in-stock combinations
  - Error handling for malformed/missing files

- **`PantryItem` stores ingredient name as `String`**: Requires a secondary lookup every time pantry item details are needed. Consider whether storing the full `Ingredient` inline (or a validated reference) would reduce lookup overhead and prevent stale references.

- **Knowledge base coupling with ingredients is undesigned**: KB articles exist but there is no defined update strategy or automated link validation between KB slugs and ingredient records.

---

## GTK Frontend (`cookbook-gtk`)

- **Excessive debug `println!` statements**: Dozens of `println!` / `eprintln!` calls remain in production code across `main.rs`, `lib.rs`, `pantry.rs`, `user_settings.rs`, and `dialogs.rs`. These should be converted to proper `log::debug!` / `log::warn!` calls or removed.

- **`update()` and `update_view()` are too large** (`main.rs`): `update()` handles 16+ message variants in a single match block (~250 lines); `update_view()` manually manages widgets for all four tabs (~230 lines). Both should delegate to per-tab handler functions.

- **Pantry list rebuilds from scratch on every change**: `rebuild_pantry_list` removes all children and re-creates every row whenever the list needs refreshing. For large ingredient sets this causes noticeable redraws. Should do incremental/diff-based updates.

- **Duplicate settings-callback patterns** (`main.rs` vs `lib.rs`): `UserSettings` loading and the language/theme/data-dir callback closures are duplicated between `main.rs` and `lib.rs`. Consolidate into a single initialisation path.

- **Deprecated GTK API** (`lib.rs` lines 131, 136, 141): Uses `gtk-application-prefer-dark-theme` via `set_property()`, which is marked deprecated. Should migrate to the current GTK4 theme API.

- **`AppModel` mixes UI flags with business state** (`types.rs`): Fields like `refresh_category_popover: Cell<bool>` and `pantry_list_needs_rebuild: Cell<bool>` are internal rendering hints, not application state. Should be kept in `AppWidgets` or handled via a dedicated UI-update message.

- **GTK test suite is mostly stubs**: All 14 test files under `cookbook-gtk/tests/` exist but most contain placeholder logic rather than real assertions. The test descriptions in `TESTING.md` describe intended tests, not implemented ones.

- **Markdown support for recipe body is incomplete**: Recipes are stored as Markdown but the GTK viewer does not render Markdown formatting — the body is displayed as raw text.

- **Ingredient substitution not implemented**: The recipes tab has no mechanism for suggesting or applying ingredient substitutions (equal or non-equal).

---

## Android (`pantryman`)

- **No automated tests**: There are no unit tests, no instrumented tests, and no JNI integration tests anywhere in `pantryman/`. At a minimum, test the `CookbookEngine` Kotlin wrapper and the data-directory migration logic.

- **JNI raw pointer safety** (`pantryman/rust-bridge/src/lib.rs`): Multiple `unsafe { &*(ptr as *const DataManager) }` casts with no null-check or validity guard. A null or dangling pointer will cause a crash. Consider a `NonNull` wrapper or a validity sentinel.

- **Data directory migration is incomplete for external sources**: If the new data directory chosen by the user is not empty and is on external storage, the migration logic may not handle all content URI edge cases correctly (see content URI bug above).

- **Excessive `Log.d` spam in `MainActivity.kt`**: Debug log statements at startup and in callbacks should be removed or gated behind a debug build flag before release.

---

## Refactoring Ideas

- **Extract per-tab state into sub-structs**: `AppModel` has 12 fields. Grouping pantry-specific state (`selected_ingredient`, `selected_pantry_categories`, `show_in_stock_only`) and recipe-specific state (`selected_recipe`) into dedicated sub-structs would make the model easier to reason about and test.

- **Group `AppWidgets` by tab**: `AppWidgets` holds 13 widget references with no logical grouping. Create `PantryWidgets`, `RecipesWidgets`, `KbWidgets` structs and embed them.

- **Extract a `SettingsCallbacks` struct**: The language, theme, and data-dir callbacks in `lib.rs` and `main.rs` are structurally identical closures with boilerplate save/reload sequences. A dedicated struct or builder would eliminate the duplication and make unit-testing settings changes possible.

- **Move ingredient translation lookup out of the view layer**: `pantry.rs` contains pluralisation and translation logic that belongs in `cookbook-engine`. The view should receive a pre-translated display string from the engine.

- **Split large source files**: `pantry.rs` (1 232 lines) and `recipes.rs` (1 177 lines) each mix filtering, widget-building, event-binding, and detail-view rendering. These should be split into focused modules (e.g., `pantry/filter.rs`, `pantry/list.rs`, `pantry/detail.rs`).

- **Language-agnostic ingredient identity**: The project goal is for ingredients to have a canonical identity independent of language. Currently the slug serves this role, but the engine's `find_ingredient_by_name_or_translation` does ad-hoc matching. A cleaner design would make slug the sole key and keep display names purely in a translation layer.

---

## Documentation

- **`pantryman/README.md` is outdated**: Still describes the project as a "Hello World" app.
- **`cookbook-engine/README.md` is empty**: Needs API overview and data-format reference.
- **`PANTRYMAN_INGREDIENT_UPDATE_RESOLUTION.md`**: Historical artefact; can be deleted once the fix is confirmed stable.
- **No `CONTRIBUTING.md`**: No guide for external contributors (branching strategy, PR checklist, code style).
- **No architecture diagram**: A simple diagram showing `cookbook-engine` ← GTK / Android ← JNI bridge would help new contributors orient quickly.
