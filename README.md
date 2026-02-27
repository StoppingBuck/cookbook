# Cookbook

A cross-platform recipe and pantry management application.

- **cookbook-gtk** — GTK4 desktop app (Linux/macOS/Windows via WSL)
- **pantryman** — Android companion app for managing your pantry on the go

Data is stored as plain text files (YAML for ingredients/pantry, Markdown for recipes), so it can be synced between devices with any tool you already use — Syncthing, Dropbox, pCloud, Git, etc.

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

Both frontends use the same Rust library (`cookbook-engine`) to read and write data. Point both apps at the same synced folder and they stay in sync automatically.

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

### 1. Rust toolchain (required for everything)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### 2. GTK desktop app (`cookbook-gtk`)

#### Arch Linux
```bash
sudo pacman -S gtk4 libadwaita pkgconf
```

#### Ubuntu / Debian
```bash
sudo apt install -y pkg-config libgtk-4-dev libadwaita-1-dev
```

#### Fedora
```bash
sudo dnf install -y gtk4-devel libadwaita-devel pkgconf
```

#### macOS
```bash
brew install gtk4 libadwaita pkg-config
```

Then run:
```bash
COOKBOOK_DATA_DIR=example/data ./dev.sh gtk
```

### 3. Android app (`pantryman`)

The Android build requires the Android SDK, NDK, and a JDK. The recommended way is **Android Studio**, which bundles the SDK manager and makes NDK installation straightforward. A command-line-only path is also documented below.

---

#### Option A — Android Studio (recommended)

Android Studio is **not in the official Arch Linux repositories**. Install it via an AUR helper:

```bash
# If you have yay:
yay -S android-studio

# If you have paru:
paru -S android-studio
```

Or download the tarball directly from https://developer.android.com/studio, extract it, and run `bin/studio.sh`.

On **Ubuntu/Debian**, use `snap`:
```bash
sudo snap install android-studio --classic
```

On **Fedora**:
```bash
sudo dnf install android-studio   # available in some third-party repos
# or download the tarball from developer.android.com/studio
```

Once Android Studio is installed:

1. **Install the SDK and NDK** via *SDK Manager* (Settings → SDK Manager → SDK Tools tab):
   - Android SDK Platform 35
   - Android SDK Build-Tools
   - NDK (Side by side) — install the **LTS** version
   - Android Emulator (if you want to run on a virtual device)
   - Android SDK Platform-Tools (`adb`)

2. Note your SDK path — it is shown at the top of the SDK Manager. By default it is `~/Android/Sdk` on Linux.

---

#### Option B — Command-line tools only (no Android Studio)

Download the SDK command-line tools from https://developer.android.com/studio#command-line-tools-only. Extract them and install what you need:

```bash
# Extract to ~/Android/Sdk/cmdline-tools/latest/
mkdir -p ~/Android/Sdk/cmdline-tools
unzip commandlinetools-linux-*.zip -d ~/Android/Sdk/cmdline-tools
mv ~/Android/Sdk/cmdline-tools/cmdline-tools ~/Android/Sdk/cmdline-tools/latest

# Add to PATH
export PATH="$HOME/Android/Sdk/cmdline-tools/latest/bin:$HOME/Android/Sdk/platform-tools:$PATH"

# Accept licences then install SDK, NDK, platform-tools, and emulator
sdkmanager --licenses
sdkmanager "platforms;android-35" \
           "build-tools;35.0.0" \
           "ndk;27.2.12479018" \
           "platform-tools" \
           "emulator" \
           "system-images;android-35;google_apis_playstore;x86_64"
```

---

#### Rust targets and cargo-ndk

After installing the SDK/NDK, add the Rust cross-compilation targets and `cargo-ndk`:

```bash
# Physical arm64 phone (most modern Android phones):
rustup target add aarch64-linux-android

# Older arm32 phones:
rustup target add armv7-linux-androideabi

# x86_64 emulator (running on your desktop):
rustup target add x86_64-linux-android

# cargo-ndk: wraps cargo to cross-compile for Android
cargo install cargo-ndk
```

---

#### `local.properties`

Create `pantryman/local.properties` pointing at your SDK:

```properties
sdk.dir=/home/YOUR_USERNAME/Android/Sdk
```

If the file does not exist, Gradle will fail with a cryptic error about the SDK location.

---

#### `ANDROID_NDK_HOME`

`cargo-ndk` needs to know where the NDK is. The NDK is installed inside the SDK directory under `ndk/<version>/`. Set the variable before building:

```bash
# Exact path — replace the version number with the one you installed:
export ANDROID_NDK_HOME=$HOME/Android/Sdk/ndk/27.2.12479018

# Or if you only have one NDK installed, this picks it automatically:
export ANDROID_NDK_HOME=$HOME/Android/Sdk/ndk/$(ls $HOME/Android/Sdk/ndk | tail -1)
```

Add this export to your `~/.bashrc` / `~/.zshrc` so you do not have to type it every time.

---

## Common dev tasks

```bash
./dev.sh gtk              # build and run the GTK desktop app
./dev.sh gtk-compile      # compile only (no run) — use this after every edit
./dev.sh gtk-test         # run all tests for cookbook-gtk
./dev.sh android          # build Rust bridge + Android app, install, run, stream logs
./dev.sh check            # cargo check on all Rust components
./dev.sh test             # run all tests
./dev.sh engine-test      # run cookbook-engine tests (no display needed)
./dev.sh clean            # clean build artifacts
./dev.sh help             # show all commands
```

The `COOKBOOK_DATA_DIR` environment variable overrides the data directory at runtime:
```bash
COOKBOOK_DATA_DIR=/path/to/your/pcloud/cookbook ./dev.sh gtk
```

---

## Running on an Android emulator

An emulator lets you test pantryman without a physical device.

### Create a virtual device

**Via Android Studio** (easiest): open *Device Manager* (the phone icon in the toolbar or *Tools → Device Manager*), click *Create Device*, pick a Pixel profile, select a system image with API 35, and finish.

**Via command line:**
```bash
# List available system images (should show the one you installed above):
avdmanager list target

# Create a Pixel 8-like AVD:
avdmanager create avd \
  --name "Pixel8_API35" \
  --package "system-images;android-35;google_apis_playstore;x86_64" \
  --device "pixel_8"
```

### Start the emulator

```bash
# Make sure the emulator binary is on your PATH:
export PATH="$HOME/Android/Sdk/emulator:$PATH"

emulator -avd Pixel8_API35 &
```

Wait for the boot animation to finish (~30–60 seconds). Then verify `adb` sees it:
```bash
adb devices
# Should show: emulator-5554   device
```

### Build for the emulator and install

The default `dev.sh android` builds only `arm64-v8a` (for physical phones). To target the x86_64 emulator, build that ABI instead:

```bash
cd pantryman/rust-bridge
cargo ndk -t x86_64 -o ../app/src/main/jniLibs build --release
cd ..
./gradlew installDebug
adb shell am start -n com.example.pantryman/.MainActivity
```

Or build all ABIs so the same APK works on both emulator and phone:
```bash
cargo ndk \
  -t arm64-v8a \
  -t armeabi-v7a \
  -t x86_64 \
  -o ../app/src/main/jniLibs \
  build --release
```

---

## Installing on a physical Android phone

### 1. Enable USB debugging on the phone

1. Go to **Settings → About phone**.
2. Tap **Build number** seven times. A message will say "You are now a developer."
3. Go back to **Settings → System → Developer options** (exact location varies by Android version and manufacturer).
4. Enable **USB debugging**.

### 2. Connect and authorise

1. Connect the phone to your computer with a USB cable.
2. On the phone, a prompt will ask *"Allow USB debugging from this computer?"* — tap **Allow** (tick "Always allow from this computer" so you don't have to do this every time).
3. Verify the connection:
   ```bash
   adb devices
   # Should show your device serial number with status "device" (not "unauthorized")
   ```

### 3. Install

Build and install in one step:
```bash
./dev.sh android
```

Or manually:
```bash
cd pantryman/rust-bridge
cargo ndk -t arm64-v8a -o ../app/src/main/jniLibs build --release
cd ..
./gradlew installDebug
adb shell am start -n com.example.pantryman/.MainActivity
```

The APK is also available at `pantryman/app/build/outputs/apk/debug/app-debug.apk` if you want to copy it to the phone and install it with a file manager (you will need to enable *Install unknown apps* for your file manager in Settings → Apps → Special app access).

---

## Troubleshooting

**`cargo-ndk: command not found`**
Run `cargo install cargo-ndk` and make sure `~/.cargo/bin` is on your `PATH`.

**`ANDROID_NDK_HOME` not set / NDK not found**
Set `ANDROID_NDK_HOME` to the full path of your NDK directory, e.g. `~/Android/Sdk/ndk/27.2.12479018`. It must point directly to the versioned folder, not the `ndk/` parent.

**`sdk.dir` missing or wrong in `local.properties`**
Create `pantryman/local.properties` with the line `sdk.dir=/home/you/Android/Sdk`. The path must not end with a slash.

**`adb: no devices/emulators found`**
- Check the USB cable (data cables vs charge-only cables).
- Make sure USB debugging is enabled and you accepted the authorisation prompt on the phone.
- Try a different USB port.
- Run `adb kill-server && adb start-server` to restart the ADB daemon.

**Emulator is very slow**
- Make sure hardware acceleration (KVM on Linux) is enabled: `ls -la /dev/kvm` — you should be in the `kvm` group (`sudo usermod -aG kvm $USER`, then log out and back in).
- On Arch: `sudo pacman -S qemu-desktop` provides KVM support.

**GTK app won't start on WSL**
You need a working X/Wayland server (WSLg covers this on Windows 11; on Windows 10 use VcXsrv or X410).

**Missing GTK libraries**
Make sure you installed the `-dev` / `-devel` packages, not just the runtime libraries.
