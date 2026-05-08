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

#[derive(Debug, Clone)]
pub enum RestartAction {
    Command(RestartCommand),
    WaybarExec {
        config_path: PathBuf,
        style_path: PathBuf,
    },
}

pub fn command_exists(cmd: &str) -> bool {
    which::which(cmd).is_ok()
}

pub fn detect_omarchy_root(config: &ResolvedConfig) -> Option<PathBuf> {
    if let Ok(path) = env::var("OMARCHY_PATH") {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return Some(PathBuf::from(trimmed));
        }
    }
    if let Some(bin_dir) = &config.omarchy_bin_dir {
        if let Some(parent) = bin_dir.parent() {
            return Some(parent.to_path_buf());
        }
    }
    env::var("HOME")
        .ok()
        .map(|home| PathBuf::from(home).join(".local/share/omarchy"))
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

pub fn ensure_awww_daemon(config: &ResolvedConfig, quiet: bool) {
    if !config.awww_transition {
        return;
    }
    if !command_exists("awww") {
        return;
    }
    if !command_exists("awww-daemon") {
        notify_awww_unavailable(quiet);
        if !quiet {
            eprintln!("theme-manager: awww-daemon not found in PATH");
        }
        return;
    }
    if !awww_daemon_running() {
        notify_awww_unavailable(quiet);
        if !quiet {
            eprintln!("theme-manager: awww-daemon not running; skipping transition");
        }
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

/// Attempt `omarchy <group> <cmd>` via the unified CLI (v3.7.0+).
/// Returns Ok(Some(())) on success, Ok(None) when omarchy is absent or the
/// subcommand is unregistered (exit 127), and Err on a genuine failure.
/// Stderr is always captured so that "Unknown Omarchy command" probe noise
/// never reaches the user; it is re-emitted only on genuine failures.
fn try_omarchy_unified(
    group: &str,
    cmd: &str,
    args: &[&str],
    quiet: bool,
    emit_stderr: bool,
) -> Result<Option<()>> {
    if !command_exists("omarchy") {
        return Ok(None);
    }
    let mut all_args = vec![group, cmd];
    all_args.extend_from_slice(args);
    let mut command = Command::new("omarchy");
    command.args(&all_args);
    command.stderr(Stdio::piped());
    if quiet {
        command.stdout(Stdio::null());
    }
    let output = command.output()?;
    if output.status.success() {
        return Ok(Some(()));
    }
    // Exit 127 = unknown subcommand; fall back to legacy silently.
    if output.status.code() == Some(127) {
        return Ok(None);
    }
    if emit_stderr && !quiet && !output.stderr.is_empty() {
        let _ = std::io::Write::write_all(&mut std::io::stderr(), &output.stderr);
    }
    Err(anyhow!("omarchy exited with {}", output.status))
}

/// Run an omarchy subcommand, preferring `omarchy <group> <cmd>` (v3.7.0+)
/// and falling back to the legacy `omarchy-<group>-<cmd>` script. Silently
/// skips if neither is found.
pub fn run_omarchy_optional(group: &str, cmd: &str, args: &[&str], quiet: bool) -> Result<()> {
    match try_omarchy_unified(group, cmd, args, quiet, false) {
        Ok(Some(())) => return Ok(()),
        Ok(None) => {}
        Err(err) => {
            if !quiet {
                eprintln!(
                    "theme-manager: optional omarchy {group} {cmd} failed; continuing: {err}"
                );
            }
            return Ok(());
        }
    }

    let legacy = format!("omarchy-{group}-{cmd}");
    if !command_exists(&legacy) {
        return Ok(());
    }
    if let Err(err) = run_command(&legacy, args, quiet) {
        if !quiet {
            eprintln!("theme-manager: optional {legacy} failed; continuing: {err}");
        }
    }
    Ok(())
}

/// Same as `run_omarchy_optional` but fails if neither the unified CLI
/// subcommand nor the legacy script is found in PATH.
pub fn run_omarchy_required(group: &str, cmd: &str, args: &[&str], quiet: bool) -> Result<()> {
    if try_omarchy_unified(group, cmd, args, quiet, true)?.is_some() {
        return Ok(());
    }
    let legacy = format!("omarchy-{group}-{cmd}");
    run_required(&legacy, args, quiet)
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

pub fn stop_swaybg() {
    if command_exists("pkill") {
        let _ = run_command("pkill", &["-x", "swaybg"], true);
    }
}

pub fn reload_components(
    quiet: bool,
    waybar_restart: Option<RestartAction>,
    waybar_restart_logs: bool,
) -> Result<()> {
    run_omarchy_optional("restart", "terminal", &[], quiet)?;
    restart_waybar_only(quiet, waybar_restart, waybar_restart_logs)?;
    restart_walker_only(quiet)?;
    restart_hyprlock_only(quiet)?;
    restart_swayosd(quiet)?;
    run_omarchy_optional("restart", "hyprctl", &[], quiet)?;
    run_optional("hyprctl", &["reload"], quiet)?;
    run_omarchy_optional("restart", "btop", &[], quiet)?;
    run_omarchy_optional("restart", "opencode", &[], quiet)?;
    run_omarchy_optional("restart", "mako", &[], quiet)?;
    run_omarchy_optional("restart", "helix", &[], quiet)?;
    reload_notifications(quiet);
    if command_exists("pkill") {
        let _ = run_command("pkill", &["-SIGUSR2", "btop"], true);
    }
    Ok(())
}

pub fn restart_walker_only(quiet: bool) -> Result<()> {
    if command_exists("pkill") {
        let _ = run_command("pkill", &["-f", "walker --gapplication-service"], true);
        let _ = run_command("pkill", &["-x", "walker"], true);
    }
    run_omarchy_optional("restart", "walker", &[], quiet)
}

pub fn restart_hyprlock_only(quiet: bool) -> Result<()> {
    let _ = quiet;
    if command_exists("pkill") {
        let _ = run_command("pkill", &["-x", "hyprlock"], true);
    }
    // Omarchy launches hyprlock on demand; no restart helper exists or is needed.
    Ok(())
}

pub fn restart_waybar_only(
    quiet: bool,
    waybar_restart: Option<RestartAction>,
    waybar_restart_logs: bool,
) -> Result<()> {
    if let Some(restart) = waybar_restart {
        let waybar_quiet = quiet || !waybar_restart_logs;
        match restart {
            RestartAction::Command(restart) => {
                let arg_refs: Vec<&str> = restart.args.iter().map(|arg| arg.as_str()).collect();
                run_command(&restart.cmd, &arg_refs, waybar_quiet)?;
            }
            RestartAction::WaybarExec {
                config_path,
                style_path,
            } => {
                restart_waybar_exec(&config_path, &style_path, waybar_quiet)?;
            }
        }
    } else {
        run_omarchy_optional("restart", "waybar", &[], quiet)?;
    }
    Ok(())
}

fn pgrep_pids(name: &str) -> Option<Vec<String>> {
    if !command_exists("pgrep") {
        return None;
    }
    let output = Command::new("pgrep").args(["-x", name]).output().ok()?;
    if !output.status.success() {
        return Some(Vec::new());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let pids: Vec<String> = stdout
        .split_whitespace()
        .map(|pid| pid.to_string())
        .collect();
    Some(pids)
}

fn start_swayosd(quiet: bool) -> Result<()> {
    if !command_exists("swayosd-server") {
        return Ok(());
    }

    let use_setsid = command_exists("setsid");
    let use_uwsm = command_exists("uwsm-app");

    let mut candidates: Vec<Vec<String>> = Vec::new();
    if use_setsid && use_uwsm {
        candidates.push(vec![
            "setsid".to_string(),
            "uwsm-app".to_string(),
            "--".to_string(),
            "swayosd-server".to_string(),
        ]);
    }
    if use_uwsm {
        candidates.push(vec![
            "uwsm-app".to_string(),
            "--".to_string(),
            "swayosd-server".to_string(),
        ]);
    }
    if use_setsid {
        candidates.push(vec!["setsid".to_string(), "swayosd-server".to_string()]);
    }
    candidates.push(vec!["swayosd-server".to_string()]);

    for parts in candidates {
        let mut iter = parts.iter();
        let Some(cmd) = iter.next() else { continue };
        let mut command = Command::new(cmd);
        command.args(iter);
        if quiet {
            command.stdout(Stdio::null()).stderr(Stdio::null());
        }
        match command.spawn() {
            Ok(mut child) => {
                thread::sleep(Duration::from_millis(120));
                match child.try_wait() {
                    Ok(Some(status)) => {
                        if status.success() {
                            return Ok(());
                        }
                    }
                    Ok(None) => return Ok(()),
                    Err(_) => {}
                }
            }
            Err(_) => {}
        }
    }

    Ok(())
}

fn restart_swayosd(quiet: bool) -> Result<()> {
    let before = pgrep_pids("swayosd-server");
    if let Err(err) = run_omarchy_optional("restart", "swayosd", &[], quiet) {
        if !quiet {
            eprintln!("theme-manager: swayosd restart command failed: {err}");
        }
    }
    let after = pgrep_pids("swayosd-server");

    if let (Some(before), Some(after)) = (&before, &after) {
        if !before.is_empty() && before != after {
            return Ok(());
        }
    }

    if command_exists("pkill") {
        let _ = run_command("pkill", &["-x", "swayosd-server"], true);
        thread::sleep(Duration::from_millis(120));
    }

    if let Some(pids) = pgrep_pids("swayosd-server") {
        if !pids.is_empty() {
            return Ok(());
        }
    }

    start_swayosd(quiet)?;
    Ok(())
}

fn reload_notifications(quiet: bool) {
    let swaync_running = pgrep_pids("swaync")
        .map(|pids| !pids.is_empty())
        .unwrap_or(false);
    let mako_running = pgrep_pids("mako")
        .map(|pids| !pids.is_empty())
        .unwrap_or(false);

    if swaync_running {
        reload_swaync(quiet, true);
    }
    if mako_running {
        reload_mako(quiet, true);
    }
    if swaync_running || mako_running {
        return;
    }

    if command_exists("swaync-client") {
        reload_swaync(true, false);
    }
    if command_exists("makoctl") {
        reload_mako(true, false);
    }
}

fn reload_swaync(quiet: bool, warn: bool) {
    if !command_exists("swaync-client") {
        if warn && !quiet {
            eprintln!("theme-manager: swaync reload skipped: swaync-client not found in PATH");
        }
        return;
    }

    if let Err(err) = run_command("swaync-client", &["--reload-config"], quiet) {
        if warn && !quiet {
            eprintln!("theme-manager: swaync reload skipped: {err}");
        }
    }
}

fn reload_mako(quiet: bool, warn: bool) {
    if !command_exists("makoctl") {
        if warn && !quiet {
            eprintln!("theme-manager: mako reload skipped: makoctl not found in PATH");
        }
        return;
    }

    if let Err(err) = run_command("makoctl", &["reload"], quiet) {
        if warn && !quiet {
            eprintln!("theme-manager: mako reload skipped: {err}");
        }
    }
}

fn restart_waybar_exec(config_path: &Path, style_path: &Path, quiet: bool) -> Result<()> {
    let mut waybar_args = Vec::new();
    if !config_path.as_os_str().is_empty() {
        waybar_args.push("-c".to_string());
        waybar_args.push(config_path.to_string_lossy().to_string());
    }
    if !style_path.as_os_str().is_empty() {
        waybar_args.push("-s".to_string());
        waybar_args.push(style_path.to_string_lossy().to_string());
    }

    if command_exists("pkill") {
        let _ = Command::new("pkill").args(["-x", "waybar"]).status();
    }

    let use_setsid = command_exists("setsid");
    let use_uwsm = command_exists("uwsm-app");

    let mut candidates: Vec<Vec<String>> = Vec::new();
    if use_setsid && use_uwsm {
        let mut cmd = vec![
            "setsid".to_string(),
            "uwsm-app".to_string(),
            "--".to_string(),
        ];
        cmd.push("waybar".to_string());
        cmd.extend(waybar_args.clone());
        candidates.push(cmd);
    }
    if use_uwsm {
        let mut cmd = vec![
            "uwsm-app".to_string(),
            "--".to_string(),
            "waybar".to_string(),
        ];
        cmd.extend(waybar_args.clone());
        candidates.push(cmd);
    }
    if use_setsid {
        let mut cmd = vec!["setsid".to_string(), "waybar".to_string()];
        cmd.extend(waybar_args.clone());
        candidates.push(cmd);
    }
    let mut cmd = vec!["waybar".to_string()];
    cmd.extend(waybar_args);
    candidates.push(cmd);

    for parts in candidates {
        let mut iter = parts.iter();
        let Some(cmd) = iter.next() else { continue };
        if !quiet {
            println!("theme-manager: starting waybar via {}", cmd);
        }
        let mut command = Command::new(cmd);
        command.args(iter);
        if quiet {
            command.stdout(Stdio::null()).stderr(Stdio::null());
        }
        match command.spawn() {
            Ok(mut child) => {
                thread::sleep(Duration::from_millis(100));
                match child.try_wait() {
                    Ok(Some(status)) => {
                        if status.success() {
                            return Ok(());
                        }
                        if !quiet {
                            eprintln!("theme-manager: waybar restart exited: {status}");
                        }
                    }
                    Ok(None) => return Ok(()),
                    Err(err) => {
                        if !quiet {
                            eprintln!("theme-manager: waybar restart check failed: {err}");
                        }
                    }
                }
            }
            Err(err) => {
                if !quiet {
                    eprintln!("theme-manager: waybar restart spawn failed: {err}");
                }
            }
        }
    }

    Err(anyhow!("failed to restart waybar"))
}

pub fn apply_theme_setters(quiet: bool) -> Result<()> {
    run_omarchy_optional("theme", "set-gnome", &[], quiet)?;
    run_omarchy_optional("theme", "set-browser", &[], quiet)?;
    run_omarchy_optional("theme", "set-vscode", &[], quiet)?;
    run_omarchy_optional("theme", "set-obsidian", &[], quiet)?;
    run_omarchy_optional("theme", "set-keyboard", &[], quiet)?;
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
                notify_awww_unavailable(quiet);
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

fn notify_awww_unavailable(quiet: bool) {
    if !command_exists("notify-send") {
        return;
    }
    let mut command = Command::new("notify-send");
    command.args([
        "--app-name=theme-manager",
        "--urgency=normal",
        "awww-daemon not available",
        "Transitions are disabled until it is running.",
    ]);
    if quiet {
        command.stdout(Stdio::null()).stderr(Stdio::null());
    }
    let _ = command.status();
}
