# droponoff

A **scriptable, reversible kill switch for Dropbox** on macOS.

---

## ⚠️ WARNING ⚠️

**DO NOT USE THIS TOOL WITHOUT FIRST READING THROUGH THE ENTIRE CODEBASE.**

- Make no assumptions about its safety or correctness.
- You must understand exactly what it does before using it and
  make your own decision on whether to risk it.
- The processes implemented by this tool is in no way supported by
  Dropbox nor the author of this tool.

---

## Purpose

`droponoff` serves two purposes:

1. **Reversible Dropbox kill switch**: Completely disable Dropbox (processes, LaunchAgents, File Provider extensions) and later restore it to normal operation. Useful for any scenario where you need Dropbox fully stopped.

2. **Recover disk space from leaked scratch files**: Dropbox can accumulate unbounded temporary files (`scratch_files`) under its File Provider group container that are never cleaned up. The `nuke-scratch` command deletes these files after verifying Dropbox is fully stopped.

## Installation

```bash
cargo install --path .
```

## Usage

```bash
# Check current Dropbox state
droponoff status

# Completely disable Dropbox. If you are intending using nuke-scratch,
# you should wait for Dropbox to have finished any synchronization
# activity (especially any uploads).
droponoff off

# Restore Dropbox to normal operation
droponoff on

# DANGEROUS:
#
# With Dropbox OFF and no pending file synchronization in flight prior
# to being turned off, delete scratch files. Will refuse
# to do anything if it detects Dropbox is still running.
droponoff nuke-scratch
```

## Requirements

- macOS only. At the time of this writing, tested on Tahoe.

## Warning

This tool directly manipulates Dropbox's runtime state. While designed to be as safe and reasonable given that it is doing inherently unsafe things, use with caution and ensure you understand what it does before running it.
