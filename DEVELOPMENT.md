# Cookbook Project Development Documentation

**Last Updated**: June 7, 2025

## Overview

Cookbook is a cross-platform recipe and pantry management application. This document consolidates all the development work and decisions made on the Pantryman Android component.

## Project Structure

- `cookbook-engine/` - Core Rust library with business logic
- `cookbook-gtk/` - GTK desktop frontend
- `pantryman/` - Android mobile app for pantry management
- `example/data/` - Sample data for testing and initial setup

## Pantryman Android App - Development Summary

### Vision: "The Obsidian Shortcut" for Recipe Management

Pantryman serves as the mobile companion to cookbook-gtk, enabling seamless synchronization of pantry and ingredient data using existing third-party sync solutions (Syncthing, Dropbox, Google Drive, etc.).

**Target Workflow:**
1. **Mobile Pantry Management**: User uses phone to update pantry while shopping/organizing
2. **Real-time Updates**: Pantryman updates ingredient quantities and availability
3. **Automatic Sync**: Third-party sync keeps data_dir synchronized across devices
4. **Desktop Recipe Planning**: User plans meals on desktop with current pantry data

### Major Issues Resolved

#### 1. Data Loss Bug (CRITICAL)
**Problem**: App was deleting entire data directory on every startup, losing all user changes.

**Root Cause**: `setupInitialData()` in MainActivity.kt always deleted existing data:
```kotlin
// BROKEN CODE
if (dataDir.exists()) { 
    dataDir.deleteRecursively() 
}
```

**Solution**: Only setup initial data if directory is empty:
```kotlin
// FIXED CODE
if (!dataDir.exists() || dataDir.listFiles()?.isEmpty() == true) {
    // copy initial data
} else {
    Log.d("MainActivity", "Data directory already exists with files, skipping initial setup")
}
```

#### 2. Incomplete Pantry Updates
**Problem**: UI updates weren't fully syncing to YAML files (missing quantity, quantityType, lastUpdated fields).

**Solution**: Enhanced `updatePantryStatus()` to handle all fields properly and ensure immediate YAML persistence.

#### 3. Dynamic Data Directory Switching
**Implementation**: Successfully added ability to change data directories without app restart:
- Live reinitialization of cookbook-engine
- Proper file migration logic
- Activity result communication between activities
- Automatic UI refresh after directory change

### Key Features Implemented

✅ **Core CRUD Operations**: Add/edit/delete ingredients and pantry items  
✅ **YAML Persistence**: All changes immediately written to YAML files  
✅ **Dynamic Data Directory**: Change data folder without restart + migration  
✅ **Error Handling**: Proper feedback on operation success/failure  
✅ **Modern UI**: Responsive design with proper loading states  

### Technical Architecture

- **Frontend**: Kotlin Android app with modern Material Design
- **Backend**: Rust cookbook-engine via JNI bridge
- **Data Storage**: YAML files (ingredients/, pantry.yaml, recipes/)
- **Sync Strategy**: File-based sync using third-party tools

### Development Tools

Use the `./dev.sh` script for common development tasks:

```bash
./dev.sh gtk                 # Run GTK application
./dev.sh android-build       # Build Android app
./dev.sh android-install     # Build and install to device
./dev.sh android-run         # Build, install, and run
./dev.sh android-logs        # Monitor app logs
./dev.sh android-data        # Check current data directory
./dev.sh check               # Run cargo check
./dev.sh clean               # Clean build artifacts
./dev.sh test                # Run all tests
```

### Testing Strategy

The engine has been thoroughly tested with comprehensive test suites:
- `test_yaml_writing.rs` - Basic YAML writing functionality
- `test_pantryman_yaml_writing.rs` - Full workflow simulation
- `test_ingredient_update.rs` - Ingredient-specific updates
- `test_pantryman_ingredient_real.rs` - Real data directory testing

All tests confirm the engine works correctly. Issues were in the Android app layer, not the core engine.

### Current Status (June 2025)

✅ **STABLE**: Core functionality working reliably  
✅ **TESTED**: Comprehensive test coverage proving engine reliability  
✅ **DOCUMENTED**: All major issues identified and resolved  
✅ **READY**: App ready for regular use with third-party sync solutions  

The Pantryman app now provides a solid foundation for mobile pantry management with reliable data persistence and sync capabilities.
