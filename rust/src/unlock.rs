use anyhow::{anyhow, Result};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::ResolvedConfig;
use crate::omarchy;
use crate::paths::{normalize_theme_name, title_case_theme};

pub fn cmd_unlock_list(config: &ResolvedConfig) -> Result<()> {
    for name in list_unlock_theme_entries(config)? {
        println!("{}", title_case_theme(&name));
    }
    println!("Default");
    Ok(())
}

pub fn cmd_unlock_set(config: &ResolvedConfig, theme_name: &str, quiet: bool) -> Result<()> {
    let normalized = normalize_theme_name(theme_name);
    if normalized == "default" {
        return cmd_unlock_reset(quiet);
    }

    let theme_dir = resolve_unlock_theme_path(config, &normalized)?;

    let unlock = theme_dir.join("unlock.png");
    let colors = theme_dir.join("colors.toml");
    if !unlock.is_file() {
        return Err(anyhow!(
            "unlock theme missing unlock.png: {}",
            theme_dir.to_string_lossy()
        ));
    }
    if !colors.is_file() {
        return Err(anyhow!(
            "unlock theme missing colors.toml: {}",
            theme_dir.to_string_lossy()
        ));
    }

    omarchy::run_omarchy_required("plymouth", "set-by-theme", &[&normalized], quiet)
}

pub fn cmd_unlock_reset(quiet: bool) -> Result<()> {
    omarchy::run_omarchy_required("plymouth", "reset", &[], quiet)
}

pub fn list_unlock_theme_entries(config: &ResolvedConfig) -> Result<Vec<String>> {
    let mut entries = Vec::new();
    let mut seen = HashSet::new();

    for root in theme_roots(config) {
        if !root.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&root)? {
            let entry = entry?;
            let path = entry.path();
            if !is_theme_dir(&path) || !path.join("preview-unlock.png").is_file() {
                continue;
            }
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if seen.insert(name.to_string()) {
                    entries.push(name.to_string());
                }
            }
        }
    }

    entries.sort();
    Ok(entries)
}

fn resolve_unlock_theme_path(config: &ResolvedConfig, normalized: &str) -> Result<PathBuf> {
    for root in theme_roots(config) {
        let candidate = root.join(normalized);
        if is_theme_dir(&candidate) && candidate.join("preview-unlock.png").is_file() {
            return Ok(candidate);
        }
    }
    Err(anyhow!("unlock theme not found: {normalized}"))
}

fn theme_roots(config: &ResolvedConfig) -> Vec<PathBuf> {
    let mut roots = vec![config.theme_root_dir.clone()];
    if let Some(root) = omarchy::detect_omarchy_root(config) {
        let omarchy_themes = root.join("themes");
        if omarchy_themes != config.theme_root_dir {
            roots.push(omarchy_themes);
        }
    }
    roots
}

fn is_theme_dir(path: &Path) -> bool {
    path.is_dir() || path.is_symlink()
}
