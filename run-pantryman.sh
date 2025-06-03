#!/bin/bash

# Pantryman Development Script
# This script provides an easy way to build and run the pantryman Android app

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔧 Pantryman Development Script${NC}"
echo "================================"

# Configuration
APP_PACKAGE="com.example.pantryman"
PROJECT_DIR="/home/mpr/code/cookbook/pantryman"

# Set Java environment for Android SDK
export JAVA_HOME=/usr/lib/jvm/java-8-openjdk

# Function to check if a real device is connected
check_device() {
    local device_count=$(adb devices | grep -v "List of devices" | grep -c "device$" || true)
    if [ "$device_count" -gt 0 ]; then
        echo -e "${GREEN}✅ Found $device_count connected device(s)${NC}"
        adb devices | grep "device$" | while read line; do
            echo -e "${GREEN}   - $line${NC}"
        done
        return 0
    else
        echo -e "${RED}❌ No devices connected${NC}"
        echo -e "${YELLOW}💡 Please connect your Android device via USB and enable USB debugging${NC}"
        return 1
    fi
}

# Function to build the app
build_app() {
    echo -e "${YELLOW}🔨 Building pantryman app...${NC}"
    cd "$PROJECT_DIR"
    ./gradlew assembleDebug
    echo -e "${GREEN}✅ Build successful${NC}"
}

# Function to install the app
install_app() {
    echo -e "${YELLOW}📱 Installing app on device...${NC}"
    cd "$PROJECT_DIR"
    ./gradlew installDebug
    echo -e "${GREEN}✅ App installed${NC}"
}

# Function to launch the app
launch_app() {
    echo -e "${YELLOW}🚀 Launching pantryman app...${NC}"
    adb shell am start -n "$APP_PACKAGE/.MainActivity"
    echo -e "${GREEN}✅ App launched${NC}"
}

# Function to show logs
show_logs() {
    echo -e "${YELLOW}📋 Showing app logs (Ctrl+C to stop)...${NC}"
    adb logcat | grep -i pantryman
}

# Function to uninstall the app
uninstall_app() {
    echo -e "${YELLOW}🗑️  Uninstalling pantryman app...${NC}"
    adb uninstall "$APP_PACKAGE" 2>/dev/null || echo -e "${YELLOW}   App not installed${NC}"
    echo -e "${GREEN}✅ App uninstalled${NC}"
}

# Main execution
main() {
    # Parse command line arguments
    case "${1:-}" in
        "device")
            check_device
            ;;
        "build")
            build_app
            ;;
        "install")
            if ! check_device; then
                exit 1
            fi
            install_app
            ;;
        "launch")
            if ! check_device; then
                exit 1
            fi
            launch_app
            ;;
        "logs")
            if ! check_device; then
                exit 1
            fi
            show_logs
            ;;
        "uninstall")
            if ! check_device; then
                exit 1
            fi
            uninstall_app
            ;;
        "full" | "")
            # Full workflow
            if ! check_device; then
                exit 1
            fi
            build_app
            install_app
            launch_app
            echo ""
            echo -e "${GREEN}🎉 Pantryman is ready! Check your device.${NC}"
            echo -e "${BLUE}💡 Tip: Run '$0 logs' to see app logs${NC}"
            ;;
        "help" | "-h" | "--help")
            echo "Usage: $0 [command]"
            echo ""
            echo "Commands:"
            echo "  full        (default) Complete workflow: build, install, launch on connected device"
            echo "  device      Check connected devices"
            echo "  build       Build the app only"
            echo "  install     Install the app on connected device"
            echo "  launch      Launch the app on connected device"
            echo "  logs        Show app logs from connected device"
            echo "  uninstall   Remove the app from connected device"
            echo "  help        Show this help"
            echo ""
            echo "Examples:"
            echo "  $0              # Run full workflow"
            echo "  $0 build        # Just build the app"
            echo "  $0 device       # Check what devices are connected"
            echo "  $0 logs         # Monitor app logs"
            echo ""
            echo "Note: All commands except 'build' require a connected Android device with USB debugging enabled."
            ;;
        *)
            echo -e "${RED}❌ Unknown command: $1${NC}"
            echo "Run '$0 help' for usage information"
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"
