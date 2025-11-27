use crate::discovery;
use crate::extensions::{self, ExtensionState};
use crate::launchagent;
use crate::processes::{self, DropboxProcess};
use anyhow::Result;
use std::path::PathBuf;
use tracing::info;

pub use crate::launchagent::LaunchAgentState;

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

    // App location
    match &status.dropbox_app_path {
        Some(path) => info!("Dropbox.app: {}", path.display()),
        None => info!("Dropbox.app: NOT FOUND"),
    }
    info!("");

    // Running processes
    info!("Running processes:");
    if status.processes.is_empty() {
        info!("  (none)");
    } else {
        for proc in &status.processes {
            info!("  PID {}: {}", proc.pid, proc.name);
        }
    }
    info!("");

    // LaunchAgent state
    let la_state = match status.launch_agent_state {
        LaunchAgentState::Enabled => "enabled",
        LaunchAgentState::Disabled => "disabled",
        LaunchAgentState::Missing => "MISSING",
    };
    info!("LaunchAgent: {}", la_state);
    info!("");

    // Extensions
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
