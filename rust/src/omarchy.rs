use anyhow::{anyhow, Result};
use std::path::Path;
use std::process::{Command, Stdio};

pub fn command_exists(cmd: &str) -> bool {
  which::which(cmd).is_ok()
}

pub fn run_required(cmd: &str, args: &[&str], quiet: bool) -> Result<()> {
  if !command_exists(cmd) {
    return Err(anyhow!("{cmd} not found in PATH"));
  }
  run_command(cmd, args, quiet)
}

pub fn run_optional(cmd: &str, args: &[&str], quiet: bool) -> Result<()> {
  if !command_exists(cmd) {
    if !quiet {
      eprintln!("theme-manager: {cmd} not found in PATH");
    }
    return Ok(());
  }
  run_command(cmd, args, quiet)
}

pub fn run_command(cmd: &str, args: &[&str], quiet: bool) -> Result<()> {
  let mut command = Command::new(cmd);
  command.args(args);
  if quiet {
    command.stdout(Stdio::null()).stderr(Stdio::null());
  }
  let status = command.status()?;
  if !status.success() {
    return Err(anyhow!("{cmd} exited with {status}"));
  }
  Ok(())
}

pub fn reload_components(quiet: bool) -> Result<()> {
  run_optional("omarchy-restart-terminal", &[], quiet)?;
  run_optional("omarchy-restart-waybar", &[], quiet)?;
  run_optional("omarchy-restart-swayosd", &[], quiet)?;
  run_optional("hyprctl", &["reload"], quiet)?;
  run_optional("makoctl", &["reload"], quiet)?;
  if command_exists("pkill") {
    let _ = run_command("pkill", &["-SIGUSR2", "btop"], true);
  }
  Ok(())
}

pub fn apply_theme_setters(quiet: bool) -> Result<()> {
  run_optional("omarchy-theme-set-gnome", &[], quiet)?;
  run_optional("omarchy-theme-set-browser", &[], quiet)?;
  run_optional("omarchy-theme-set-vscode", &[], quiet)?;
  run_optional("omarchy-theme-set-cursor", &[], quiet)?;
  run_optional("omarchy-theme-set-obsidian", &[], quiet)?;
  Ok(())
}

pub fn run_hook(hook_path: &Path, args: &[&str], quiet: bool) -> Result<()> {
  if !hook_path.is_file() {
    return Ok(());
  }
  let mut command = Command::new(hook_path);
  command.args(args);
  if quiet {
    command.stdout(Stdio::null()).stderr(Stdio::null());
  }
  let _ = command.status();
  Ok(())
}
