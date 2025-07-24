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
PROJECT_ROOT="/home/mpr/code/cookbook"
PANTRYMAN_DIR="$PROJECT_ROOT/pantryman"
APP_PACKAGE="com.example.pantryman"
# Use Windows adb.exe if available (for WSL/Arch)
ADB="/mnt/c/Android/platform-tools/adb.exe"

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
    echo "  help                - Show this help message"
    echo ""
}
gtk_compile() {
    echo -e "${CYAN}🔧 Compiling GTK application...${NC}"
    cd "$PROJECT_ROOT"
    RUST_BACKTRACE=1 cargo build -p cookbook-gtk
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
    echo -e "${CYAN}🔧 Building and running GTK application...${NC}"
    cd "$PROJECT_ROOT"
    RUST_BACKTRACE=1 cargo run -p cookbook-gtk
}


# Combined android workflow: build, install, run, and stream logs
android() {
    clear
    
    export PATH="/mnt/c/Android/platform-tools:$PATH"
    export ANDROID_NDK_HOME="/opt/android-ndk" # Adjust as needed½
    export ANDROID_SDK_ROOT="/mnt/c/Users/madsp/AppData/Local/Android/Sdk" # Adjust as needed

    echo -e "${CYAN}🔧 Building Rust JNI library for Android...${NC}"
    cd "$PANTRYMAN_DIR/rust-bridge"
    if ! command -v cargo-ndk >/dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  cargo-ndk not found, installing...${NC}"
        cargo install cargo-ndk
    fi
    cargo ndk -t arm64-v8a -o ../app/src/main/jniLibs build --release
    echo -e "${GREEN}✅ Rust JNI library built and copied${NC}"

    echo -e "${CYAN}🔧 Building Pantryman Android app...${NC}"
    cd "$PANTRYMAN_DIR"
    gradle assembleDebug
    echo -e "${GREEN}✅ Build complete${NC}"

    # Install
    check_device
    echo -e "${CYAN}📱 Installing to device...${NC}"
    cd "$PANTRYMAN_DIR"
    
    if ! ADB=$ADB gradle installDebug; then
        echo -e "${YELLOW}⚠️  Gradle installDebug failed, trying manual APK install...${NC}"
        APK_PATH="app/build/outputs/apk/debug/app-debug.apk"
        if [ -f "$APK_PATH" ]; then
            echo -e "${YELLOW}⚠️  Uninstalling existing app to avoid signature conflict...${NC}"
            $ADB uninstall $APP_PACKAGE || true
            $ADB install -r "$APK_PATH"
            echo -e "${GREEN}✅ Manual APK install complete${NC}"
        else
            echo -e "${RED}❌ APK not found at $APK_PATH${NC}"
            exit 1
        fi
    else
        echo -e "${GREEN}✅ Installation complete${NC}"
    fi

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
    gradle clean
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
       run_test
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
