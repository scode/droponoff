#[cfg(not(target_os = "macos"))]
compile_error!("droponoff only works on macOS");

mod discovery;
mod extensions;
mod finder;
mod launchagent;
mod logging;
mod processes;
mod status;
mod sudo;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{error, info, warn};

#[derive(Parser)]
#[command(name = "droponoff")]
#[command(about = "A reversible kill switch for Dropbox on macOS")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Restore Dropbox to normal operation
    On,
    /// Disable Dropbox completely
    Off,
    /// Show current Dropbox state (read-only)
    Status,
}

fn main() -> Result<()> {
    // Initialize tracing with colored output
    logging::init_logging();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Off => cmd_off(),
        Commands::On => cmd_on(),
        Commands::Status => cmd_status(),
    };

    if let Err(ref e) = result {
        error!("{}", e);
    }

    result
}

fn cmd_off() -> Result<()> {
    info!("Disabling Dropbox...\n");

    // Step 1: Request Dropbox to quit gracefully
    info!("→ Requesting Dropbox to quit...");
    if let Err(e) = processes::quit_dropbox_gracefully() {
        warn!("  Note: {}", e);
    }

    // Step 2: Disable LaunchAgent
    info!("→ Disabling LaunchAgent...");
    launchagent::unload_launch_agent().ok(); // Ignore if not loaded
    launchagent::disable_launch_agent()?;

    // Step 3: Disable extensions
    info!("→ Disabling Dropbox extensions...");
    extensions::disable_all_extensions()?;

    // Step 4: Refresh Finder
    info!("→ Restarting Finder...");
    finder::restart_finder()?;

    // Step 5: Wait for all processes to stop
    info!("→ Waiting for processes to stop...");
    processes::wait_for_processes_to_die(10)?;

    // Step 6: Verify
    info!("→ Verifying...");
    let status = status::get_status()?;

    info!("");
    if status.processes.is_empty()
        && status.launch_agent_state == status::LaunchAgentState::Disabled
        && status.extensions.iter().all(|(_, s)| !s.enabled)
    {
        info!("✓ Dropbox is now OFF");
        Ok(())
    } else {
        anyhow::bail!("Verification failed - Dropbox may not be fully disabled");
    }
}

fn cmd_on() -> Result<()> {
    info!("Enabling Dropbox...\n");

    // Step 1: Restore LaunchAgent
    info!("→ Restoring LaunchAgent...");
    launchagent::enable_launch_agent()?;
    launchagent::load_launch_agent()?;

    // Step 2: Enable extensions
    info!("→ Enabling Dropbox extensions...");
    extensions::enable_all_extensions()?;

    // Step 3: Refresh Finder
    info!("→ Restarting Finder...");
    finder::restart_finder()?;

    // Step 4: Launch Dropbox
    info!("→ Launching Dropbox...");
    processes::launch_dropbox()?;

    info!("→ Waiting for Dropbox to start...");
    processes::wait_for_dropbox_to_start(10)?;

    // Step 5: Verify
    info!("→ Verifying...");
    let status = status::get_status()?;

    info!("");
    if !status.processes.is_empty()
        && status.launch_agent_state == status::LaunchAgentState::Enabled
        && status.extensions.iter().all(|(_, s)| s.enabled)
    {
        info!("✓ Dropbox is now ON");
        Ok(())
    } else {
        anyhow::bail!("Verification failed - Dropbox may not be fully enabled");
    }
}

fn cmd_status() -> Result<()> {
    let status = status::get_status()?;
    status::print_status(&status);
    Ok(())
}
