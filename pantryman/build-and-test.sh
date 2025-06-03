#!/bin/bash

# Pantryman Android App - Build and Test Script

set -e

# Set up Android NDK environment
export ANDROID_NDK_HOME="/opt/android-ndk-r26d"

echo "🔧 Building Rust bridge library for Android..."
cd rust-bridge

# Build for x86_64 emulator (most common for development)
echo "📱 Building for x86_64 Android emulator..."
cargo ndk -t x86_64 build --release

# Also build for ARM64 (real devices)
echo "📱 Building for ARM64 Android devices..."
cargo ndk -t arm64-v8a build --release

# Copy the libraries to the correct Android jniLibs directory
echo "📋 Copying native libraries to Android project..."
mkdir -p ../app/src/main/jniLibs/x86_64
mkdir -p ../app/src/main/jniLibs/arm64-v8a

cp target/x86_64-linux-android/release/libpantryman_bridge.so ../app/src/main/jniLibs/x86_64/
cp target/aarch64-linux-android/release/libpantryman_bridge.so ../app/src/main/jniLibs/arm64-v8a/

cd ..

echo "📱 Building Android APK..."
./gradlew assembleDebug

echo "✅ Build completed successfully!"
echo "📦 APK location: app/build/outputs/apk/debug/app-debug.apk"

# Check for connected devices
echo ""
echo "📱 Checking for connected Android devices..."
adb devices

if adb devices | grep -q "device$"; then
    echo ""
    read -p "📲 Install APK on connected device? (y/n): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "📲 Installing APK..."
        adb install -r app/build/outputs/apk/debug/app-debug.apk
        echo "✅ APK installed successfully!"
        echo "🚀 You can now launch 'Pantryman' from your device"
    fi
else
    echo "ℹ️  No Android devices connected. To test:"
    echo "   1. Connect an Android device with USB debugging enabled, or"
    echo "   2. Start an Android emulator"
    echo "   3. Run: adb install app/build/outputs/apk/debug/app-debug.apk"
fi

echo ""
echo "📝 Development notes:"
echo "   • The app uses internal storage for data: /data/data/com.example.pantryman/files/cookbook_data"
echo "   • To test with example data, copy files from example/data/ to the app's data directory"
echo "   • Check logs with: adb logcat -s MainActivity,CookbookEngine"
