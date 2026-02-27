# Changelog

All notable changes to this project will be documented in this file.

Format: [Semantic Versioning](https://semver.org). Types of changes: `Added`, `Changed`, `Fixed`, `Removed`.

---

## [0.1.0] — 2026-02-27

First working beta. Core sync loop between desktop and Android is functional end-to-end.

### Added

**cookbook-engine**
- `DataManager` — core struct for loading and managing all data (ingredients, pantry, recipes, KB)
- Full CRUD API: `get_ingredient`, `create_ingredient`, `update_ingredient`, `delete_ingredient`, `update_pantry_status`, `get_all_ingredients`, `get_pantry`, `get_recipe`, `list_recipes`, `save_recipe`, `delete_recipe`, `list_kb_entries`, `get_kb_entry`
- Plain-file storage: `ingredients/*.yaml`, `pantry.yaml`, `recipes/*.md`, `kb/*.md`
- Comprehensive test suite (36 tests in `cookbook-engine/tests/`)

**cookbook-gtk**
- Recipes tab: browse, create, edit, delete recipes; filter by in-stock ingredients
- Pantry tab: ingredient library with categories, tags, quantities, and units; in-stock filter
- Knowledge Base tab: browse and view KB articles with Pango-rendered Markdown
- Settings: folder picker to point the app at any local or cloud-synced directory; light/dark/system theme
- Async data directory loading — window appears immediately even on slow/FUSE filesystems
- Structured logging via `env_logger` (`RUST_LOG=debug ./dev.sh gtk` for verbose output)

**pantryman**
- Pantry list with search, category grouping, and quantity display
- Add/update/remove pantry items with quantity and unit
- Create new ingredients from within the pantry flow
- SAF-based cloud sync: pick any SAF folder (pCloud, Google Drive, Nextcloud, etc.)
- Automatic sync-in on `onResume`, sync-out on `onPause`
- Manual "Sync Now" button in settings
- Bidirectional mirror including deletion propagation
- Material3 UI; Android 15 edge-to-edge handled correctly

**Infrastructure**
- `dev.sh` helper script covering build, test, check, and Android deploy workflows
- `./dev.sh android` requires a connected hardware device — no emulator fallback

### Fixed
- Migrated Android from deprecated `onActivityResult` / `SettingsActivity` to `registerForActivityResult` + SAF
- Migrated Android `finalize()`-based engine cleanup to explicit `onDestroy` lifecycle
- GTK settings panel layout — removed shrink-wrapping outer Box that caused character-level text wrapping
- GTK subtitle path display — limited to one line with ellipsis
- Android status bar overlap on Android 15+ (edge-to-edge enforcement)
