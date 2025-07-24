# Cookbook Project - Development Environment Setup

This guide will help you set up your development environment for the Cookbook project, including both the GTK desktop frontend (`cookbook-gtk`) and the Android app (`pantryman`).

## Table of Contents
- [General Requirements](#general-requirements)
- [Platform-specific Setup](#platform-specific-setup)
  - [Windows 11 + WSL (Oracle Linux 9)](#windows-11--wsl-oracle-linux-9)
  - [Oracle Linux 9](#oracle-linux-9)
  - [Ubuntu/Debian](#ubuntudebian)
  - [Fedora](#fedora)
  - [Arch Linux](#arch-linux)
  - [NixOS](#nixos)
  - [macOS](#macos)
- [Android Development](#android-development)
- [GTK Desktop Development](#gtk-desktop-development)
- [Common Issues](#common-issues)

---

## General Requirements

- **Rust toolchain** (cargo, rustc)
- **GTK4 development libraries**
- **Android SDK & NDK** (for Pantryman)
- **Java (JDK 21+)**
- **Gradle**
- **Kotlin**
- **Git**
- **Python 3** (for some scripts)

## Platform-specific Setup

### Windows 11 + WSL (Oracle Linux 9)

1. **Update your system:**
   ```bash
   sudo dnf update -y
   ```
2. **Install core dependencies:**
   ```bash
   sudo dnf install -y git python3 make gcc gcc-c++ pkgconf-pkg-config
   ```
3. **Install Rust:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   
   To enable cross-compiling to Android:
   rustup target add aarch64-linux-android
   ```
4. **Install GTK4 development libraries:**
   ```bash
   sudo dnf install -y gtk4-devel gdk-pixbuf2-devel glib2-devel xz
   ```
5. **Install Java (JDK 21+):**
   ```bash
   sudo dnf install -y java-21-openjdk java-21-openjdk-devel
   ```
6. **Install Gradle (via SDKMAN!):**
   ```bash
   # Ensure zip and unzip are installed (required by SDKMAN!) and also adb
   sudo dnf install -y unzip zip android-tools
   # Install SDKMAN!
   curl -s "https://get.sdkman.io" | bash
   source "$HOME/.sdkman/bin/sdkman-init.sh"
   # Install Gradle
   sdk install gradle
   ```
7. **Android SDK & NDK:**
   - **Install Android Studio in OL9/WSL:**
     ```bash
     # Download the latest Android Studio .tar.gz for Linux from:
     # https://developer.android.com/studio
     # Example (update version as needed):
     wget https://redirector.gvt1.com/edgedl/android/studio/ide-zips/2025.1.1.14/android-studio-2025.1.1.14-linux.tar.gz
     tar -xzvf android-studio-*-linux.tar.gz
     mv android-studio ~/android-studio
     # Launch Android Studio (requires a working X11 setup)
     ~/android-studio/bin/studio.sh &
     ```
     - Follow the setup wizard to install the SDK and NDK inside WSL.
     - If you are in WSL, make sure you have WSLg or another X Server running in order for Pantryman to render in Windows.
   - create pantryman/local.properties with a line to where you put the SDK: sdk.dir=/home/yourusername/Android/Sdk

8. If running WSL:
1. Run ADB on Windows and Connect from WSL

### Oracle Linux 9

Same as above (see WSL instructions).

### Ubuntu/Debian

```bash
sudo apt update
sudo apt install -y git python3 make gcc g++ pkg-config libgtk-4-dev libgdk-pixbuf-2.0-dev openjdk-21-jdk gradle
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```
- For Android: Install Android Studio or SDK/NDK manually.

### Fedora

```bash
sudo dnf install -y git python3 make gcc gcc-c++ pkgconf-pkg-config gtk4-devel gdk-pixbuf2-devel java-21-openjdk java-21-openjdk-devel gradle
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```


### Arch Linux

1. **Update your system:**
   ```bash
   sudo pacman -Syu
   ```
2. **Install core dependencies:**
   ```bash
   sudo pacman -S git python rust gcc pkgconf make
   ```
3. **Install GTK4 development libraries:**
   ```bash
   sudo pacman -S gtk4 gdk-pixbuf2 glib2 xz
   sudo pacman -S gtk3 (for UI testing framework... maybe?)
   sudo pacman -S noto-fonts-emoji (or another font that supports emojis)
   sudo pacman -S gnome-themes-extra (for dark theme support)
   ```
4. **Install Java (JDK 21+):**
   ```bash
   sudo pacman -S jdk-openjdk
   ```
5. **Install Gradle:**
   ```bash
   sudo pacman -S gradle
   ```
6. **Android SDK & NDK:**
   - **Install Android Studio:**
     ```bash

   Install android-ndk from the AUR (or through the Studio)

     # Download the latest Android Studio .tar.gz for Linux from:
     # https://developer.android.com/studio
     # Example (update version as needed):
     wget https://redirector.gvt1.com/edgedl/android/studio/ide-zips/2025.1.1.14/android-studio-2025.1.1.14-linux.tar.gz
     tar -xzvf android-studio-*-linux.tar.gz
     mv android-studio ~/android-studio
     # Launch Android Studio (requires a working X11 setup)
     ~/android-studio/bin/studio.sh &
     ```
     - Follow the setup wizard to install the SDK and NDK.
     - If you are in WSL, make sure you have WSLg or another X Server running in order for Pantryman to render in Windows.

- For Rust: After install, run:
  ```bash
  source $HOME/.cargo/env
  ```

### NixOS

Add the following to your `configuration.nix` or use a shell.nix:
```nix
{ pkgs ? import <nixpkgs> {} }:
  pkgs.mkShell {
    buildInputs = [
      pkgs.rustc pkgs.cargo pkgs.gtk4 pkgs.gdk-pixbuf pkgs.pkg-config pkgs.openjdk pkgs.gradle pkgs.git pkgs.python3
    ];
  }
```
- For Android: Use Android Studio from `pkgs.androidStudioPackages.stable`.

### macOS

```bash
brew install git python rust gtk4 gdk-pixbuf gradle openjdk
```
- For Android: Install Android Studio from official site.

---

## Android Development

- **Android Studio** (recommended)
- **SDK/NDK**: Required for JNI bridge and building Rust for Android
- **Gradle**: Used for building Pantryman
- **JDK 21+**
- **Kotlin**

## GTK Desktop Development

- **GTK4 development libraries**
- **Rust toolchain**
- **Relm4** (Rust crate, installed via Cargo)
- **gdk-pixbuf**
- **gettext**

## Common Issues

- If you get errors about missing GTK libraries, ensure you have the `-dev` or `-devel` packages installed for your distro.
- For Android builds, ensure your NDK path is set and the correct ABI filters are configured.
- On WSL, you may need to set up X11 forwarding to run GTK apps. Use an X server like [VcXsrv](https://sourceforge.net/projects/vcxsrv/) or [X410](https://x410.dev/).

---

## Quick Start

1. **Clone the repository:**
   ```bash
   git clone https://github.com/StoppingBuck/cookbook.git
   cd cookbook
   ```
2. **Install dependencies (see above for your OS).**
3. **Run development helper script:**
   ```bash
   ./dev.sh help
   ```

---

## Questions?
If you run into issues, check the [DEVELOPMENT.md](DEVELOPMENT.md) for troubleshooting and workflow tips.

---

## Maintainers
StoppingBuck and contributors

---
