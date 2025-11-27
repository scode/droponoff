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

`droponoff` exists to **make it safe to clean up Dropbox's leaked scratch files** that eat unbounded amount of disk space. Dropbox sometimes leaves behind `scratch_files` under its File Provider group container, and you shouldn't touch them while Dropbox is running. This tool provides a reversible way to stop Dropbox completely, verify it's off, and then remove those leftovers. (But again, this is entirely unsupported and risky.)

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
