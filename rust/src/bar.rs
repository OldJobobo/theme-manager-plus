use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::omarchy;

#[derive(Debug, Clone)]
pub struct BarOption {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
struct CatalogEntry {
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    kinds: Vec<String>,
    #[serde(rename = "barPath")]
    bar_path: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ShellConfig {
    bar: Option<BarSection>,
}

#[derive(Debug, Deserialize, Default)]
struct BarSection {
    id: Option<String>,
}

fn require_shell() -> Result<()> {
    if omarchy::shell_available() {
        return Ok(());
    }
    Err(anyhow!(
        "bar management requires Omarchy 4.0+ (the Quickshell-based omarchy-shell); \
         omarchy-shell/omarchy-bar not found in PATH"
    ))
}

fn shell_config_path() -> Result<PathBuf> {
    let home = env::var("HOME").map_err(|_| anyhow!("HOME is not set"))?;
    Ok(PathBuf::from(home).join(".config/omarchy/shell.json"))
}

/// The active bar's plugin id, or `None` when the built-in `omarchy.bar` is active.
pub fn current_bar_id() -> Result<Option<String>> {
    let path = shell_config_path()?;
    if !path.is_file() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path)?;
    if raw.trim().is_empty() {
        return Ok(None);
    }
    let parsed: ShellConfig = serde_json::from_str(&raw)
        .map_err(|err| anyhow!("failed to parse {}: {err}", path.to_string_lossy()))?;
    Ok(parsed.bar.and_then(|bar| bar.id))
}

/// Every installed bar-kind plugin (full alternate bar layouts), built-in and
/// user-installed, sourced from `omarchy-plugin-catalog` (manifest walking,
/// works even when the shell isn't running).
pub fn list_bar_options() -> Result<Vec<BarOption>> {
    require_shell()?;
    let output = Command::new("omarchy-plugin-catalog")
        .output()
        .map_err(|err| anyhow!("failed to run omarchy-plugin-catalog: {err}"))?;
    if !output.status.success() {
        return Err(anyhow!(
            "omarchy-plugin-catalog exited with {}",
            output.status
        ));
    }
    let raw = String::from_utf8_lossy(&output.stdout);
    let entries: Vec<CatalogEntry> = serde_json::from_str(&raw)
        .map_err(|err| anyhow!("failed to parse omarchy-plugin-catalog output: {err}"))?;

    let mut options: Vec<BarOption> = entries
        .into_iter()
        .filter(|entry| entry.kinds.iter().any(|kind| kind == "bar") && entry.bar_path.is_some())
        .map(|entry| BarOption {
            id: entry.id.clone(),
            name: if entry.name.is_empty() {
                entry.id
            } else {
                entry.name
            },
            description: entry.description,
        })
        .collect();

    options.sort_by(|a, b| {
        // Keep the built-in bar pinned first, like Waybar/Walker tabs pin
        // `Omarchy-Default`.
        let a_builtin = a.id == "omarchy.bar";
        let b_builtin = b.id == "omarchy.bar";
        b_builtin.cmp(&a_builtin).then_with(|| a.name.cmp(&b.name))
    });

    Ok(options)
}

pub fn cmd_bar_list() -> Result<()> {
    let current = current_bar_id()?;
    for option in list_bar_options()? {
        let is_current = match &current {
            Some(id) => id == &option.id,
            None => option.id == "omarchy.bar",
        };
        let marker = if is_current { "*" } else { " " };
        println!("{marker} {} ({})", option.name, option.id);
    }
    Ok(())
}

pub fn cmd_bar_use(id: &str, quiet: bool) -> Result<()> {
    require_shell()?;
    omarchy::run_omarchy_required("bar", "use", &[id], quiet)
}

pub fn cmd_bar_reset(quiet: bool) -> Result<()> {
    require_shell()?;
    omarchy::run_omarchy_required("bar", "reset", &[], quiet)
}

pub fn cmd_bar_defaults(quiet: bool) -> Result<()> {
    require_shell()?;
    omarchy::run_omarchy_required("bar", "defaults", &[], quiet)
}

pub fn cmd_bar_position(position: &str, quiet: bool) -> Result<()> {
    require_shell()?;
    if !matches!(position, "top" | "bottom" | "left" | "right") {
        return Err(anyhow!(
            "position must be one of top, bottom, left, right (got '{position}')"
        ));
    }
    omarchy::run_omarchy_required("bar", "position", &[position], quiet)
}

pub fn cmd_bar_transparent(value: &str, quiet: bool) -> Result<()> {
    require_shell()?;
    if !matches!(value, "true" | "false") {
        return Err(anyhow!(
            "transparent must be 'true' or 'false' (got '{value}')"
        ));
    }
    omarchy::run_omarchy_required("bar", "transparent", &[value], quiet)
}

/// Passthrough for widget-layout editing (add/move/remove/set/replace):
/// these accept a mix of positional args and `--section`/`--index`/
/// `--before`/`--after`/`--json` flags that aren't worth re-modeling in
/// clap, so exec `omarchy-bar-plugin` directly with inherited stdio.
pub fn cmd_bar_plugin_passthrough(args: &[String]) -> Result<()> {
    require_shell()?;
    let status = Command::new("omarchy-bar-plugin")
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|err| anyhow!("failed to run omarchy-bar-plugin: {err}"))?;
    if !status.success() {
        return Err(anyhow!("omarchy-bar-plugin exited with {status}"));
    }
    Ok(())
}
