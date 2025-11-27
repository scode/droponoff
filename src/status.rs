use crate::discovery;
use crate::extensions::{self, ExtensionState};
use crate::launchagent;
use crate::processes::{self, DropboxProcess};
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

pub use crate::launchagent::LaunchAgentState;

const DROPBOX_TEAM_ID: &str = "G7HH3F8CAK";
const GROUP_CONTAINER_SUFFIX: &str = ".com.getdropbox.dropbox.sync";

pub struct Status {
    pub dropbox_app_path: Option<PathBuf>,
    pub processes: Vec<DropboxProcess>,
    pub launch_agent_state: LaunchAgentState,
    pub extensions: Vec<(String, ExtensionState)>,
}

pub fn get_status() -> Result<Status> {
    let dropbox_app_path = discovery::find_dropbox_app().ok();
    let processes = processes::list_dropbox_processes()?;
    let launch_agent_state = launchagent::get_launch_agent_state()?;

    let mut ext_states = Vec::new();
    for bundle_id in discovery::DROPBOX_BUNDLE_IDS {
        let state = extensions::get_extension_state(bundle_id)?;
        ext_states.push((bundle_id.to_string(), state));
    }

    Ok(Status {
        dropbox_app_path,
        processes,
        launch_agent_state,
        extensions: ext_states,
    })
}

pub fn print_status(status: &Status) {
    info!("Dropbox Status");
    info!("==============\n");

    match &status.dropbox_app_path {
        Some(path) => info!("Dropbox.app: {}", path.display()),
        None => info!("Dropbox.app: NOT FOUND"),
    }
    info!("");

    info!("Running processes:");
    if status.processes.is_empty() {
        info!("  (none)");
    } else {
        for proc in &status.processes {
            info!("  PID {}: {}", proc.pid, proc.name);
        }
    }
    info!("");

    let la_state = match status.launch_agent_state {
        LaunchAgentState::Enabled => "enabled",
        LaunchAgentState::Disabled => "disabled",
        LaunchAgentState::Missing => "MISSING",
    };
    info!("LaunchAgent: {}", la_state);
    info!("");

    info!("Extensions:");
    for (bundle_id, state) in &status.extensions {
        let status_str = if !state.found {
            "not found"
        } else if state.enabled {
            "enabled"
        } else {
            "disabled"
        };
        info!("  {}: {}", bundle_id, status_str);
    }
}

/// Delete immediate children inside any scratch_files directories under the Dropbox root mount.
pub fn clean_scratch_files() -> Result<()> {
    let home = discovery::get_home_dir()?;
    let data_home = PathBuf::from("/System/Volumes/Data")
        .join(home.strip_prefix(Path::new("/")).unwrap_or(&home));
    let base_home = if data_home.exists() { data_home } else { home };

    let container_name = format!("{}{}", DROPBOX_TEAM_ID, GROUP_CONTAINER_SUFFIX);
    let root_mount = base_home
        .join("Library/Group Containers")
        .join(container_name)
        .join("root-mount");

    if !root_mount.exists() {
        anyhow::bail!("Dropbox root-mount not found at {}", root_mount.display());
    }

    // Find files immediately inside a directory like this and delete them:
    //
    // System/Volumes/Data/USERNAME/scode/Library/Group Containers/G7HH3F8CAK.com.getdropbox.dropbox.sync/root-mount/UUID/scratch_files
    let mut found_any = false;
    for entry in fs::read_dir(&root_mount)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        let scratch_dir = entry.path().join("scratch_files");
        if !scratch_dir.is_dir() {
            continue;
        }

        found_any = true;
        info!("  Cleaning {}", scratch_dir.display());

        // Only remove immediate files/symlinks; skip nested
        // directories as we don't expect them.
        for child in fs::read_dir(&scratch_dir)? {
            let child = child?;
            let child_type = child.file_type()?;
            let child_path = child.path();

            if child_type.is_file() || child_type.is_symlink() {
                info!("    rm {}", child_path.display());
                fs::remove_file(&child_path)?;
            } else {
                info!("    Skipping directory {}", child_path.display());
            }
        }
    }

    if !found_any {
        info!(
            "  No scratch_files directories found under {}",
            root_mount.display()
        );
    }

    Ok(())
}
