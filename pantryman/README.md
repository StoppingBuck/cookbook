# Pantryman

Android companion app for [Cookbook](../README.md). Lets you manage your pantry on the go — update what ingredients you have in stock while shopping, and have those changes automatically available on your desktop when you get home (via Syncthing, Dropbox, etc.).

## Features

- View all ingredients and their pantry status (in stock / out of stock, quantity)
- Add, edit, and delete ingredients
- Update pantry quantities
- Filter ingredients by category
- Change the data directory at runtime without restarting the app
- Data persists as YAML files — the same files read by cookbook-gtk

## Architecture

```
Kotlin UI (MainActivity, SettingsActivity)
        ↓
CookbookEngine.kt  ←  JNI  →  pantryman/rust-bridge/
                                        ↓
                              cookbook-engine (Rust)
                                        ↓
                              YAML / Markdown files
```

The Rust library `cookbook-engine` is compiled as a `.so` shared library (`libpantryman_bridge.so`) via `cargo-ndk` and bundled into the APK. `CookbookEngine.kt` is the Kotlin wrapper that calls into it via JNI.

## Building

### Prerequisites

- Android Studio or Android SDK + NDK
- JDK 21+
- Gradle
- Rust with the `aarch64-linux-android` target: `rustup target add aarch64-linux-android`
- `cargo-ndk`: `cargo install cargo-ndk`
- `pantryman/local.properties` with your SDK path:
  ```
  sdk.dir=/home/<you>/Android/Sdk
  ```

### Full build (recommended)

From the repo root:
```bash
./dev.sh android
```
This builds the Rust JNI bridge, assembles the APK, installs it to a connected device, launches it, and streams logcat output.

### Manual steps

```bash
# 1. Build the Rust JNI shared library
cd pantryman/rust-bridge
cargo ndk -t arm64-v8a -o ../app/src/main/jniLibs build --release

# 2. Build and install the APK
cd ..
gradle installDebug
```

## Data directory

By default, data is stored in the app's internal storage under `cookbook_data/`. You can change this in Settings to point at a shared folder (e.g., a Syncthing-synced directory).

When you change the data directory:
- If the new directory is **empty**, the current ingredient and pantry files are migrated there.
- If the new directory **already has valid data** (an `ingredients/` folder and a `pantry.yaml`), the app switches to using it and leaves the old directory untouched.

The selected data directory is persisted across app restarts via `SharedPreferences`.

## Development notes

- The JNI bridge is a separate Cargo crate (`pantryman/rust-bridge/`) excluded from the main workspace. Run `cargo check` inside that directory separately.
- Log output is prefixed with tag `MainActivity` / `CookbookEngine` — filter with `adb logcat -s MainActivity:D CookbookEngine:D`.
- Sample data bundled in `app/src/main/assets/` is only copied to the data directory on the very first launch (empty directory). Subsequent launches leave existing data untouched.
