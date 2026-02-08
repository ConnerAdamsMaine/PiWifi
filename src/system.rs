use anyhow::{Context, Result};
use std::process::Command;

pub struct SystemCommand;

impl SystemCommand {
    /// Execute a shell command (requires elevated privileges in some cases)
    pub fn run(cmd: &str, args: &[&str]) -> Result<String> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!("{} {}", cmd, args.join(" ")))
            .output()
            .context("Failed to execute command")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Command failed: {} {}",
                cmd,
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Run a command with sudo (requires passwordless sudo)
    pub fn run_sudo(cmd: &str, args: &[&str]) -> Result<String> {
        let full_cmd = format!("sudo {} {}", cmd, args.join(" "));
        let output = Command::new("sh")
            .arg("-c")
            .arg(&full_cmd)
            .output()
            .context("Failed to execute sudo command")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
