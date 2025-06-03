# Pantryman Vision & Roadmap

## Core Vision: "The Obsidian Shortcut" for Recipe Management

Pantryman is designed to be the mobile companion to cookbook-gtk, enabling seamless synchronization of pantry and ingredient data without implementing custom sync logic. The key insight is leveraging existing third-party sync solutions (like Syncthing, Dropbox, Google Drive, etc.) to handle the heavy lifting.

## The Target Workflow

1. **Mobile Pantry Management**: User grabs their phone and goes through their physical pantry
2. **Real-time Updates**: Using Pantryman to add/edit/delete ingredients and update quantities
3. **Automatic Sync**: Third-party sync app automatically syncs the data_dir folder between devices
4. **Desktop Recipe Planning**: User sits down at their computer with cookbook-gtk to browse recipes, knowing their pantry data is current and accurate

## Current Status (June 2025)

### ✅ **COMPLETED - Core UI Foundation**
- Fixed button responsiveness issues (action bar overlay conflict resolved)
- Loading text management working properly
- Basic CRUD operations for ingredients and pantry items
- Settings page with data directory selection
- App successfully reads/writes YAML files through cookbook-engine

### ✅ **COMPLETED - Dynamic Data Directory Switching**
**Goal**: Change data_dir without app restart + proper file migration

**Implemented Features**:
- ✅ Live reinitialization of cookbook-engine when data_dir changes (no app restart required)
- ✅ Proper file migration logic when changing data directories
- ✅ Activity result communication between SettingsActivity and MainActivity
- ✅ Automatic UI refresh after data directory change
- ✅ Full migration of ingredients/, recipes/, and pantry.yaml files
- ✅ Support for both empty directories (migration) and existing cookbook directories (switching)

**How it works**:
1. User opens Settings and selects new data directory
2. App detects if directory is empty (migration needed) or contains valid cookbook data (direct switch)
3. Files are copied/migrated using Android DocumentFile API
4. SettingsActivity notifies MainActivity via onActivityResult
5. MainActivity calls reinitializeWithNewDataDirectory() 
6. New CookbookEngine instance created with new data_dir
7. UI refreshes automatically with new data

### 🎯 **NEXT PRIORITY - Real-World Sync Testing**
**The Big Picture**: Enable seamless sync between Pantryman (Android) and cookbook-gtk (desktop) through shared data_dir folder, synced by third-party apps.

**Sync Workflow**:
```
[Pantryman Mobile] 
    ↕ (writes/reads YAML files)
[Shared data_dir folder]
    ↕ (synced by Syncthing/Dropbox/etc.)
[cookbook-gtk Desktop]
```

**Benefits**:
- No custom sync server required
- User controls their data and sync method
- Works offline with eventual consistency
- Leverages battle-tested sync solutions

## Technical Architecture

### Data Flow
- **Pantryman**: Android app ↔ cookbook-engine (Rust) ↔ YAML files in data_dir
- **cookbook-gtk**: Desktop app ↔ cookbook-engine (Rust) ↔ YAML files in data_dir
- **Sync Layer**: Third-party app syncs data_dir folder between devices

### File Structure
```
data_dir/
├── pantry.yaml           # Current pantry state
├── ingredients/          # Available ingredients database
│   ├── potato.yaml
│   ├── tomato.yaml
│   └── ...
└── recipes/             # Recipe collection
    ├── lasagna.md
    └── ...
```

## Future Enhancements (Lower Priority)

### UI Improvements
- Category dropdown with existing categories + "new" option
- Better visual design and layout
- Improved form validation and error handling

### Advanced Features
- Recipe suggestions based on current pantry
- Shopping list generation
- Expiration date tracking
- Barcode scanning for ingredient addition

## Success Metrics

The vision is achieved when:
1. ✅ User can manage pantry on mobile (Pantryman)
2. ✅ User can plan recipes on desktop (cookbook-gtk)
3. 🎯 Data syncs seamlessly between devices via third-party sync
4. 🎯 No app restarts required for data directory changes
5. 🎯 Files properly migrate when changing data directories

## Key Insight: "The Obsidian Shortcut"

Just like Obsidian doesn't implement its own sync but relies on users to sync their vault folder via their preferred method, Pantryman + cookbook-gtk can achieve robust synchronization by:
- Using a standardized file format (YAML/Markdown)
- Reading/writing to a shared folder structure
- Letting users choose their preferred sync solution
- Focusing on the core functionality rather than sync infrastructure

This approach is simpler, more reliable, and gives users full control over their data.
