# TODO

Open issues, known bugs, and planned work. Roughly prioritised within each section.

---

## Bugs

- **Quantity matching is incomplete** (`cookbook-gtk`, recipes tab): The recipe-to-pantry match only checks that *some* quantity exists, not whether it is enough. Needs actual comparison (recipe needs 500 g, pantry has 200 g → insufficient).

- **Language hardcoded to `"en"` in pantry list** (`cookbook-gtk/src/pantry/`): `rebuild_pantry_list` always passes `"en"` to `filter_ingredients` instead of reading from `UserSettings`. Translated ingredient names are never used for display or filtering.

- **JNI raw pointer safety** (`pantryman/rust-bridge/src/lib.rs`): Multiple `unsafe { &*(ptr as *const DataManager) }` casts with no null-check. A null or dangling pointer will crash the app. Wrap in `NonNull` or add a validity sentinel.

- **`onPause` sync not guaranteed to complete** (`pantryman`): `syncToSAF` runs on a background thread started from `onPause`. Android can kill background threads quickly after pause. For reliable write-on-close, migrate to `WorkManager`.

- **SAF `"wt"` mode not universally supported**: Some SAF providers do not support write-truncate (`"wt"`) mode in `openOutputStream`. If sync-to silently produces empty files on a given provider, try `"w"` instead.

---

## cookbook-engine

- **`PantryItem` stores ingredient name as `String`**: Every pantry lookup requires a secondary map lookup. Consider validated references or embedding the slug.

- **Knowledge Base ↔ ingredient link validation**: No mechanism to detect broken `kb:` references in ingredient YAML files.

- **Quantity unit normalisation**: No canonical unit system. `"kg"`, `"Kg"`, and `"KG"` are treated as different units.

---

## cookbook-gtk

- **Pantry list rebuilds from scratch on every change**: `rebuild_pantry_list` removes and recreates all rows on every update. Should do incremental/diff-based updates for large ingredient sets.

- **Recipe body renders as raw Markdown**: The recipe detail view displays raw Markdown text. Should render to Pango markup or a WebView equivalent.

- **Ingredient substitution not implemented**: No suggestion or application of ingredient substitutions in the recipes tab.

- **`update()` handler is large**: The `update()` match block handles many variants. Consider delegating to per-tab handler functions.

---

## pantryman

- **Recipes tab**: Not implemented. There is a clear use case (reading recipes while cooking), but scope was deliberately limited for v0.1.0.

- **Knowledge Base tab**: Not implemented.

- **No automated tests**: Add unit tests for the Kotlin layer (`CookbookEngine` wrapper, sync logic). The JNI bridge has no integration tests.

- **Deletion sync edge case**: If an ingredient is deleted on Android while the cloud folder is unreachable, the deletion will sync out on next `onPause` but will not be reflected on desktop until desktop reloads its data dir.

---

## Infrastructure / docs

- **`cookbook-engine` API docs**: No `rustdoc` comments on public types and methods.

- **CI**: No automated build or test pipeline.

- **Release signing**: The Android APK is debug-signed. A proper release signing setup is needed before distribution.

- **`pantryman/rust-bridge` workspace exclusion**: The bridge crate must be checked and built separately. Consider whether a single-workspace setup with conditional compilation targets is feasible.
