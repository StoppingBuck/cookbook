# Cookbook

A cross-platform recipe and pantry management application.

- **cookbook-gtk** — GTK4 desktop app (Linux/macOS/Windows via WSL)
- **pantryman** — Android companion app for managing your pantry on the go

Data is stored as plain text files (YAML for ingredients/pantry, Markdown for recipes), so it can be synced between devices with any tool you already use — Syncthing, Dropbox, Git, etc.

---

## How it works

```
Desktop (cookbook-gtk)          Mobile (pantryman)
        |                               |
        └──── cookbook-engine ──────────┘
                    |
           YAML / Markdown files
           (your sync folder)
```

Both frontends use the same Rust library (`cookbook-engine`) to read and write data. You point both apps at the same synced folder and they stay in sync automatically.

**Typical workflow:**
1. Use pantryman on your phone while shopping to update what you have in stock.
2. Your sync tool keeps the data folder up to date on your desktop.
3. Open cookbook-gtk on your desktop to browse recipes based on what you currently have.

---

## Data format

All data lives in a directory with this structure:

```
data/
├── ingredients/
│   ├── potato.yaml
│   ├── tomato.yaml
│   └── ...
├── recipes/
│   ├── Lasagna.md
│   └── ...
├── kb/                  ← optional knowledge base articles
│   └── potato.md
└── pantry.yaml
```

**Ingredient** (`ingredients/potato.yaml`):
```yaml
name: potato
slug: potato
category: vegetable
kb: potato            # optional link to a KB article
tags: [vegetable, starch]
```

**Pantry** (`pantry.yaml`):
```yaml
version: 1
items:
  - ingredient: potato
    quantity: 2
    quantity_type: kg
    last_updated: 2025-05-10
```

**Recipe** (`recipes/Lasagna.md`):
```markdown
---
Title: Lasagna
Ingredients:
  - ingredient: potato
    quantity: 2
    quantity_type: kg
PrepTime: 30
Downtime: 60
Servings: 4
Tags: [dinner, pasta]
---

Start by boiling the potatoes...
```

---

## Development setup

See the platform-specific dependency installation instructions below, then use `./dev.sh` for all common tasks.

### Quick start

```bash
git clone https://github.com/StoppingBuck/cookbook.git
cd cookbook
./dev.sh gtk          # build and run the desktop app
```

### Dependencies

| Component | Required |
|-----------|---------|
| Rust toolchain (cargo, rustc) | All |
| GTK4 development libraries | cookbook-gtk |
| JDK 21+, Gradle, Kotlin | pantryman |
| Android SDK + NDK | pantryman |
| cargo-ndk | pantryman Rust bridge |

#### Arch Linux
```bash
sudo pacman -S git rust gcc pkgconf make gtk4 gdk-pixbuf2 glib2 jdk-openjdk gradle
sudo pacman -S noto-fonts-emoji gnome-themes-extra   # optional: emoji + dark theme support
```

#### Ubuntu / Debian
```bash
sudo apt install -y git make gcc g++ pkg-config libgtk-4-dev libgdk-pixbuf-2.0-dev openjdk-21-jdk gradle
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### Fedora / Oracle Linux / RHEL
```bash
sudo dnf install -y git python3 make gcc gcc-c++ pkgconf-pkg-config gtk4-devel gdk-pixbuf2-devel java-21-openjdk java-21-openjdk-devel gradle
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### NixOS
```nix
{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  buildInputs = [
    pkgs.rustc pkgs.cargo pkgs.gtk4 pkgs.gdk-pixbuf pkgs.pkg-config
    pkgs.openjdk pkgs.gradle pkgs.git pkgs.python3
  ];
}
```

#### macOS
```bash
brew install git rust gtk4 gdk-pixbuf gradle openjdk
```

#### Windows (WSL)
Use the Oracle Linux / RHEL instructions inside WSL. For Android development, install Android Studio inside WSL and run ADB on the Windows host — see [Android development](#android-development) below.

---

## Common dev tasks

```bash
./dev.sh gtk              # build and run the GTK desktop app
./dev.sh gtk-compile      # compile only (no run) — use this after every edit
./dev.sh gtk-test         # run all tests for cookbook-gtk
./dev.sh android          # build Rust bridge + Android app, install, run, stream logs
./dev.sh check            # cargo check on all Rust components
./dev.sh test             # run all tests
./dev.sh clean            # clean build artifacts
./dev.sh help             # show all commands
```

The `COOKBOOK_DATA_DIR` environment variable overrides the data directory at runtime:
```bash
COOKBOOK_DATA_DIR=/path/to/your/data ./dev.sh gtk
```

---

## Android development

### Prerequisites
- Android Studio (recommended) or SDK + NDK installed manually
- `cargo-ndk` (`cargo install cargo-ndk`)
- `rustup target add aarch64-linux-android`
- Create `pantryman/local.properties` with your SDK path:
  ```
  sdk.dir=/home/<you>/Android/Sdk
  ```

### Building
```bash
./dev.sh android          # full cycle: build Rust bridge, build APK, install, run
```

Or step by step:
```bash
# 1. Build the Rust JNI library
cd pantryman/rust-bridge
cargo ndk -t arm64-v8a -o ../app/src/main/jniLibs build --release

# 2. Build and install the Android app
cd ../..
cd pantryman && gradle installDebug
```

### WSL + Windows ADB
If you are on WSL and your Android device is connected to Windows, run ADB on the Windows host and connect to it from WSL. The `dev.sh` script expects `adb.exe` at `/mnt/c/Android/platform-tools/adb.exe` — adjust the `ADB` variable in `dev.sh` if your path differs.

---

## Troubleshooting

**Missing GTK libraries:** Make sure you installed the `-dev` / `-devel` packages, not just the runtime libraries.

**Android NDK not found:** Set `ANDROID_NDK_HOME` to your NDK path before running `cargo ndk`.

**GTK app won't start in WSL:** You need a working X server (WSLg, VcXsrv, or X410).

**App data is reset on every launch (Android):** This was a known bug that has been fixed — the app now only copies the bundled sample data on first launch (empty data directory). If you are seeing this, make sure you are running the latest build.
