# Pantryman

Pantryman is a simple Android app for managing your pantry ingredients. This is a minimal starter project that displays "Hello world!" on launch.

## Building

You can build this project using Gradle:

```
gradle assembleDebug
```

Or open the project in Android Studio for full IDE support.

## Running

To run on your Pixel 7 or any Android device:
1. Enable Developer Mode and USB debugging on your device.
2. Connect your device via USB.
3. Run:
   ```
   gradle installDebug
   ```
4. The app should appear as "Pantryman" and display "Hello world!" on launch.

Quick check: quick_check.sh

# Get fresh view anytime
./monitor_data_dir.sh fresh

# Just check onion files
./monitor_data_dir.sh check

# Monitor changes in real-time
./monitor_data_dir.sh monitor