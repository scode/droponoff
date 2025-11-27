# Dropbox Kill Switch Plan (macOS)

## Intent

Provide a **scriptable, reversible master switch for Dropbox** on macOS:

- **OFF mode:** Dropbox is completely inert (no UI, no background processes, no auto-relaunch, no File Provider behavior), so internal Dropbox files can be safely manipulated.
- **ON mode:** Restore Dropbox to normal operation so it can reindex/rebuild whatever changed.

The steps below are for a **person or AI agent** implementing this as a tool. They describe what must happen, not how to write the code.

---

## Global Requirements

1. **Scope**
   - Operate at **user level** (current user account) unless explicitly stated otherwise.

2. **Idempotence**
   - Re-running OFF or ON should not cause additional damage or instability.

3. **Reversibility**
   - Do not delete config or system files.
   - Prefer **rename/move** to a backup name (e.g. `.disabled`, `.bak`) so they can be restored.

4. **Data Safety**
   - Do **not** delete or modify user content in the Dropbox folder unless explicitly instructed.
   - Only assume permission to touch:
     - Caches
     - Databases
     - Support / internal state files

---

## Phase 0 – Discovery & Preconditions

**Goal:** Understand the environment and locate Dropbox-related components.

1. **Verify macOS and identify user context**
   - Confirm the OS is macOS.
   - Determine the current user's home directory.

2. **Locate Dropbox application bundle**
   - Check typical locations:
     - `/Applications/Dropbox.app`
     - `~/Applications/Dropbox.app`
   - Record whether Dropbox appears to be installed.

3. **Enumerate Dropbox extensions via PlugInKit**
   - Query PlugInKit for installed extensions.
   - Identify presence of the following bundle IDs (record each as found/not found):
     - `com.getdropbox.dropbox.fileprovider`
     - `com.getdropbox.dropbox.TransferExtension`
     - `com.getdropbox.dropbox.garcon`

4. **Locate Dropbox LaunchAgent(s)**
   - Check for user LaunchAgents in:
     - `~/Library/LaunchAgents/`
   - Specifically note (if present):
     - `com.dropbox.DropboxMacUpdate.agent.plist`
   - Record the full path(s) for later use.

5. **Check for running Dropbox processes**
   - Enumerate running processes.
   - Identify any whose executable name, bundle name, or command line clearly indicate Dropbox:
     - Main Dropbox client
     - Dropbox helpers
     - Dropbox updater (e.g. `DropboxMacUpdate`)
   - Record the list (for reporting and verification).

---

## Phase 1 – Turn Dropbox OFF (Hard Disable)

**Goal:** Ensure Dropbox cannot run or interact with the filesystem until re-enabled.

### Step 1 – Stop All Dropbox Processes

1. **Request clean app shutdown**
   - If Dropbox is running as a GUI application:
     - Send a polite "quit" request to `Dropbox.app`.

2. **Force termination of remaining Dropbox processes**
   - Enumerate processes again.
   - Terminate any process related to Dropbox, including:
     - Main Dropbox client
     - Any `Dropbox` or `DropboxMacUpdate` helpers
   - Repeat until no Dropbox-related processes remain for the current user.

---

### Step 2 – Disable Auto-Relaunch (LaunchAgent)

**Goal:** Prevent Dropbox from restarting automatically.

1. **Unload Dropbox LaunchAgent for current user**
   - If `~/Library/LaunchAgents/com.dropbox.DropboxMacUpdate.agent.plist` exists:
     - Use the platform mechanism (`launchctl` or equivalent) to unload this LaunchAgent for the current user session.

2. **Persistently disable LaunchAgent (reversible)**
   - If the LaunchAgent plist exists:
     - Rename it to a disabled name (e.g. `com.dropbox.DropboxMacUpdate.agent.plist.disabled`)
       - or move it to a clearly marked “disabled” location within the same directory.
   - Record that this file was modified so it can be restored during the ON phase.

---

### Step 3 – Disable Dropbox Extensions (File Provider / Finder)

**Goal:** Ensure macOS no longer loads Dropbox as a file system provider or Finder integration.

1. **For each discovered Dropbox extension bundle ID** (from Phase 0):
   - `com.getdropbox.dropbox.fileprovider`
   - `com.getdropbox.dropbox.TransferExtension`
   - `com.getdropbox.dropbox.garcon`

2. **Mark each as “ignored / disabled” via PlugInKit**
   - For every bundle ID found:
     - Set its state so it will **not** be loaded for this user (equivalent to the System Settings toggle being off).

3. **Refresh Finder**
   - Request Finder to restart.
   - This ensures:
     - Existing extension hooks are dropped.
     - Dropbox context menus, badges, and other Finder UI elements are removed.

---

### Step 4 – OFF State Verification

**Goal:** Confirm Dropbox is fully inert.

1. **Process verification**
   - Enumerate processes.
   - Confirm:
     - No Dropbox main client process is running.
     - No known Dropbox helper or updater processes are running.

2. **Extension state verification**
   - Query PlugInKit for each Dropbox bundle ID:
     - Confirm that each is in a disabled/ignored state for the current user.

3. **Optional File Provider verification**
   - Optionally, query File Provider domains.
   - Confirm that Dropbox’s File Provider domain is not actively being serviced (or is in the expected inactive state).

If all checks pass, the system is considered in **“Dropbox OFF”** mode and it is safe to manipulate internal Dropbox state.

---

## Phase 2 – Internal File Manipulation (User/Operator Phase)

**Goal:** Allow the user or higher-level logic to modify Dropbox’s internal files while Dropbox is off.

The agent/tool should:

1. **Avoid user content modifications by default**
   - Do not alter files within the user’s Dropbox synced folder (e.g. `~/Dropbox` or `~/Library/CloudStorage/Dropbox`) unless explicitly instructed.

2. **Prefer rename/move over delete**
   - When requested to change internal state files (e.g., caches, databases, indexes):
     - Move or rename these to backup names.
     - Example: rename `state.db` → `state.db.bak`.

3. **Maintain OFF state during operations**
   - Ensure no new Dropbox processes are launched while internal file operations are ongoing.
   - Do not reload LaunchAgents or re-enable extensions until Phase 3.

The specifics of which internal files to manipulate are outside this generic plan and should be treated as explicit user instructions.

---

## Phase 3 – Turn Dropbox ON (Restore Operation)

**Goal:** Return Dropbox to normal operation, allowing it to rebuild or resync as needed.

### Step 1 – Restore and Reload LaunchAgent

1. **Restore LaunchAgent**
   - If the tool previously renamed or moved:
     - `~/Library/LaunchAgents/com.dropbox.DropboxMacUpdate.agent.plist`
   - Move it back to its original filename and location.

2. **Reload LaunchAgent for the current user**
   - Use the platform mechanism (`launchctl` or equivalent) to load the restored LaunchAgent.
   - This re-enables Dropbox’s background update behavior as originally configured.

---

### Step 2 – Re-enable Dropbox Extensions

1. **For each known Dropbox extension bundle ID**:
   - `com.getdropbox.dropbox.fileprovider`
   - `com.getdropbox.dropbox.TransferExtension`
   - `com.getdropbox.dropbox.garcon` (if present)

2. **Set extensions to active via PlugInKit**
   - Change the state from “ignored/disabled” back to “enabled/in use” for each bundle ID that exists.

3. **Refresh Finder**
   - Request Finder to restart again so it:
     - Re-attaches Dropbox Finder integrations.
     - Restores context menus, badges, and any other relevant UI.

---

### Step 3 – Relaunch Dropbox Client

1. **Start Dropbox application**
   - Launch the Dropbox app for the current user.

2. **Allow initialization time**
   - Wait for Dropbox’s main process to appear.
   - Allow time for initial startup, reindexing detection, and syncing logic to begin.

---

### Step 4 – ON State Verification

**Goal:** Make sure Dropbox is operational and integrated again.

1. **Process verification**
   - Confirm that the Dropbox main client process is running.
   - Confirm that expected helper/updater processes exist (if applicable).

2. **Extension/Provider verification**
   - Query PlugInKit for each Dropbox extension bundle ID.
   - Confirm they are in an enabled/active state.

3. **Functional smoke test (optional)**
   - Check that the Dropbox folder (e.g. under `~/Library/CloudStorage/Dropbox` or `~/Dropbox`) is:
     - Present
     - Browsable
   - Optionally perform a small test change:
     - E.g., create a small file and verify Dropbox acknowledges it (if such a test is within the tool’s scope).

---

## Phase 4 – Error Handling & Reporting

**Goal:** Provide clear feedback about what succeeded and what failed.

For both OFF and ON flows, the tool/agent should:

1. **Capture errors per step**
   - Record any failures:
     - Could not find Dropbox.app
     - LaunchAgent missing
     - PlugInKit operations failed
     - Finder restart failed
   - Continue with other steps where safe, but mark failed ones in the final report.

2. **Degrade gracefully**
   - If a piece is absent (e.g. no LaunchAgent found), treat that step as “not applicable” rather than fatal.
   - Ensure the system is left in a consistent state even when some actions fail.

3. **Summarize final state**

   For **OFF**:
   - Dropbox processes running: yes/no (expected: no)
   - LaunchAgent disabled: yes/no/not applicable
   - Extensions disabled (per bundle ID): yes/no/not found

   For **ON**:
   - LaunchAgent restored: yes/no/not applicable
   - Extensions enabled (per bundle ID): yes/no/not found
   - Dropbox app running: yes/no

---

## Conceptual Summary

- **User intent:** Provide a robust, reversible “kill switch” for Dropbox on macOS so internal files can be modified without interference.
- **OFF mode:**  
  - Stop processes → disable LaunchAgent → disable extensions → verify everything is inert.
- **ON mode:**  
  - Restore LaunchAgent → re-enable extensions → relaunch app → verify functionality.

This document is intended as a **behavioral spec** for a human implementer or AI agent to build a higher-level “Dropbox Kill Switch” tool.
