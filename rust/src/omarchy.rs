use anyhow::{anyhow, Result};
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use crate::config::ResolvedConfig;
use crate::paths::resolve_link_target;
use rand::random;

#[derive(Debug, Clone)]
pub struct RestartCommand {
  pub cmd: String,
  pub args: Vec<String>,
}

pub fn command_exists(cmd: &str) -> bool {
  which::which(cmd).is_ok()
}

fn awww_daemon_running() -> bool {
  if !command_exists("pgrep") {
    return false;
  }
  Command::new("pgrep")
    .args(["-x", "awww-daemon"])
    .status()
    .map(|status| status.success())
    .unwrap_or(false)
}

fn awww_socket_path() -> Option<PathBuf> {
  let runtime = env::var("XDG_RUNTIME_DIR").ok()?;
  let display = env::var("WAYLAND_DISPLAY").ok()?;
  Some(PathBuf::from(runtime).join(format!("{display}-awww-daemon..sock")))
}

pub fn ensure_awww_daemon(config: &ResolvedConfig, quiet: bool) {
  if !config.awww_transition || !config.awww_auto_start {
    return;
  }
  if !command_exists("awww") {
    return;
  }
  if !command_exists("awww-daemon") {
    if !quiet {
      eprintln!("theme-manager: awww-daemon not found in PATH");
    }
    return;
  }
  if awww_daemon_running() {
    return;
  }
  if !quiet {
    eprintln!("theme-manager: starting awww-daemon for transitions");
  }
  let _ = Command::new("awww-daemon").spawn();
  if let Some(socket_path) = awww_socket_path() {
    for _ in 0..40 {
      if socket_path.exists() {
        return;
      }
      thread::sleep(Duration::from_millis(50));
    }
    if !quiet {
      eprintln!(
        "theme-manager: awww-daemon socket not found at {}",
        socket_path.to_string_lossy()
      );
    }
  } else {
    thread::sleep(Duration::from_millis(200));
  }
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

pub fn reload_components(quiet: bool, waybar_restart: Option<RestartCommand>) -> Result<()> {
  run_optional("omarchy-restart-terminal", &[], quiet)?;
  if let Some(restart) = waybar_restart {
    let arg_refs: Vec<&str> = restart.args.iter().map(|arg| arg.as_str()).collect();
    run_command(&restart.cmd, &arg_refs, quiet)?;
  } else {
    run_optional("omarchy-restart-waybar", &[], quiet)?;
  }
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

pub fn run_awww_transition(config: &ResolvedConfig, quiet: bool, debug_awww: bool) -> Result<()> {
  if !config.awww_transition {
    return Ok(());
  }
  if !command_exists("awww") {
    return Ok(());
  }

  let background = resolve_background(&config.current_background_link)?;
  let Some(background) = background else {
    return Ok(());
  };
  if !background.is_file() {
    return Ok(());
  }

  let angle = if random::<bool>() {
    config.awww_transition_angle
  } else {
    -config.awww_transition_angle
  };
  let args = vec![
    "img".to_string(),
    background.to_string_lossy().to_string(),
    "--transition-type".to_string(),
    config.awww_transition_type.clone(),
    "--transition-duration".to_string(),
    format!("{}", config.awww_transition_duration),
    format!("--transition-angle={}", angle),
    "--transition-fps".to_string(),
    format!("{}", config.awww_transition_fps),
    "--transition-pos".to_string(),
    config.awww_transition_pos.clone(),
    "--transition-bezier".to_string(),
    config.awww_transition_bezier.clone(),
    "--transition-wave".to_string(),
    config.awww_transition_wave.clone(),
  ];

  if debug_awww {
    eprintln!("theme-manager: awww cmd: awww {}", args.join(" "));
  }
  match Command::new("awww").args(&args).output() {
    Ok(output) if output.status.success() => Ok(()),
    Ok(output) => {
      let stderr = String::from_utf8_lossy(&output.stderr);
      let socket_error = stderr.contains("awww-daemon") || stderr.contains("Socket file");
      if socket_error {
        if config.awww_auto_start {
          if !quiet {
            eprintln!("theme-manager: starting awww-daemon for transitions");
          }
          if !command_exists("awww-daemon") {
            if !quiet {
              eprintln!("theme-manager: awww-daemon not found in PATH");
            }
            return Ok(());
          }
          let _ = Command::new("awww-daemon").spawn();
          let mut last_err = String::new();
          for _ in 0..60 {
            thread::sleep(Duration::from_millis(50));
            let retry = Command::new("awww").args(&args).output();
            if let Ok(retry) = retry {
              if retry.status.success() {
                return Ok(());
              }
              let retry_err = String::from_utf8_lossy(&retry.stderr);
              last_err = retry_err.to_string();
              if !(retry_err.contains("awww-daemon") || retry_err.contains("Socket file")) {
                break;
              }
            }
          }
          if !quiet && !last_err.is_empty() {
            eprintln!("theme-manager: awww transition retry failed: {last_err}");
          }
        }
        if !quiet {
          eprintln!("theme-manager: awww-daemon not running; skipping transition");
        }
      } else if !quiet {
        eprintln!("theme-manager: awww transition failed");
      }
      Ok(())
    }
    Err(err) => {
      if !quiet {
        eprintln!("theme-manager: awww transition failed: {err}");
      }
      Ok(())
    }
  }
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

fn resolve_background(link_path: &Path) -> Result<Option<PathBuf>> {
  if !link_path.exists() {
    return Ok(None);
  }
  if link_path.is_symlink() {
    return Ok(Some(resolve_link_target(link_path)?));
  }
  Ok(Some(link_path.to_path_buf()))
}
