# DBC Viewer

A quick and dirty Tauri app for viewing and searching CAN DBC files. Yeah, the code is pretty ugly, but it works! 🚗

## What does it do?

This app lets you:
- Upload and parse DBC (CAN database) files
- Search for signals and messages by name
- Browse through all messages in your DBC file
- Explore individual signals and their properties
- Navigate through search results with a history feature

## Tech Stack

- **Frontend**: Vanilla HTML/CSS/JavaScript with Bootstrap 5
- **Backend**: Rust with Tauri 2.5
- **Parser**: Custom DBC file parser (in Rust)

## Running the thing

Install dependencies and run:
```bash
bun install 
bun run tauri dev
```

If you want the exe
```bash
bun run tauri build
```

## Note

This is a work in progress. The code could definitely use some refactoring, but hey, it parses DBC files and displays them nicely! 🤷‍♂️
