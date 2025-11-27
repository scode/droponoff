#[cfg(not(target_os = "macos"))]
compile_error!("droponoff only works on macOS");

mod discovery;
mod extensions;
mod finder;
mod launchagent;
mod logging;
mod processes;
mod status;

use anyhow::Result;
use clap::{Parser, Subcommand};
use indoc::indoc;
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
    /// Disable Dropbox completely (DOES NOT WAIT FOR SYNCHRONIZATION TO FINISH).
    Off,
    /// Show current Dropbox state (read-only)
    Status,
    #[command(
        about = "DANGEROUS: Delete scratch_files contents after ensuring Dropbox is stopped (READ FULL HELP).",
        long_about = indoc! {r#"
            DANGEROUS: Delete scratch_files contents after ensuring Dropbox is stopped (READ FULL HELP).

            This will nuke scratch files. It should only be used:

            - With Dropbox entirely turned off.
            - With no pending synchronization operations (especially uploads) at the time Dropbox was turned off.

            If this is run while uploads are occurring it is highly likely to lead to data loss. Even if used as
            recommended, this command is risky and is not in any way supported by Dropbox or the author of this tool.
        "#}
    )]
    NukeScratch,
}

fn main() -> Result<()> {
    logging::init_logging();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Off => cmd_off(),
        Commands::On => cmd_on(),
        Commands::Status => cmd_status(),
        Commands::NukeScratch => cmd_nuke_scratch(),
    };

    if let Err(ref e) = result {
        error!("{}", e);
    }

    result
}

fn verify_with_retry<F>(check_fn: F, max_attempts: u32, delay_ms: u64) -> Result<()>
where
    F: Fn(&status::Status) -> bool,
{
    for attempt in 1..=max_attempts {
        let status = status::get_status()?;

        if check_fn(&status) {
            return Ok(());
        }

        if attempt < max_attempts {
            std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        }
    }

    anyhow::bail!("Verification failed after {} attempts", max_attempts)
}

fn cmd_off() -> Result<()> {
    info!("Disabling Dropbox...\n");

    info!("→ Requesting Dropbox to quit...");
    if let Err(e) = processes::quit_dropbox_gracefully() {
        warn!("  Note: {}", e);
    }

    info!("→ Disabling LaunchAgent...");
    launchagent::unload_launch_agent().ok(); // Ignore if not loaded
    launchagent::disable_launch_agent()?;

    info!("→ Disabling Dropbox extensions...");
    extensions::disable_all_extensions()?;

    // Refresh finder in the hopes it will relinquish any active
    // file provider dependencies.
    info!("→ Restarting Finder...");
    finder::restart_finder()?;

    info!("→ Waiting for non-FileProvider processes to stop...");
    processes::wait_for_non_fileprovider_processes_to_die(10)?;

    // Note: We have found no other way to gracefully terminate FileProvider processes
    // using command-line tools. SIGTERM should be reasonably safe unless the provider
    // already has other bugs - it's at the very least as graceful as an actual normal
    // crash or similar event. There used to be a fileproviderctl command to
    // remove "domains" - but the command disappeared in Sonoma.
    info!("→ Terminating DropboxFileProvider processes...");
    processes::kill_fileprovider_processes()?;

    info!("→ Waiting for all processes to stop...");
    processes::wait_for_processes_to_die(10)?;

    info!("→ Checking status...");
    verify_with_retry(
        |status| {
            let mut verified = true;

            if !status.processes.is_empty() {
                warn!("  Still running: {} process(es)", status.processes.len());
                verified = false;
            }

            if status.launch_agent_state != status::LaunchAgentState::Disabled {
                warn!("  LaunchAgent state: {:?}", status.launch_agent_state);
                verified = false;
            }

            let enabled_exts: Vec<_> = status
                .extensions
                .iter()
                .filter(|(_, s)| s.enabled)
                .map(|(name, _)| name.as_str())
                .collect();
            if !enabled_exts.is_empty() {
                warn!("  Extensions still enabled: {}", enabled_exts.join(", "));
                verified = false;
            }

            verified
        },
        5,
        500,
    )?;

    info!("");
    info!("✓ Dropbox is now OFF");
    Ok(())
}

fn cmd_on() -> Result<()> {
    info!("Enabling Dropbox...\n");

    info!("→ Restoring LaunchAgent...");
    launchagent::enable_launch_agent()?;
    launchagent::load_launch_agent()?;

    info!("→ Enabling Dropbox extensions...");
    extensions::enable_all_extensions()?;

    info!("→ Launching Dropbox...");
    processes::launch_dropbox()?;

    info!("→ Waiting for Dropbox to start...");
    processes::wait_for_dropbox_to_start(10)?;

    info!("→ Checking status...");
    verify_with_retry(
        |status| {
            let mut verified = true;

            if status.processes.is_empty() {
                warn!("  No processes running yet");
                verified = false;
            }

            if status.launch_agent_state != status::LaunchAgentState::Enabled {
                warn!("  LaunchAgent state: {:?}", status.launch_agent_state);
                verified = false;
            }

            let disabled_exts: Vec<_> = status
                .extensions
                .iter()
                .filter(|(bundle_id, s)| {
                    // Exclude garcon from the check (seems to have to do with
                    // old school non-file system provider operation)
                    *bundle_id != "com.getdropbox.dropbox.garcon" && !s.enabled
                })
                .map(|(name, _)| name.as_str())
                .collect();
            if !disabled_exts.is_empty() {
                warn!("  Extensions still disabled: {}", disabled_exts.join(", "));
                verified = false;
            }

            verified
        },
        5,
        500,
    )?;

    info!("");
    info!("✓ Dropbox is now ON");
    Ok(())
}

fn cmd_status() -> Result<()> {
    let status = status::get_status()?;
    status::print_status(&status);
    Ok(())
}

fn cmd_nuke_scratch() -> Result<()> {
    info!("Deleting scratch_files contents...\n");

    info!("→ Checking Dropbox status...");
    let status = status::get_status()?;
    if !status.processes.is_empty() {
        anyhow::bail!(
            "Dropbox appears to be running. Run `droponoff status` to check and `droponoff off` to turn it off."
        );
    }

    info!("→ Cleaning scratch_files directories...");
    status::clean_scratch_files()?;

    info!("");
    info!("✓ scratch_files contents deleted");
    Ok(())
}
