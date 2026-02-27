# Cookbook (archived)

> **This repository has been archived.** Development has moved to three focused repos:
>
> - [janus-engine](https://github.com/StoppingBuck/janus-engine) — shared Rust engine
> - [pantryman-linux](https://github.com/StoppingBuck/pantryman-linux) — GTK4 desktop app
> - [pantryman-android](https://github.com/StoppingBuck/pantryman-android) — Android app

---

A cross-platform recipe and pantry manager built on the principle that **your data belongs to you**.

---

## Vision

*[This section is yours to write — your words on what makes this different.]*

The short version: Cookbook stores everything as plain text files. There is no account to create, no server to trust, no subscription to pay, and no proprietary format to escape from later. You own your data in the most literal sense — it lives on your disk as YAML and Markdown files you can read, edit, and back up with any tool you already use.

Most recipe and pantry apps are either walled-garden mobile apps that lock you in, or complex self-hosted servers that require a Raspberry Pi and a weekend to set up. Cookbook is neither. It is a small, fast, offline-first application that syncs the same way your notes or dotfiles do — through a folder.

---

## Features (v0.1.0)

### cookbook-gtk — Desktop app (Linux)
- Browse, create, edit, and delete recipes (Markdown with YAML frontmatter)
- Pantry management: track what you have in stock with quantities and units
- Filter recipes by available ingredients
- Ingredient library with categories and tags
- Knowledge Base for culinary notes and technique articles
- Folder picker to point the app at any local or cloud-synced directory
- Light/dark/system theme

### pantryman — Android companion app
- View and manage your pantry on the go
- Add new ingredients and update stock quantities
- Bidirectional sync with a cloud folder via Android's Storage Access Framework (SAF) — works with pCloud, Google Drive, Nextcloud, and any other SAF-compatible provider
- Automatic sync on app open (pull) and app close (push)
- Manual "Sync Now" button in settings

### cookbook-engine — Shared Rust library
- Single implementation of all data reading and writing, shared between both frontends
- Plain-file storage: `ingredients/*.yaml`, `pantry.yaml`, `recipes/*.md`, `kb/*.md`
- No database, no migrations, no lock-in

---

## How it works

```
cookbook-gtk (GTK4/Relm4)       pantryman (Android/Kotlin)
         |                                |
         |                         JNI bridge
         |                                |
         └──────── cookbook-engine ───────┘
                   (Rust library)
                         |
              YAML + Markdown files
              (your local or synced folder)
```

Both frontends are thin shells over `cookbook-engine`. The engine handles all file I/O and business logic; the frontends handle only display and user interaction.

**Sync model:**
- **Desktop:** point cookbook-gtk directly at your cloud folder (pCloud, Syncthing, Dropbox, etc.) — it reads and writes there directly.
- **Android:** Pantryman keeps a local working copy and mirrors it to/from a SAF cloud folder on every open and close. The cloud folder is the shared truth; local storage is a cache.

---

## Quick start

```bash
# Clone
git clone https://github.com/yourname/cookbook.git
cd cookbook

# Run the desktop app against the bundled example data
./dev.sh gtk

# Or point it at your own data folder
COOKBOOK_DATA_DIR=/path/to/your/data ./dev.sh gtk
```

For full development environment setup (Android SDK, NDK, dependencies by distro) see [CONTRIBUTING.md](CONTRIBUTING.md).

---

## Data format

```
data/
├── ingredients/
│   ├── potato.yaml
│   └── tomato.yaml
├── recipes/
│   └── Lasagna.md
├── kb/
│   └── potato.md
└── pantry.yaml
```

See [CONTRIBUTING.md](CONTRIBUTING.md#data-format) for the full schema reference.

---

## Project status

**v0.1.0 — first working beta.** Core functionality is solid. See [TODO.md](TODO.md) for known issues and planned work, and [CHANGELOG.md](CHANGELOG.md) for release history.

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## AI

See [AI.md](AI.md) for how AI tools were used in this project and what that means for contributors.

## License

Apache 2.0 — see [LICENSE-APACHE](LICENSE-APACHE).
