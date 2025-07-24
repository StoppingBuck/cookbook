
# Pantryman
- Display pantry status (read pantry.yaml through the engine)
- Edit pantry (as in: set InStock boolean, which triggers an update of pantry.yaml using the engine)
- Settings
 - Set data_dir (most importantly, use the Android native "Files" navigator or whatever it is called so the user can pick a folder). Once data_dir is set, if it differs from the previous data_dir location, this should trigger one of two things:
   - If the new data_dir is empty, the existing ingredient and pantry YAML files should be moved there
   - If the new data_dir is NOT empty AND is a valid data_dir (at least one correctly formatted ingredient YAML file in a folder called 'ingredients' and a correctly formatted 'pantry.yaml'), Pantryman should instead forget about the existing YAML files, scrub its internal database and instead present the YAML files from the new data_dir
   - Every time you set a new data_dir the new data_dir should become the app default, which it should remember between sessions using whatever logic Android apps use for that.

# Cookbook-GTK
## General
- Markdown support for recipes (read/write)
- Tests. Especially unit tests on the engine
- [WIP] All ingredients should be language-agnostic (how?)

## Recipes
- Substitute ingredients (equal / non-equal)
- Quantity matching is off. Right now it just checks that you have A quantity, not whether it's enough for the recipe

## Knowledge Base
- Think of a way to keep updates coming, and coupling with ingredients...
