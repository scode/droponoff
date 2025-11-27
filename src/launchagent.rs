use crate::discovery;
use anyhow::{Context, Result};
use duct::cmd;
use std::fs;
use tracing::info;

fn get_user_id() -> Result<String> {
    let output = cmd!("id", "-u")
        .stdout_capture()
        .run()
        .context("Failed to get user ID")?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn unload_launch_agent() -> Result<()> {
    let uid = get_user_id()?;

    let service_target = format!("gui/{}/com.dropbox.DropboxMacUpdate.agent", uid);

    cmd!("launchctl", "bootout", &service_target)
        .stdout_null()
        .stderr_null()
        .unchecked()
        .run()
        .context("Failed to unload LaunchAgent")?;

    Ok(())
}

pub fn load_launch_agent() -> Result<()> {
    let path = discovery::get_launch_agent_path()?;
    let uid = get_user_id()?;

    if !path.exists() {
        anyhow::bail!("LaunchAgent plist not found at {:?}", path);
    }

    let domain_target = format!("gui/{}", uid);

    cmd!("launchctl", "bootstrap", &domain_target, &path)
        .stdout_null()
        .stderr_null()
        .unchecked()
        .run()
        .context("Failed to load LaunchAgent")?;

    Ok(())
}

pub fn disable_launch_agent() -> Result<()> {
    let enabled_path = discovery::get_launch_agent_path()?;
    let disabled_path = discovery::get_launch_agent_disabled_path()?;

    if disabled_path.exists() {
        info!("  LaunchAgent already disabled");
        return Ok(());
    }

    if !enabled_path.exists() {
        anyhow::bail!("LaunchAgent plist not found at {:?}", enabled_path);
    }

    fs::rename(&enabled_path, &disabled_path).context("Failed to rename LaunchAgent plist")?;
    info!("  Renamed {:?} → {:?}", enabled_path, disabled_path);
    Ok(())
}

pub fn enable_launch_agent() -> Result<()> {
    let enabled_path = discovery::get_launch_agent_path()?;
    let disabled_path = discovery::get_launch_agent_disabled_path()?;

    if enabled_path.exists() {
        info!("  LaunchAgent already enabled");
        return Ok(());
    }

    if !disabled_path.exists() {
        anyhow::bail!(
            "LaunchAgent plist not found at {:?} or {:?}",
            enabled_path,
            disabled_path
        );
    }

    fs::rename(&disabled_path, &enabled_path).context("Failed to rename LaunchAgent plist")?;
    info!("  Renamed {:?} → {:?}", disabled_path, enabled_path);
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchAgentState {
    Enabled,
    Disabled,
    Missing,
}

pub fn get_launch_agent_state() -> Result<LaunchAgentState> {
    let enabled_path = discovery::get_launch_agent_path()?;
    let disabled_path = discovery::get_launch_agent_disabled_path()?;

    if enabled_path.exists() {
        Ok(LaunchAgentState::Enabled)
    } else if disabled_path.exists() {
        Ok(LaunchAgentState::Disabled)
    } else {
        Ok(LaunchAgentState::Missing)
    }
}
