# Overview

This project is a Rust-based client for parsing and processing Nix build logs using the `nix build --log-format internal-json` protocol. The primary goal is to make Nix logging more human-readable by:

- Accumulating individual logs when building multiple derivations in parallel and only displaying them on errors
- Preventing duplicate error messages and removing verbose indentation from Nix's error summaries
- Providing both normal and verbose monitoring modes for different debugging needs
- **Real-time status display with target tracking and intelligent redraw optimization to prevent console flickering**

The project serves as both a standalone binary and demonstrates how to implement the Nix JSON protocol, though it's currently in prototype stage with plans to become a library crate.

## Recent Changes (September 2025)
- **Implemented complete target tracking system** with FIFO ordering for build targets in status display
- **Added anti-flickering optimization** using state change detection to prevent unnecessary status line redraws  
- **Enhanced DisplayManager** with terminal width-aware status formatting showing `[ stats ] target1, target2, target3`

# User Preferences

Preferred communication style: Simple, everyday language.

# System Architecture

## Core Architecture
The project follows a typical Rust CLI application structure with the main binary in `src/bin/` and library code in `src/lib.rs`. The architecture is designed around:

**Stream Processing Pattern**: The application reads JSON-formatted log messages from stdin (typically piped from `nix build --log-format internal-json`) and processes them in real-time.

**Event-Driven Design**: Uses an event-based approach to handle different types of Nix log messages, allowing for flexible processing and formatting of build events.

**Error Aggregation**: Implements a buffering mechanism to collect logs from parallel builds and only display them when errors occur, reducing noise during successful builds.

## Key Components

**CLI Interface**: Built using the `clap` crate with derive macros for argument parsing, supporting JSON output mode and verbose flags.

**JSON Processing**: Utilizes `serde` and `serde_json` for deserializing Nix's internal JSON log format into structured Rust types.

**Terminal Output**: Integrates `console` and `crossterm` crates for enhanced terminal output with color support and cross-platform compatibility.

**Async Runtime**: Uses `tokio` for asynchronous I/O operations, enabling efficient handling of streaming log data.

## Error Handling
Implements comprehensive error handling using the `thiserror` crate for custom error types and `anyhow` for error context propagation throughout the application.

# External Dependencies

## Core Rust Ecosystem
- **tokio**: Async runtime for handling streaming I/O operations
- **serde/serde_json**: JSON serialization and deserialization for processing Nix's log format
- **clap**: Command-line argument parsing with derive support
- **anyhow/thiserror**: Error handling and custom error type definitions

## Terminal and UI
- **console**: Cross-platform terminal manipulation and styling
- **crossterm**: Low-level terminal control for enhanced output formatting
- **regex**: Pattern matching for log parsing and filtering

## Utility Libraries
- **chrono**: Date and time handling for log timestamps
- **bytes**: Efficient byte buffer management for stream processing
- **once_cell**: Thread-safe lazy static initialization

The project specifically targets integration with Nix's build system and processes the internal JSON protocol that Nix outputs when using the `--log-format internal-json` flag. Test data is provided in the `tests/` directory for development and validation purposes.