use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::ResolvedConfig;
use crate::paths::{is_symlink, normalize_theme_name};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PresetFile {
  #[serde(default)]
  pub preset: BTreeMap<String, PresetEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PresetEntry {
  pub theme: Option<String>,
  pub waybar: Option<PresetWaybarEntry>,
  pub walker: Option<PresetWalkerEntry>,
  pub starship: Option<PresetStarshipEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PresetWaybarEntry {
  pub mode: Option<String>,
  pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PresetWalkerEntry {
  pub mode: Option<String>,
  pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PresetStarshipEntry {
  pub mode: Option<String>,
  pub preset: Option<String>,
  pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub enum PresetWaybarValue {
  None,
  Auto,
  Named(String),
}

#[derive(Debug, Clone)]
pub enum PresetWalkerValue {
  None,
  Auto,
  Named(String),
}

#[derive(Debug, Clone)]
pub enum PresetStarshipValue {
  None,
  Preset(String),
  Named(String),
  Theme,
}

#[derive(Debug, Clone)]
pub struct PresetDefinition {
  pub name: String,
  pub theme: String,
  pub waybar: PresetWaybarValue,
  pub walker: PresetWalkerValue,
  pub starship: PresetStarshipValue,
}

#[derive(Debug, Clone)]
pub struct PresetSummary {
  pub theme: String,
  pub waybar: String,
  pub walker: String,
  pub starship: String,
  pub errors: Vec<String>,
}

pub fn presets_path() -> Result<PathBuf> {
  let home = env::var("HOME").map_err(|_| anyhow!("HOME is not set"))?;
  Ok(PathBuf::from(home).join(".config/theme-manager/presets.toml"))
}

pub fn load_presets() -> Result<PresetFile> {
  let path = presets_path()?;
  load_presets_from_path(&path)
}

pub fn load_presets_from_path(path: &Path) -> Result<PresetFile> {
  if !path.is_file() {
    return Ok(PresetFile::default());
  }
  let content = fs::read_to_string(path)?;
  let parsed: PresetFile = toml::from_str(&content)?;
  Ok(parsed)
}

pub fn write_presets(file: &PresetFile) -> Result<()> {
  let path = presets_path()?;
  write_presets_to_path(&path, file)
}

pub fn write_presets_to_path(path: &Path, file: &PresetFile) -> Result<()> {
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)?;
  }
  let output = toml::to_string_pretty(file)?;
  fs::write(path, output)?;
  Ok(())
}

pub fn list_preset_names() -> Result<Vec<String>> {
  let mut names: Vec<String> = load_presets()?.preset.keys().cloned().collect();
  names.sort();
  Ok(names)
}

pub fn get_preset_entry(name: &str) -> Result<PresetEntry> {
  let key = name.trim();
  if key.is_empty() {
    return Err(anyhow!("missing preset name"));
  }
  let file = load_presets()?;
  file
    .preset
    .get(key)
    .cloned()
    .ok_or_else(|| anyhow!("preset not found: {key}"))
}

pub fn summarize_preset(
  config: &ResolvedConfig,
  name: &str,
  entry: &PresetEntry,
) -> PresetSummary {
  let mut errors = Vec::new();
  let theme = entry
    .theme
    .as_ref()
    .map(|val| val.trim().to_string())
    .filter(|val| !val.is_empty());
  let theme_label = theme.clone().unwrap_or_else(|| "Missing theme".to_string());
  if theme.is_none() {
    errors.push(format!("preset \"{name}\" missing theme"));
  }

  let waybar_value = parse_waybar(entry.waybar.as_ref(), &mut errors);
  let walker_value = parse_walker(entry.walker.as_ref(), &mut errors);
  let starship_value = parse_starship(entry.starship.as_ref(), &mut errors);

  if let Some(theme_name) = theme.as_ref() {
    let normalized = normalize_theme_name(theme_name);
    let theme_path = config.theme_root_dir.join(&normalized);
    if is_broken_theme(&theme_path) {
      errors.push(format!("theme not found: {normalized}"));
    }
    if matches!(starship_value, PresetStarshipValue::Theme) {
      let starship_path = theme_path.join("starship.toml");
      if !starship_path.is_file() {
        errors.push("theme starship.toml not found".to_string());
      }
    }
  }

  PresetSummary {
    theme: theme_label,
    waybar: format_waybar(&waybar_value),
    walker: format_walker(&walker_value),
    starship: format_starship(&starship_value),
    errors,
  }
}

pub fn load_preset_definition(config: &ResolvedConfig, name: &str) -> Result<PresetDefinition> {
  let entry = get_preset_entry(name)?;
  let summary = summarize_preset(config, name, &entry);
  if !summary.errors.is_empty() {
    return Err(anyhow!(summary.errors.join("; ")));
  }

  let theme = entry
    .theme
    .as_ref()
    .map(|val| val.trim().to_string())
    .filter(|val| !val.is_empty())
    .ok_or_else(|| anyhow!("preset \"{name}\" missing theme"))?;

  Ok(PresetDefinition {
    name: name.trim().to_string(),
    theme: theme.clone(),
    waybar: parse_waybar(entry.waybar.as_ref(), &mut Vec::new()),
    walker: parse_walker(entry.walker.as_ref(), &mut Vec::new()),
    starship: parse_starship(entry.starship.as_ref(), &mut Vec::new()),
  })
}

pub fn save_preset(name: &str, entry: PresetEntry, config: &ResolvedConfig) -> Result<()> {
  let trimmed = name.trim();
  if trimmed.is_empty() {
    return Err(anyhow!("missing preset name"));
  }

  let summary = summarize_preset(config, trimmed, &entry);
  if !summary.errors.is_empty() {
    return Err(anyhow!(summary.errors.join("; ")));
  }

  let mut file = load_presets()?;
  file.preset.insert(trimmed.to_string(), entry);
  write_presets(&file)?;
  Ok(())
}

pub fn remove_preset(name: &str) -> Result<()> {
  let trimmed = name.trim();
  if trimmed.is_empty() {
    return Err(anyhow!("missing preset name"));
  }
  let mut file = load_presets()?;
  if file.preset.remove(trimmed).is_none() {
    return Err(anyhow!("preset not found: {trimmed}"));
  }
  write_presets(&file)?;
  Ok(())
}

fn parse_waybar(entry: Option<&PresetWaybarEntry>, errors: &mut Vec<String>) -> PresetWaybarValue {
  let mode = entry
    .and_then(|val| val.mode.as_deref())
    .unwrap_or("none")
    .trim();
  match mode {
    "" | "none" => PresetWaybarValue::None,
    "auto" => PresetWaybarValue::Auto,
    "named" => match entry.and_then(|val| val.name.clone()) {
      Some(name) if !name.trim().is_empty() => PresetWaybarValue::Named(name),
      _ => {
        errors.push("waybar.mode = named requires waybar.name".to_string());
        PresetWaybarValue::None
      }
    },
    _ => {
      errors.push(format!("invalid waybar.mode: {mode}"));
      PresetWaybarValue::None
    }
  }
}

fn parse_starship(
  entry: Option<&PresetStarshipEntry>,
  errors: &mut Vec<String>,
) -> PresetStarshipValue {
  let mode = entry
    .and_then(|val| val.mode.as_deref())
    .unwrap_or("none")
    .trim();
  match mode {
    "" | "none" => PresetStarshipValue::None,
    "preset" => match entry.and_then(|val| val.preset.clone()) {
      Some(name) if !name.trim().is_empty() => PresetStarshipValue::Preset(name),
      _ => {
        errors.push("starship.mode = preset requires starship.preset".to_string());
        PresetStarshipValue::None
      }
    },
    "named" => match entry.and_then(|val| val.name.clone()) {
      Some(name) if !name.trim().is_empty() => PresetStarshipValue::Named(name),
      _ => {
        errors.push("starship.mode = named requires starship.name".to_string());
        PresetStarshipValue::None
      }
    },
    "theme" => PresetStarshipValue::Theme,
    _ => {
      errors.push(format!("invalid starship.mode: {mode}"));
      PresetStarshipValue::None
    }
  }
}

fn is_broken_theme(path: &Path) -> bool {
  if path.is_dir() {
    return false;
  }
  if let Ok(true) = is_symlink(path) {
    return fs::metadata(path).is_err();
  }
  true
}

fn format_waybar(value: &PresetWaybarValue) -> String {
  match value {
    PresetWaybarValue::None => "none".to_string(),
    PresetWaybarValue::Auto => "auto".to_string(),
    PresetWaybarValue::Named(name) => format!("named ({name})"),
  }
}

fn format_walker(value: &PresetWalkerValue) -> String {
  match value {
    PresetWalkerValue::None => "none".to_string(),
    PresetWalkerValue::Auto => "auto".to_string(),
    PresetWalkerValue::Named(name) => format!("named ({name})"),
  }
}

fn parse_walker(entry: Option<&PresetWalkerEntry>, errors: &mut Vec<String>) -> PresetWalkerValue {
  let mode = entry
    .and_then(|val| val.mode.as_deref())
    .unwrap_or("none")
    .trim();
  match mode {
    "" | "none" => PresetWalkerValue::None,
    "auto" => PresetWalkerValue::Auto,
    "named" => match entry.and_then(|val| val.name.clone()) {
      Some(name) if !name.trim().is_empty() => PresetWalkerValue::Named(name),
      _ => {
        errors.push("walker.mode = named requires walker.name".to_string());
        PresetWalkerValue::None
      }
    },
    _ => {
      errors.push(format!("invalid walker.mode: {mode}"));
      PresetWalkerValue::None
    }
  }
}

fn format_starship(value: &PresetStarshipValue) -> String {
  match value {
    PresetStarshipValue::None => "none".to_string(),
    PresetStarshipValue::Preset(name) => format!("preset ({name})"),
    PresetStarshipValue::Named(name) => format!("named ({name})"),
    PresetStarshipValue::Theme => "theme".to_string(),
  }
}
