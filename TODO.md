
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
- The UI constants were thrown in there rather blindly. May need to be better structured to avoid nasty surprises in the future
- Markdown support for recipes (read/write)
- Tests. Especially unit tests on the engine
- An icon for the window (X is currently givin' it to ya)
- [WIP] All ingredients should be language-agnostic (how?)
- [WIP] Handle pluralization in pantry

## Recipes
- Edit dialog is fugly
- Dropdown ingredient auto-suggest for name_field in 'Edit Recipe' window should trigger immediately
- Substitute ingredients (equal / non-equal)
- Tighter coupling to ingredients (how should we handle recipes created with recipes not known in the pantry?)
  - Right now, once you create the ingredient at some point, the coupling happens by itself. Maybe that's fine?
- Quantity matching is off. Right now it just checks that you have A quantity, not whether it's enough for the recipe

## Pantry
- Move categories filter somewhere else. 
- Way too unoptimized screen real estate in general
- Prettier indication of InStock and unknown quantity
- Error dialog for "Cannot edit ingredient" lacks transient parent despite the code being set in pantry.rs. Investigate at some point

## Knowledge Base
- Think of a way to keep updates coming, and coupling with ingredients...

## Settings
- [WIP] data_dir location and validation
- [WIP] language?

## About
- Done

## Help
- Help docs
