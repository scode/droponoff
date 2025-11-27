use crate::discovery;
use anyhow::{Context, Result};
use duct::cmd;
use tracing::info;

#[derive(Debug, Clone)]
pub struct ExtensionState {
    #[allow(dead_code)]
    pub bundle_id: String,
    pub enabled: bool,
    pub found: bool,
}

pub fn get_extension_state(bundle_id: &str) -> Result<ExtensionState> {
    // Query pluginkit for this specific bundle ID
    let output = cmd!("pluginkit", "-m", "-i", bundle_id)
        .stdout_capture()
        .stderr_capture()
        .unchecked()
        .run()
        .context("Failed to query pluginkit")?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    if stdout.trim().is_empty() {
        return Ok(ExtensionState {
            bundle_id: bundle_id.to_string(),
            enabled: false,
            found: false,
        });
    }

    // The output format: lines start with "+" for enabled, "-" for disabled
    let enabled = stdout.trim_start().starts_with('+');

    Ok(ExtensionState {
        bundle_id: bundle_id.to_string(),
        enabled,
        found: true,
    })
}

pub fn disable_extension(bundle_id: &str) -> Result<()> {
    cmd!("pluginkit", "-e", "ignore", "-i", bundle_id)
        .stdout_null()
        .stderr_null()
        .run()
        .with_context(|| format!("Failed to disable extension {}", bundle_id))?;
    info!("  Disabled {}", bundle_id);
    Ok(())
}

pub fn enable_extension(bundle_id: &str) -> Result<()> {
    cmd!("pluginkit", "-e", "use", "-i", bundle_id)
        .stdout_null()
        .stderr_null()
        .run()
        .with_context(|| format!("Failed to enable extension {}", bundle_id))?;
    info!("  Enabled {}", bundle_id);
    Ok(())
}

pub fn disable_all_extensions() -> Result<()> {
    for bundle_id in discovery::DROPBOX_BUNDLE_IDS {
        let state = get_extension_state(bundle_id)?;
        if state.found {
            disable_extension(bundle_id)?;
        } else {
            info!("  {} not found, skipping", bundle_id);
        }
    }
    Ok(())
}

pub fn enable_all_extensions() -> Result<()> {
    for bundle_id in discovery::DROPBOX_BUNDLE_IDS {
        let state = get_extension_state(bundle_id)?;
        if state.found {
            enable_extension(bundle_id)?;
        } else {
            info!("  {} not found, skipping", bundle_id);
        }
    }
    Ok(())
}
