## General
- The UI constants were thrown in there rather blindly. May need to be better structured to avoid nasty surprises in the future
- UI looks like butt. Adawaita can only do so much
- Markdown support for recipes (read/write) and knowledge base (read-only)
- Tests. Especially unit tests on the engine
- An icon for the window (X is currently givin' it to ya)
- Translations! All ingredients should be language-agnostic (how?)

## Recipes
- Dropdown ingredient auto-suggest for name_field in 'Edit Recipe' window should trigger immediately
- Substitute ingredients (equal / non-equal)

## Pantry
- Error dialog for "Cannot edit ingredient" lacks transient parent despite the code being set in pantry.rs. Investigate at some point

## Knowledge Base
- Expand
- Think of a way to keep updates coming, and coupling with ingredients...

## Settings
- data_dir location and validation

## About
- About + license (MIT engine, GPL app)

## Help
- Help docs
