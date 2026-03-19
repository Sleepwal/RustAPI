# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust binary project using the 2024 edition. It is a minimal project created with `cargo new`.

## Common Commands

```bash
# Build the project
cargo build

# Build for release
cargo build --release

# Run the project
cargo run

# Run tests
cargo test

# Run a specific test
cargo test <test_name>

# Format code
cargo fmt

# Lint with Clippy
cargo clippy

# Check for errors without building
cargo check
```

## Project Structure

- `Cargo.toml` - Project configuration and dependencies
- `src/main.rs` - Binary entry point
- `src/` - Source code directory
