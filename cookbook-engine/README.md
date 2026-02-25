# cookbook-engine

The core Rust library shared by all frontends. Contains all business logic for reading, writing, and querying recipe, ingredient, pantry, and knowledge-base data. Has no GUI dependencies.

## Public API

The main entry point is `DataManager`. Create one by pointing it at a data directory:

```rust
use cookbook_engine::DataManager;

let dm = DataManager::new("/path/to/data")?;
```

### Ingredients

```rust
// List all ingredients
let ingredients: Vec<Ingredient> = dm.get_all_ingredients();

// Find by slug or translated name
let ingredient = dm.find_ingredient_by_name_or_translation("potato", "en");

// Filter (used for pantry list)
let filtered = dm.filter_ingredients(
    "pot",              // search text
    &["vegetable"],     // category filter (empty = all)
    true,               // in-stock only
    "en",               // display language
);

// Add / update / delete
dm.add_ingredient(&ingredient)?;
dm.update_ingredient("old-slug", &updated_ingredient)?;
dm.delete_ingredient("slug")?;
```

### Pantry

```rust
// Read pantry state
let pantry: Vec<PantryItem> = dm.get_pantry_items();

// Update a single pantry item
dm.update_pantry_status(
    "potato",           // ingredient slug
    true,               // in stock
    Some(2.0),          // quantity
    Some("kg"),         // quantity type
)?;
```

### Recipes

```rust
// List all recipes
let recipes: Vec<Recipe> = dm.get_all_recipes();

// Save a recipe (creates or overwrites the Markdown file)
dm.save_recipe(&recipe)?;

// Delete
dm.delete_recipe("Lasagna")?;
```

### Knowledge Base

```rust
let entries: Vec<KbEntry> = dm.get_all_kb_entries();
let entry = dm.get_kb_entry_by_slug("potato");
```

## Data types

```rust
pub struct Ingredient {
    pub name: String,
    pub slug: String,
    pub category: String,
    pub kb: Option<String>,
    pub tags: Option<Vec<String>>,
    pub translations: Option<HashMap<String, TranslationForms>>,
}

pub struct PantryItem {
    pub ingredient: String,   // ingredient slug
    pub quantity: Option<f64>,
    pub quantity_type: Option<String>,
    pub last_updated: Option<String>,
    pub in_stock: Option<bool>,
}

pub struct Recipe {
    pub title: String,
    pub ingredients: Vec<RecipeIngredient>,
    pub prep_time: Option<u32>,
    pub downtime: Option<u32>,
    pub servings: Option<u32>,
    pub tags: Option<Vec<String>>,
    pub body: String,
}
```

## Data directory layout

```
data/
├── ingredients/<slug>.yaml   one file per ingredient
├── pantry.yaml               all pantry state
├── recipes/<Title>.md        Markdown with YAML frontmatter
└── kb/<slug>.md              optional knowledge base articles
```

The `slug` field in ingredient YAML is the canonical identifier used everywhere. File names are derived from slugs but you should never rely on file name matching — always use the `slug` field.

## Error handling

All fallible operations return `Result<T, CookbookError>`. The `CookbookError` enum covers I/O errors, YAML parse errors, and domain-level errors (e.g., duplicate ingredient slug).

## Using from Android (JNI)

See `pantryman/rust-bridge/` for the JNI wrapper. The bridge exposes a subset of `DataManager` methods as C-compatible functions that are called from Kotlin via `CookbookEngine.kt`.
