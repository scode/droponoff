use anyhow::{Context, Result};
use duct::cmd;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DropboxProcess {
    pub pid: u32,
    pub name: String,
}

pub fn list_dropbox_processes() -> Result<Vec<DropboxProcess>> {
    let user = std::env::var("USER").context("Could not get USER environment variable")?;

    // Use pgrep to find Dropbox processes for current user
    let output = cmd!("pgrep", "-l", "-u", &user, "-f", "Dropbox")
        .stdout_capture()
        .stderr_capture()
        .unchecked()
        .run()?;

    if !output.status.success() {
        // pgrep returns 1 when no processes found, which is fine
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut processes = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        if parts.len() >= 2 {
            if let Ok(pid) = parts[0].parse::<u32>() {
                processes.push(DropboxProcess {
                    pid,
                    name: parts[1].to_string(),
                });
            }
        }
    }

    Ok(processes)
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
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    loop {
        let processes = list_dropbox_processes()?;
        if processes.is_empty() {
            return Ok(());
        }

        if start.elapsed() > timeout {
            anyhow::bail!(
                "Timeout waiting for Dropbox processes to stop. {} still running: {:?}",
                processes.len(),
                processes.iter().map(|p| &p.name).collect::<Vec<_>>()
            );
        }

        thread::sleep(Duration::from_millis(500));
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

        thread::sleep(Duration::from_millis(500));
    }
}
