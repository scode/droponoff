use anyhow::{Context, Result};
use duct::cmd;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DropboxProcess {
    pub pid: u32,
    pub name: String,
}

struct DropboxProcessLists {
    all: Vec<DropboxProcess>,
    fileprovider: Vec<DropboxProcess>,
    non_fileprovider: Vec<DropboxProcess>,
}

fn list_all_dropbox_processes() -> Result<DropboxProcessLists> {
    let user = std::env::var("USER").context("Could not get USER environment variable")?;

    let output = cmd!("pgrep", "-l", "-u", &user, "-f", "Dropbox")
        .stdout_capture()
        .stderr_capture()
        .unchecked()
        .run()?;

    // pgrep exit codes:
    // 0: One or more processes matched
    // 1: No processes matched
    // 2: Syntax error in the command line
    // 3: Fatal error (e.g., out of memory)
    if !output.status.success() {
        let exit_code = output.status.code().unwrap_or(-1);
        if exit_code == 1 {
            // No processes found, which is fine
            return Ok(DropboxProcessLists {
                all: Vec::new(),
                fileprovider: Vec::new(),
                non_fileprovider: Vec::new(),
            });
        } else {
            anyhow::bail!("pgrep failed with exit code {}", exit_code);
        }
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut all = Vec::new();
    let mut fileprovider = Vec::new();
    let mut non_fileprovider = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        if parts.len() >= 2 {
            if let Ok(pid) = parts[0].parse::<u32>() {
                let name = parts[1].to_string();
                let process = DropboxProcess {
                    pid,
                    name: name.clone(),
                };

                all.push(process.clone());

                if name.contains("DropboxFileProvider") {
                    fileprovider.push(process);
                } else {
                    non_fileprovider.push(process);
                }
            }
        }
    }

    Ok(DropboxProcessLists {
        all,
        fileprovider,
        non_fileprovider,
    })
}

pub fn list_dropbox_processes() -> Result<Vec<DropboxProcess>> {
    Ok(list_all_dropbox_processes()?.all)
}

pub fn quit_dropbox_gracefully() -> Result<()> {
    let script = r#"tell application "Dropbox" to quit"#;
    cmd!("osascript", "-e", script)
        .stdout_null()
        .stderr_null()
        .unchecked()
        .run()
        .context("Failed to send quit request to Dropbox")?;
    Ok(())
}

pub fn wait_for_processes_to_die(timeout_secs: u64) -> Result<()> {
    wait_for_processes_to_die_impl(timeout_secs, false)
}

pub fn wait_for_non_fileprovider_processes_to_die(timeout_secs: u64) -> Result<()> {
    wait_for_processes_to_die_impl(timeout_secs, true)
}

fn wait_for_processes_to_die_impl(timeout_secs: u64, exclude_fileprovider: bool) -> Result<()> {
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    loop {
        let process_lists = list_all_dropbox_processes()?;
        let processes = if exclude_fileprovider {
            &process_lists.non_fileprovider
        } else {
            &process_lists.all
        };

        if processes.is_empty() {
            return Ok(());
        }

        if start.elapsed() > timeout {
            let process_type = if exclude_fileprovider {
                "non-FileProvider Dropbox"
            } else {
                "Dropbox"
            };
            anyhow::bail!(
                "Timeout waiting for {} processes to stop. {} still running: {:?}",
                process_type,
                processes.len(),
                processes.iter().map(|p| &p.name).collect::<Vec<_>>()
            );
        }

        thread::sleep(Duration::from_millis(100));
    }
}

pub fn launch_dropbox() -> Result<()> {
    cmd!("open", "-a", "Dropbox")
        .stdout_null()
        .stderr_null()
        .run()
        .context("Failed to launch Dropbox")?;
    Ok(())
}

pub fn wait_for_dropbox_to_start(timeout_secs: u64) -> Result<()> {
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    loop {
        let processes = list_dropbox_processes()?;
        if !processes.is_empty() {
            return Ok(());
        }

        if start.elapsed() > timeout {
            anyhow::bail!("Timeout waiting for Dropbox to start");
        }

        thread::sleep(Duration::from_millis(100));
    }
}

pub fn kill_fileprovider_processes() -> Result<()> {
    let process_lists = list_all_dropbox_processes()?;

    for process in &process_lists.fileprovider {
        let _ = cmd!("kill", process.pid.to_string())
            .stdout_null()
            .stderr_null()
            .unchecked()
            .run();
        // Ignore errors - process may have already exited
    }

    Ok(())
}
