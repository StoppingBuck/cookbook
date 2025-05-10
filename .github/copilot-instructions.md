> This file exists to help GitHub Copilot understand the structure, purpose and intended behavior of the `cookbook` project. It may also help human collaborators get oriented faster.


# Quick note about the Agent mode
Whenever you execute commands in Agent mode, sometimes the process will 'hang' - either because it is supposed to stay active or because the command just doesn't exit properly. Either way, that often results in you hanging. 
In those cases, I will use CTRL+C to kill the process and 'free' you. Thus, when analyzing terminal output, ignore anything related to CTRL+C behavior when seen at the end of a command's output (that would be me, killing the process to free you to continue your work)

# Project: Cookbook
Cookbook is a cross-platform recipe and pantry manager, as well as wiki. It is split into multiple crates:

## 1. `cookbook-engine`
This is the core library, written in Rust. It contains all business logic:
- Managing recipes and ingredients (add, edit, delete, list)
- Searching and tagging
- Saving/loading data (from YAML and Markdown with YAML frontmatter)
- No GUI. Just pure logic.

It is designed to be usable from multiple frontends via FFI.

### Naming
pub struct Ingredient {
    pub name: String,
    pub quantity: Option<String>,
    // ...
}

pub struct Pantry {
    pub ingredients: Vec<Ingredient>
}

In essence, the pantry is a collection of ingredients (with optional future inclusions of other types, like 'PreparedDish' or whatever).

### Reading from files
The engine has no 'database' in the traditional sense. Instead, it utilizes the same approach as Obsidian, where all data is stored in text files (Markdown for recipes, YAML for everything else).
It should expect a data directory with a specific structure, where files are formatted in specific formats:
data/
├── ingredients/
│   ├── potato.yaml
│   └── egg.yaml
├── kb/
│   ├── potato.md
│   └── potato.jpg
├── pantry.yaml
└── recipes/
    └── lasagna.md

The engine may eventually support data migrations, so it is recommended to include a version field in pantry.yaml or other global state files.

An example data folder is included in the repository as example/data/. This folder contains example files for each type of data, and should be imported as the default data folder when the engine is first run. The engine should be able to read and write to this folder, and should be able to parse the data in it. It should also be able to handle errors gracefully, such as when a file is missing or has invalid data.

#### Ingredients
Each ingredient YAML file should contain the following:
```yaml
name: potato
category: vegetable
kb: potato # slug referring to a knowledge base entry in kb/ - may be null or blank
tags: [vegetable, starch]
```

#### Pantry
The pantry file uses this format:
```yaml
version: 1
items:
  - ingredient: potato
    quantity: 2
    quantity_type: kg
    last_updated: 2025-05-10
  - ingredient: tomato
    quantity: null # or empty string - both mean 'I don't know how much I have')
    quantity_type: null
    last_updated: 2025-05-10
```
The category, tags, etc. of an ingredient is then pulled from the data/ingredients/ directory. The name of a pantry item should equal the name of the ingredient.

#### Recipes
The recipe files are in Markdown format with YAML frontmatter:

An example recipe file looks like this:
```yaml
---
Title: Lasagna
Ingredients:
  - ingredient: potato
    quantity: 2
    quantity_type: kg
  - ingredient: tomato
    quantity: 1
    quantity_type: kg
PrepTime: 30
Downtime: 60
Servings: 4
Tags: [dinner, pasta]
---
Start by boiling the potatoes. Then, layer them with the tomatoes and cheese in a baking dish. Bake for 60 minutes at 180C.
```

As you can see, the file is a mix of YAML and Markdown. The YAML part is used to store structured data, while the rest is free-form text, and should be treated as an Instructions section for the recipe, complete with any markdown formatting the user chooses. 
The engine should be able to parse this format and extract the relevant information.

Note: We use two time fields:
- `PrepTime`: Time where the user is actively working (chopping, stirring, etc.)
- `Downtime`: Time where the recipe requires waiting, but no attention (oven, resting, etc.)

This makes it easier to distinguish between physical effort and elapsed duration.

#### KB (Knowledge Base)
The knowledge base section is its own separate small (but growing) database of notes about food, focusing on information like nutrition, history, and other interesting facts. These are meant to be curated by the cookbook developers centrally and then pushed to the cookbook users (so not editable by the user). Just like recipes, they are a mix of YAML frontmatter and Markdown. 
Each KB note should include a `slug` field in its YAML frontmatter. This slug is used to connect ingredients to their corresponding knowledge base entry via the `kb:` field. The engine should never rely on file name or title matching to find the correct KB note – it must always match on slug. Multiple ingredients can point to the same KB note. KB notes do not point to Ingredients, only the other way around.

Here is an example of a kb note:

```markdown
---
slug: potato # slug for linking to this note from Ingredient files
title: Potato
image: potato.jpg # image file should be in data/kb/
---

### Potato
Potatoes are root vegetables that are native to the Andes mountains in South America. They are a staple food in many cultures and are known for their versatility in cooking.
They are rich in carbohydrates and provide a good source of energy. Potatoes can be prepared in many ways, including boiling, baking, frying, and mashing. They are often used as a side dish or as an ingredient in soups, stews, and casseroles.
```

KB notes can be linked to by slug in the ingredient YAML files. The engine should be able to parse these files and extract the relevant information.

## 2. `cookbook-gtk`
The first GUI frontend, targeting GTK users. It is a simple GTK application that uses the `cookbook-engine` library to manage recipes and pantry items and more.
It is written using gtk4-rs and Relm4 in order to best interact with the Rust cookbook-engine without need for FFI hacks.

It should have a modern UI, with a sidebar for navigation and a main area for displaying recipes and pantry items. The sidebar should have sections for:
- Recipes
- Pantry
- Knowledge Base
- Settings
- About
- Help
The main area should display the selected item, with options to edit, delete, and add new items. The UI should be responsive and work well on different screen sizes.
The application should be able to load and save data from the `cookbook-engine` library (pantry, ingredients, recipes - NOT KB notes), and should be able to display recipes and pantry items in a user-friendly way.
The application should also have a search function, allowing users to search for recipes and pantry items by name or tag. The search function should be fast and responsive, and should update the displayed items in real-time as the user types.
The application should also have a settings page, allowing users to configure the application to their liking. The settings page should include options for:
- Changing the theme (light/dark)
- Changing the font size
- Changing the data directory (default is the example/data/ directory in the repository, but users should be able to change it to any directory they want)
- Changing the language (English-only for right now, transifex later)
The application should also have an about page, displaying information about the application, including the version number, license, and credits. The about page should also include a link to the project's website and a link to the project's GitHub repository.
The application should also have a help page, displaying information about how to use the application, including a user guide and a FAQ section. The help page should also include a link to the project's website and a link to the project's GitHub repository.
The application should also have a feedback page, allowing users to report bugs and suggest features. The feedback page should include a link to the project's GitHub repository and a link to the project's website.

## 3. `cookbook-qt`
A coming  GUI frontend, targeting KDE (Plasma) desktop users. Not yet implemented.

## 4. `cookbook-shared`
This is types or utils that are shared between engine and GUI or between different GUIs, to avoid code duplication.

## Code Style & Expectations
- Favor clean separation of concerns: logic in `engine`, UI in frontend.
- Keep interfaces predictable and ergonomic for GUI devs.
- Prefer small, focused functions over large ones.
- Prefer best practices - no dirty hacks.

## Naming
- Use snake_case in Rust.
- Use PascalCase in Qt/C++.
- Consistent naming: `Recipe`, `Ingredient`, etc.