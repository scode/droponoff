use anyhow::{Context, Result};
use duct::cmd;

pub fn restart_finder() -> Result<()> {
    cmd!("killall", "Finder")
        .stdout_null()
        .stderr_null()
        .unchecked() // Finder may not be running
        .run()
        .context("Failed to restart Finder")?;
    Ok(())
}
