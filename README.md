# DBC Viewer

A Tauri app for viewing and searching CAN DBC files.

## What It Does

This app lets you:
- Upload and parse DBC (CAN database) files
- Search for signals and messages by name or ID
- Browse through all messages in a DBC file
- Explore individual signals and their properties
- Navigate through search result details with a history feature

## Tech Stack

- **Frontend**: Vanilla HTML/CSS/JavaScript with Bootstrap 5
- **Backend**: Rust with Tauri 2
- **Parser**: Custom DBC parser in Rust

## Running

Install dependencies and run:

```bash
bun install
bun run tauri dev
```

Build the desktop app:

```bash
bun run tauri build
```

## Note

This is a work in progress. The current focus is keeping the parser reliable and sharing the same Tauri entry point across desktop and mobile builds.
