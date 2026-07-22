use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::process::{Command, Stdio};

use crate::omarchy;

#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub first_party: bool,
    pub kinds: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PluginJson {
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    enabled: bool,
    #[serde(default, rename = "firstParty")]
    first_party: bool,
    #[serde(default)]
    kinds: Vec<String>,
}

fn require_shell() -> Result<()> {
    if omarchy::shell_available() {
        return Ok(());
    }
    Err(anyhow!(
        "plugin management requires Omarchy 4.0+ (the Quickshell-based omarchy-shell); \
         omarchy-shell/omarchy-plugin not found in PATH"
    ))
}

/// Every discovered shell plugin (bar options, bar widgets, services,
/// overlays, panels, ...) with live enabled state. Requires omarchy-shell to
/// actually be running (unlike `bar::list_bar_options`, which only walks
/// manifests on disk).
pub fn list_plugins() -> Result<Vec<PluginInfo>> {
    require_shell()?;
    let output = Command::new("omarchy-plugin")
        .args(["list", "--json"])
        .output()
        .map_err(|err| anyhow!("failed to run omarchy-plugin: {err}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("omarchy-plugin list failed: {}", stderr.trim()));
    }
    let raw = String::from_utf8_lossy(&output.stdout);
    let entries: Vec<PluginJson> = serde_json::from_str(&raw)
        .map_err(|err| anyhow!("failed to parse omarchy-plugin output: {err}"))?;

    let mut plugins: Vec<PluginInfo> = entries
        .into_iter()
        .map(|entry| PluginInfo {
            id: entry.id,
            name: entry.name,
            enabled: entry.enabled,
            first_party: entry.first_party,
            kinds: entry.kinds,
        })
        .collect();
    plugins.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(plugins)
}

pub fn cmd_plugin_list() -> Result<()> {
    for plugin in list_plugins()? {
        let state = if plugin.enabled { "enabled" } else { "disabled" };
        let source = if plugin.first_party {
            "first-party"
        } else {
            "third-party"
        };
        let name = if plugin.name.is_empty() {
            &plugin.id
        } else {
            &plugin.name
        };
        println!(
            "{} [{state}] {source} ({}) - {name}",
            plugin.id,
            plugin.kinds.join(",")
        );
    }
    Ok(())
}

pub fn cmd_plugin_enable(id: &str, quiet: bool) -> Result<()> {
    require_shell()?;
    omarchy::run_omarchy_required("plugin", "enable", &[id], quiet)
}

pub fn cmd_plugin_disable(id: &str, quiet: bool) -> Result<()> {
    require_shell()?;
    omarchy::run_omarchy_required("plugin", "disable", &[id], quiet)
}

/// Passthrough for everything else (add/update/remove/rescan/clone/edit/
/// validate): these need real interactive stdio (git clone progress, `gum
/// confirm` prompts, diff review), so exec `omarchy-plugin` directly with
/// inherited stdio instead of capturing output like the other commands here.
pub fn cmd_plugin_passthrough(args: &[String]) -> Result<()> {
    require_shell()?;
    let status = Command::new("omarchy-plugin")
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|err| anyhow!("failed to run omarchy-plugin: {err}"))?;
    if !status.success() {
        return Err(anyhow!("omarchy-plugin exited with {status}"));
    }
    Ok(())
}
