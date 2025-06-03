#!/bin/bash

# Test script for Pantryman data directory synchronization
# This script demonstrates how to test the data directory changing feature

echo "=== Pantryman Data Directory Sync Test ==="

# Set variables
APK_PATH="/home/mpr/code/cookbook/pantryman/app/build/outputs/apk/debug/app-debug.apk"
PACKAGE_NAME="com.example.pantryman"
TEST_DIR="/sdcard/Documents/cookbook_test"

echo "1. Installing latest APK..."
cd /home/mpr/code/cookbook/pantryman
./gradlew installDebug

echo "2. Starting app to initialize default data..."
adb shell am start -n $PACKAGE_NAME/.MainActivity
sleep 3

echo "3. Stopping app..."
adb shell am force-stop $PACKAGE_NAME

echo "4. Creating test directory structure on device..."
adb shell "mkdir -p $TEST_DIR/ingredients"
adb shell "mkdir -p $TEST_DIR/recipes"

echo "5. Creating a test ingredient file..."
adb shell "echo 'name: test_ingredient
category: test
kb: 
tags: [test, sync]' > $TEST_DIR/ingredients/test_ingredient.yaml"

echo "6. Creating a test pantry file..."
adb shell "echo 'version: 1
items:
  - ingredient: test_ingredient
    quantity: 5
    quantity_type: kg
    last_updated: 2025-06-03' > $TEST_DIR/pantry.yaml"

echo "7. Test data structure created at: $TEST_DIR"
echo "   - ingredients/test_ingredient.yaml"
echo "   - pantry.yaml"

echo ""
echo "=== Manual Test Instructions ==="
echo "1. Open the Pantryman app"
echo "2. Click the Settings button (⚙)"
echo "3. Click 'Choose Data Directory'"
echo "4. Navigate to Documents/cookbook_test"
echo "5. Select that directory"
echo "6. Choose 'Yes' to switch to the new data"
echo "7. Restart the app"
echo "8. Verify that you see 'test_ingredient' in the list"
echo ""
echo "To monitor logs during the test:"
echo "adb logcat -s MainActivity SettingsActivity CookbookEngine RustBridge -v time"

echo ""
echo "Test setup complete!"
