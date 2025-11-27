use anyhow::{Context, Result};
use duct::cmd;

#[allow(dead_code)]
pub fn run_with_sudo(command: &str, args: &[&str]) -> Result<()> {
    let mut sudo_args = vec![command];
    sudo_args.extend(args);

    cmd("sudo", &sudo_args)
        .run()
        .with_context(|| format!("Failed to run 'sudo {} {:?}'", command, args))?;
    Ok(())
}

#[allow(dead_code)]
pub fn check_sudo_available() -> bool {
    cmd!("sudo", "-n", "true")
        .stdout_null()
        .stderr_null()
        .unchecked()
        .run()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
