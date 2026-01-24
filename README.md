# Volunteering Log

A simple cross-platform desktop app to log your volunteering hours.

## Features

- Log volunteering entries with place, date, hours, and notes
- Date auto-fills to today (editable)
- View all entries in a table
- Edit and delete entries
- Total hours tracked automatically
- Data stored locally (no sign-up required)

## Development

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

## Building

```bash
# Build for your current platform
npm run tauri build
```

Binaries will be in `src-tauri/target/release/bundle/`.

## Tech Stack

- **Frontend**: Svelte + TypeScript
- **Backend**: Tauri (Rust)
- **Storage**: Local JSON file
