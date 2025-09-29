# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Exek is a fast TUI application launcher written in Rust that uses fuzzy matching and frecency-based sorting to help users quickly launch applications.

## Build and Development Commands

```bash
# Build the project
cargo build              # Debug build
cargo build --release    # Optimized release build

# Run tests
cargo test              # Run all tests
cargo test <test_name>  # Run specific test

# Lint and format
cargo clippy            # Run clippy linter
cargo fmt               # Format code

# Run the application
cargo run               # Debug mode
./target/release/exek   # Run release build
```

## Architecture

The application is organized into five main modules:

- **main.rs**: Entry point, terminal setup/teardown, and event loop handling
- **desktop_entry.rs**: Parses XDG desktop files from standard locations to discover installed applications
- **database.rs**: Manages frecency data persistence (launch counts and timestamps) in JSON format at `~/.config/exek/`
- **search.rs**: Implements fuzzy matching using the `fuzzy-matcher` crate and frecency scoring algorithm
- **ui.rs**: Manages the TUI using `ratatui`, handles keyboard input, and renders the application list

## Key Implementation Details

The application scans these directories for desktop files:
- `/usr/share/applications`
- `/usr/local/share/applications`
- `~/.local/share/applications`
- Flatpak application directories

Terminal applications are detected via the `Terminal=true` field in desktop files and launched with terminal emulator auto-detection.

The frecency algorithm combines frequency (launch count) and recency (time since last launch) to prioritize frequently and recently used applications in search results.

## Dependencies

Key dependencies defined in Cargo.toml:
- `ratatui` for TUI rendering
- `crossterm` for terminal manipulation
- `fuzzy-matcher` for fuzzy string matching
- `tokio` for async runtime
- `serde`/`serde_json` for data persistence
