# Contributing to Cookbook

Thank you for your interest in contributing. This document covers everything you need to get a working development environment and submit changes.

---

## Table of contents

1. [Architecture overview](#architecture-overview)
2. [Data format reference](#data-format-reference)
3. [Development setup](#development-setup)
   - [Rust toolchain](#1-rust-toolchain)
   - [GTK desktop app](#2-gtk-desktop-app)
   - [Android app](#3-android-app)
4. [Common dev tasks](#common-dev-tasks)
5. [Code conventions](#code-conventions)
6. [Submitting changes](#submitting-changes)

---

## Architecture overview

```
cookbook/
├── cookbook-engine/     # Rust library — all business logic and file I/O
├── cookbook-gtk/        # GTK4 desktop app (Rust + Relm4)
└── pantryman/
    ├── app/             # Android app (Kotlin)
    └── rust-bridge/     # cdylib crate that exposes engine over JNI
```

**The strict rule:** business logic lives in `cookbook-engine`. The frontends are UI only. Never add file I/O or domain logic to `cookbook-gtk` or `pantryman`.

`rust-bridge` is excluded from the main Cargo workspace because it is a `cdylib` targeting Android ABIs, not the host. Check it separately with `cargo check` inside `pantryman/rust-bridge/`.

---

## Data format reference

```
data/
├── ingredients/<slug>.yaml
├── recipes/<Title>.md
├── kb/<slug>.md
└── pantry.yaml
```

**Ingredient** (`ingredients/potato.yaml`):
```yaml
name: potato
slug: potato
category: vegetable
kb: potato            # optional — links to a KB article by slug
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

**KB article** (`kb/potato.md`):
```markdown
---
slug: potato
title: Potato
---

The potato (Solanum tuberosum) is a starchy root vegetable...
```

**Never match KB entries by filename — always use the `slug` field.**

---

## Development setup

### 1. Rust toolchain

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### 2. GTK desktop app

Install GTK4 and libadwaita dev packages for your distro:

| Distro | Command |
|--------|---------|
| Arch Linux | `sudo pacman -S gtk4 libadwaita pkgconf` |
| Ubuntu/Debian | `sudo apt install -y pkg-config libgtk-4-dev libadwaita-1-dev` |
| Fedora | `sudo dnf install -y gtk4-devel libadwaita-devel pkgconf` |
| macOS | `brew install gtk4 libadwaita pkg-config` |

Then:
```bash
./dev.sh gtk                        # build and run against example data
COOKBOOK_DATA_DIR=/your/data ./dev.sh gtk   # run against your own data
```

**Compile loop:** after every edit to `cookbook-gtk`, run `./dev.sh gtk-compile` immediately and fix all errors before running again.

### 3. Android app

#### Requirements
- Android SDK (API 35+) and NDK
- JDK 21+
- A physical device with USB debugging enabled (recommended), or an x86_64 emulator

#### Recommended: Android Studio

Install via your distro's package manager or from [developer.android.com/studio](https://developer.android.com/studio). Use the SDK Manager to install:
- Android SDK Platform 35
- NDK (Side by side) — any recent LTS version
- SDK Platform-Tools (`adb`)

#### Command-line only

```bash
# Download cmdline-tools from developer.android.com/studio#command-line-tools-only
mkdir -p ~/Android/Sdk/cmdline-tools
unzip commandlinetools-linux-*.zip -d ~/Android/Sdk/cmdline-tools
mv ~/Android/Sdk/cmdline-tools/cmdline-tools ~/Android/Sdk/cmdline-tools/latest
export PATH="$HOME/Android/Sdk/cmdline-tools/latest/bin:$HOME/Android/Sdk/platform-tools:$PATH"

sdkmanager --licenses
sdkmanager "platforms;android-35" "build-tools;35.0.0" "ndk;27.2.12479018" "platform-tools"
```

#### Rust targets and cargo-ndk

```bash
rustup target add aarch64-linux-android   # physical arm64 phones
rustup target add x86_64-linux-android    # x86_64 emulator
cargo install cargo-ndk
```

#### local.properties

Create `pantryman/local.properties`:
```properties
sdk.dir=/home/YOUR_USERNAME/Android/Sdk
```

#### ANDROID_NDK_HOME

```bash
export ANDROID_NDK_HOME=$HOME/Android/Sdk/ndk/$(ls $HOME/Android/Sdk/ndk | tail -1)
```

Add to your `~/.bashrc` or `~/.zshrc`.

#### Enable USB debugging on your phone

1. Settings → About phone → tap **Build number** seven times
2. Settings → System → Developer options → enable **USB debugging**
3. Connect via USB, accept the prompt on the phone
4. Verify: `adb devices` — should show your device as `device` (not `unauthorized`)

---

## Common dev tasks

```bash
./dev.sh gtk                # build and run GTK app
./dev.sh gtk-compile        # compile only — use after every edit
./dev.sh gtk-test           # run GTK tests (requires display)
./dev.sh gtk-test-headless  # run GTK tests via xvfb-run
./dev.sh android            # build Rust bridge + APK, install on device, stream logs
./dev.sh engine-test        # run cookbook-engine tests (fast, no display)
./dev.sh check              # cargo check on all Rust workspace members
./dev.sh test               # run all tests
./dev.sh clean              # clean all build artifacts
```

Single test:
```bash
cargo test -p cookbook-engine <test_name>
cargo test -p cookbook-gtk <test_name>
```

Note: `pantryman/rust-bridge` is excluded from the workspace. Check it separately:
```bash
cd pantryman/rust-bridge && cargo check
```

---

## Code conventions

- **Strict separation:** business logic in `cookbook-engine`, UI only in frontends.
- **Rust naming:** `snake_case` for files, structs, functions.
- **Android:** idiomatic Kotlin with Jetpack libraries; FFI calls only through `rust-bridge`.
- **Error handling:** propagate errors to the UI — toasts/dialogs. Don't silently swallow errors.
- **Data migration:** use `version:` fields in YAML; don't implement migrations until required.
- **Logging:** use `log::debug!` / `log::info!` / `log::error!` in Rust. No `println!` in production code. Android: use `Log.d/e/w` with the `TAG` constant.
- **No speculative abstractions:** solve the problem in front of you. Three similar lines of code is better than a premature abstraction.

---

## Submitting changes

1. Fork the repository and create a branch from `main`.
2. Keep commits focused — one logical change per commit.
3. If your change touches `cookbook-engine`, add or update tests in `cookbook-engine/tests/`.
4. Run `./dev.sh check` and `./dev.sh engine-test` before pushing.
5. Open a pull request with a clear description of what and why.

There is no formal PR template yet. Use common sense.

---

## Troubleshooting

**`cargo-ndk: command not found`** — run `cargo install cargo-ndk` and ensure `~/.cargo/bin` is on `$PATH`.

**`ANDROID_NDK_HOME` not set** — set it to the full versioned path, e.g. `~/Android/Sdk/ndk/27.2.12479018`.

**`sdk.dir` missing** — create `pantryman/local.properties` with `sdk.dir=/home/you/Android/Sdk`.

**`adb: no devices found`** — check the USB cable (data vs charge-only), USB debugging toggle, and the authorisation prompt on the phone. Try `adb kill-server && adb start-server`.

**GTK app: `Failed to register: Timeout was reached`** — a previous instance is still running (possibly stuck in uninterruptible sleep on a FUSE mount). Find and kill it: `pkill -9 cookbook-gtk`.

**GTK app won't start on WSL** — you need a working X/Wayland server (WSLg on Windows 11; VcXsrv or X410 on Windows 10).
