use anyhow::{Context, Result};
use std::path::PathBuf;

pub const DROPBOX_BUNDLE_IDS: &[&str] = &[
    "com.getdropbox.dropbox.fileprovider",
    "com.getdropbox.dropbox.TransferExtension",
    "com.getdropbox.dropbox.garcon",
];

pub const LAUNCH_AGENT_NAME: &str = "com.dropbox.DropboxMacUpdate.agent.plist";

pub fn get_home_dir() -> Result<PathBuf> {
    dirs::home_dir().context("Could not determine home directory")
}

pub fn find_dropbox_app() -> Result<PathBuf> {
    let system_path = PathBuf::from("/Applications/Dropbox.app");
    if system_path.exists() {
        return Ok(system_path);
    }

    let home = get_home_dir()?;
    let user_path = home.join("Applications/Dropbox.app");
    if user_path.exists() {
        return Ok(user_path);
    }

    anyhow::bail!("Dropbox.app not found in /Applications or ~/Applications")
}

pub fn get_launch_agent_path() -> Result<PathBuf> {
    let home = get_home_dir()?;
    Ok(home.join("Library/LaunchAgents").join(LAUNCH_AGENT_NAME))
}

pub fn get_launch_agent_disabled_path() -> Result<PathBuf> {
    let home = get_home_dir()?;
    Ok(home
        .join("Library/LaunchAgents")
        .join(format!("{}.disabled", LAUNCH_AGENT_NAME)))
}
