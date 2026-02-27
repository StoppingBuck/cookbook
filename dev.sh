#!/bin/bash

# Cookbook Development Helper Script
# Provides common development tasks for the cookbook project

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color


# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PANTRYMAN_DIR="$PROJECT_ROOT/pantryman"
APP_PACKAGE="com.example.pantryman"
ADB="$HOME/Android/Sdk/platform-tools/adb"

echo -e "${BLUE}🍳 Cookbook Development Helper${NC}"
echo "=============================="

show_help() {
    echo "Usage: $0 <command>"
    echo ""
    echo "Commands:"
    echo "  gtk                 - Build and run the GTK cookbook application"
    echo "  gtk-compile         - Compile the GTK cookbook application (no run)"
    echo "  gtk-test            - Run all tests for cookbook-gtk (unit, integration, UI)"
    echo "  android             - Build, install, run, and stream logs for Pantryman (Android)"
    echo "  check               - Run cargo check on all Rust components"
    echo "  clean               - Clean all build artifacts"
    echo "  test                - Run all tests"
    echo "  engine-test         - Run cookbook-engine tests (fast, no display needed)"
    echo "  gtk-test-headless   - Run GTK tests headlessly (requires xvfb-run)"
    echo "  help                - Show this help message"
    echo ""
}
gtk_compile() {
    echo -e "${CYAN}🔧 Compiling GTK application...${NC}"
    cd "$PROJECT_ROOT"
    RUST_BACKTRACE=1 cargo build -p cookbook-gtk
}

engine_test() {
    echo -e "${CYAN}🧪 Running cookbook-engine tests (fast, no display needed)...${NC}"
    cd "$PROJECT_ROOT"
    cargo test -p cookbook-engine
    echo -e "${GREEN}✅ Engine tests complete${NC}"
}

gtk_test_headless() {
    echo -e "${CYAN}🧪 Running GTK tests headlessly (requires xvfb)...${NC}"
    cd "$PROJECT_ROOT"
    if ! command -v xvfb-run >/dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  xvfb-run not found. Install with: sudo pacman -S xorg-server-xvfb${NC}"
        exit 1
    fi
    xvfb-run -a cargo test -p cookbook-gtk
    echo -e "${GREEN}✅ GTK headless tests complete${NC}"
}

check_device() {
    local device=$($ADB devices | tr -d '\r' | grep -E "device$" | head -1 | cut -f1)
    if [ -z "$device" ]; then
        echo -e "${RED}❌ No Android device connected${NC}"
        exit 1
    fi
    echo -e "${GREEN}📱 Device connected: $device${NC}"
}

run_gtk() {
    # TIP: For verbose logging, run: RUST_LOG=debug ./dev.sh gtk
    echo -e "${CYAN}🔧 Building and running GTK application...${NC}"
    cd "$PROJECT_ROOT"
    RUST_BACKTRACE=1 cargo run -p cookbook-gtk
}


# Combined android workflow: build, install, run, and stream logs
android() {
    clear

    # Set up Android SDK/NDK paths
    local NDK_VERSION
    NDK_VERSION=$(ls "$HOME/Android/Sdk/ndk" 2>/dev/null | tail -1)
    if [ -z "$NDK_VERSION" ]; then
        echo -e "${RED}❌ NDK not found at ~/Android/Sdk/ndk/${NC}"
        echo "Install via SDK Manager: NDK (Side by side)"
        exit 1
    fi
    export ANDROID_NDK_HOME="$HOME/Android/Sdk/ndk/$NDK_VERSION"
    export ANDROID_SDK_ROOT="$HOME/Android/Sdk"
    export PATH="$HOME/Android/Sdk/platform-tools:$PATH"
    echo -e "${CYAN}Using NDK: $ANDROID_NDK_HOME${NC}"

    # Step 1: Build everything
    echo -e "${CYAN}🔧 Building Rust JNI library for Android...${NC}"
    cd "$PANTRYMAN_DIR/rust-bridge"
    if ! command -v cargo-ndk >/dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  cargo-ndk not found, installing...${NC}"
        cargo install cargo-ndk
    fi
    cargo ndk -t arm64-v8a -t x86_64 -o ../app/src/main/jniLibs build --release
    echo -e "${GREEN}✅ Rust JNI library built${NC}"

    echo -e "${CYAN}🔧 Building Pantryman Android app...${NC}"
    cd "$PANTRYMAN_DIR"
    ./gradlew assembleDebug
    echo -e "${GREEN}✅ Build complete${NC}"

    # Step 2: Require a connected hardware device
    local device
    device=$($ADB devices | tr -d '\r' | grep -E "device$" | head -1 | cut -f1)
    if [ -z "$device" ]; then
        echo -e "${RED}❌ No device connected. Plug in your Pixel 7 and try again.${NC}"
        exit 1
    fi
    echo -e "${GREEN}📱 Device: $device${NC}"

    # Step 3: Install via adb directly (more reliable than gradlew installDebug)
    echo -e "${CYAN}📱 Installing to device...${NC}"
    APK_PATH="$PANTRYMAN_DIR/app/build/outputs/apk/debug/app-debug.apk"
    if [ ! -f "$APK_PATH" ]; then
        echo -e "${RED}❌ APK not found at $APK_PATH${NC}"
        exit 1
    fi
    $ADB uninstall $APP_PACKAGE 2>/dev/null || true
    $ADB install "$APK_PATH"
    echo -e "${GREEN}✅ Installation complete${NC}"

    # Run
    echo -e "${CYAN}🚀 Starting app...${NC}"
    $ADB shell am start -n "$APP_PACKAGE/.MainActivity"
    echo -e "${GREEN}✅ App started${NC}"

    # Logs
    echo -e "${CYAN}📊 Streaming logs (Press Ctrl+C to stop)...${NC}"
    $ADB logcat | grep --line-buffered -E "(MainActivity|CookbookEngine)"
}

android_data() {
    check_device
    echo -e "${CYAN}📁 Current Android app data directory:${NC}"
    echo ""
    echo "--- Ingredients ---"
    $ADB shell run-as "$APP_PACKAGE" ls -la files/cookbook_data/ingredients/ 2>/dev/null || echo "No access or directory doesn't exist"
    echo ""
    echo "--- Pantry ---"
    $ADB shell run-as "$APP_PACKAGE" cat files/cookbook_data/pantry.yaml 2>/dev/null || echo "No access or file doesn't exist"
    echo ""
}

run_check() {
    echo -e "${CYAN}🔍 Running cargo check...${NC}"
    cd "$PROJECT_ROOT"
    cargo check --workspace
    echo -e "${GREEN}✅ Check complete${NC}"
}

run_clean() {
    echo -e "${CYAN}🧹 Cleaning build artifacts...${NC}"
    cd "$PROJECT_ROOT"
    cargo clean
    cd "$PANTRYMAN_DIR"
    ./gradlew clean
    echo -e "${GREEN}✅ Clean complete${NC}"
}

run_test() {
    echo -e "${CYAN}🧪 Running tests...${NC}"
    cd "$PROJECT_ROOT"
    cargo test --workspace
    echo -e "${GREEN}✅ Tests complete${NC}"
}

# Main command handling
case "${1:-help}" in
    "gtk")
        run_gtk
        ;;
    "gtk-compile")
        gtk_compile
        ;;
    "android")
        android
        ;;
    "check")
        run_check
        ;;
    "engine-test")
        engine_test
        ;;
    "gtk-test-headless")
        gtk_test_headless
        ;;
    "gtk-test")
        echo -e "${CYAN}🧪 Running GTK tests...${NC}"
        cd "$PROJECT_ROOT"
        cargo test --manifest-path cookbook-gtk/Cargo.toml --all-features
        echo -e "${GREEN}✅ GTK tests complete${NC}"
        ;;
    "clean")
        run_clean
        ;;
    "test")
        run_test
        ;;
    "help"|"--help"|"-h")
        show_help
        ;;
    *)
        echo -e "${RED}❌ Unknown command: $1${NC}"
        echo ""
        show_help
        exit 1
        ;;
esac
