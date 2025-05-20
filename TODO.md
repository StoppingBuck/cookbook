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
