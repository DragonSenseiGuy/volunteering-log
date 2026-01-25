# Volunteering Log - Agent Guidelines

## Project Overview

Cross-platform desktop and mobile app for logging volunteer hours. Built with Tauri 2.x + Svelte + TypeScript.

## Tech Stack

- **Frontend**: Svelte 5, TypeScript, SvelteKit
- **Backend**: Tauri 2.x (Rust)
- **Storage**: Local JSON file (no database)
- **Platforms**: macOS, Windows, Linux, iOS

## Commands

```bash
# Install dependencies
npm install

# Development (desktop)
npm run tauri dev

# Development (iOS - requires physical device or paid Apple account for simulator)
npm run tauri ios dev

# Build for production
npm run tauri build

# Type checking
npm run check

# Rust check
cd src-tauri && cargo check
```

## Project Structure

```
├── src/                    # Svelte frontend
│   └── routes/
│       └── +page.svelte    # Main app component
├── src-tauri/              # Rust backend
│   ├── src/
│   │   └── lib.rs          # Tauri commands (CRUD operations)
│   ├── tauri.conf.json     # Tauri configuration
│   └── gen/                # Generated mobile projects
│       ├── apple/          # iOS Xcode project
│       └── android/        # Android Studio project
└── static/                 # Static assets
```

## Key Files

- `src/routes/+page.svelte` - All UI logic and styling
- `src-tauri/src/lib.rs` - Rust backend with `get_entries`, `add_entry`, `update_entry`, `delete_entry` commands
- `src-tauri/tauri.conf.json` - App config, window settings, iOS team ID

## Code Conventions

- Use TypeScript for frontend code
- All form fields stack vertically on mobile
- Use card-based layout instead of tables for mobile compatibility
- Font size 16px on inputs to prevent iOS zoom
- Include iOS safe area padding

## iOS Development Notes

- Requires Apple Developer account (free works with physical device)
- Team ID configured in `tauri.conf.json` under `bundle.iOS.developmentTeam`
- Must enable Developer Mode on iPhone (Settings → Privacy & Security)
- Must trust developer certificate (Settings → General → VPN & Device Management)

## Data Storage

Data stored as JSON at the app's data directory:
- macOS: `~/Library/Application Support/com.ajayn.volunteerlog/volunteer_log.json`
- Windows: `%APPDATA%/com.ajayn.volunteerlog/volunteer_log.json`
- Linux: `~/.local/share/com.ajayn.volunteerlog/volunteer_log.json`
