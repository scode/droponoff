# droponoff

A **scriptable, reversible kill switch for Dropbox** on macOS.

---

## ⚠️ WARNING ⚠️

**DO NOT USE THIS TOOL WITHOUT FIRST READING THROUGH THE ENTIRE CODEBASE.**

This is early-stage software. Make no assumptions about its safety or correctness. You must understand exactly what it does before running any commands. Improper use could affect your Dropbox installation and data synchronization.

---

## Purpose

`droponoff` provides complete control over Dropbox's operation on macOS, allowing you to safely manipulate Dropbox's internal state files:

- **OFF mode:** Dropbox becomes completely inert—no UI, no background processes, no auto-relaunch, no File Provider behavior. Internal Dropbox files can be safely manipulated without interference.
- **ON mode:** Restore Dropbox to normal operation so it can reindex and rebuild whatever changed.

## Key Features

- **Reversible:** All changes can be undone. Configuration files are renamed (not deleted) and can be fully restored.
- **Safe:** Only touches Dropbox processes, LaunchAgents, and extensions. Your Dropbox files remain untouched.
- **Idempotent:** Running `off` or `on` multiple times is safe and won't cause instability.
- **User-level:** Operates within your user account without requiring system-wide changes.

## Installation

```bash
cargo build --release
```

## Usage

```bash
# Completely disable Dropbox
droponoff off

# Check current Dropbox state
droponoff status

# Restore Dropbox to normal operation
droponoff on
```

## Requirements

- macOS only
- Rust toolchain for building

## Warning

This tool directly manipulates Dropbox's runtime state. While designed to be safe and reversible, use with caution and ensure you understand what it does before running it.
