use anyhow::{anyhow, Result};
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use crate::config::ResolvedConfig;
use crate::omarchy;
use crate::paths::{current_theme_dir, normalize_theme_name, resolve_link_target, title_case_theme};
use crate::starship;
use crate::waybar;

#[derive(Debug, Clone)]
pub enum WaybarMode {
  None,
  Auto,
  Named,
}

#[derive(Debug, Clone)]
pub enum StarshipMode {
  None,
  Preset { preset: String },
  Named { name: String },
  Theme { path: Option<PathBuf> },
}

pub struct CommandContext<'a> {
  pub config: &'a ResolvedConfig,
  pub quiet: bool,
  pub skip_apps: bool,
  pub skip_hook: bool,
  pub waybar_mode: WaybarMode,
  pub waybar_name: Option<String>,
  pub starship_mode: StarshipMode,
  pub debug_awww: bool,
}

pub fn waybar_from_defaults(config: &ResolvedConfig) -> (WaybarMode, Option<String>) {
  match config.default_waybar_mode.as_deref() {
    Some("auto") => (WaybarMode::Auto, None),
    Some("named") => (WaybarMode::Named, config.default_waybar_name.clone()),
    _ => (WaybarMode::None, None),
  }
}

pub fn starship_from_defaults(config: &ResolvedConfig) -> StarshipMode {
  match config.default_starship_mode.as_deref() {
    Some("preset") => {
      if let Some(preset) = &config.default_starship_preset {
        StarshipMode::Preset {
          preset: preset.clone(),
        }
      } else {
        StarshipMode::None
      }
    }
    Some("named") => {
      if let Some(name) = &config.default_starship_name {
        StarshipMode::Named { name: name.clone() }
      } else {
        StarshipMode::None
      }
    }
    _ => StarshipMode::None,
  }
}

pub fn cmd_list(config: &ResolvedConfig) -> Result<()> {
  let entries = sorted_theme_entries(&config.theme_root_dir)?;
  for name in entries {
    println!("{}", title_case_theme(&name));
  }
  Ok(())
}

pub fn cmd_set(ctx: &CommandContext<'_>, theme_name: &str) -> Result<()> {
  let normalized = normalize_theme_name(theme_name);
  let theme_path = ctx.config.theme_root_dir.join(&normalized);

  if is_broken_symlink(&theme_path)? {
    return Err(anyhow!(
      "theme symlink is broken: {}",
      theme_path.to_string_lossy()
    ));
  }
  if !theme_path.is_dir() && !is_symlink(&theme_path)? {
    if normalized != theme_name {
      return Err(anyhow!(
        "theme not found: {normalized} (from '{theme_name}')"
      ));
    }
    return Err(anyhow!("theme not found: {normalized}"));
  }

  omarchy::ensure_awww_daemon(ctx.config, ctx.quiet);

  ensure_parent_dir(&ctx.config.current_theme_link)?;
  replace_symlink(&theme_path, &ctx.config.current_theme_link)?;

  let mut waybar_restart = None;
  if !ctx.skip_apps {
    waybar_restart = waybar::prepare_waybar(ctx, &theme_path)?;
    starship::apply_starship(ctx, &theme_path)?;
  }

  if !ctx.skip_apps {
    if ctx.config.awww_transition && omarchy::command_exists("awww") {
      cycle_background(ctx, &theme_path)?;
      let _ = omarchy::run_awww_transition(ctx.config, ctx.quiet, ctx.debug_awww);
    } else {
      omarchy::run_required("omarchy-theme-bg-next", &[], ctx.quiet)?;
    }
    omarchy::reload_components(ctx.quiet, waybar_restart)?;
    omarchy::apply_theme_setters(ctx.quiet)?;
  }

  if !ctx.skip_hook {
    let hook_path = PathBuf::from(format!(
      "{}/.config/omarchy/hooks/theme-set",
      std::env::var("HOME").unwrap_or_default()
    ));
    let _ = omarchy::run_hook(&hook_path, &[&normalized], ctx.quiet);
  }

  Ok(())
}

pub fn cmd_next(ctx: &CommandContext<'_>) -> Result<()> {
  let entries = sorted_theme_entries(&ctx.config.theme_root_dir)?;
  if entries.is_empty() {
    return Err(anyhow!("no themes available"));
  }

  let current_dir = current_theme_dir(&ctx.config.current_theme_link).ok();
  let current_name = current_dir
    .as_ref()
    .and_then(|path| path.file_name())
    .and_then(|name| name.to_str())
    .map(|name| name.to_string());

  let next = next_theme(&entries, current_name.as_deref());
  cmd_set(ctx, &next)
}

pub fn cmd_current(config: &ResolvedConfig) -> Result<()> {
  if !is_symlink(&config.current_theme_link)? {
    return Err(anyhow!(
      "current theme not set: {}",
      config.current_theme_link.to_string_lossy()
    ));
  }
  let target = resolve_link_target(&config.current_theme_link)?;
  let name = target
    .file_name()
    .and_then(|n| n.to_str())
    .ok_or_else(|| anyhow!("current theme not set: invalid link target"))?;
  println!("{}", title_case_theme(name));
  Ok(())
}

pub fn cmd_bg_next(_config: &ResolvedConfig) -> Result<()> {
  omarchy::run_required("omarchy-theme-bg-next", &[], false)?;
  Ok(())
}

pub fn cmd_version() {
  println!("{}", env!("CARGO_PKG_VERSION"));
}

pub fn cmd_browse_stub(_ctx: &CommandContext<'_>) -> Result<()> {
  Err(anyhow!(
    "browse is not implemented in the Rust binary yet (use the Bash CLI for now)"
  ))
}

fn sorted_theme_entries(theme_root: &Path) -> Result<Vec<String>> {
  let mut entries = list_theme_entries(theme_root)?;
  entries.sort();
  Ok(entries)
}

pub fn list_theme_entries(theme_root: &Path) -> Result<Vec<String>> {
  if !theme_root.is_dir() {
    return Err(anyhow!(
      "themes directory not found: {}",
      theme_root.to_string_lossy()
    ));
  }
  let mut entries = Vec::new();
  for entry in fs::read_dir(theme_root)? {
    let entry = entry?;
    let path = entry.path();
    if path.is_dir() || is_symlink(&path)? {
      if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        entries.push(name.to_string());
      }
    }
  }
  Ok(entries)
}

fn next_theme(entries: &[String], current: Option<&str>) -> String {
  if let Some(current) = current {
    if let Some(idx) = entries.iter().position(|name| name == current) {
      let next_idx = (idx + 1) % entries.len();
      return entries[next_idx].clone();
    }
  }
  entries[0].clone()
}

fn replace_symlink(target: &Path, link_path: &Path) -> Result<()> {
  if let Ok(meta) = fs::symlink_metadata(link_path) {
    if meta.file_type().is_dir() {
      fs::remove_dir_all(link_path)?;
    } else {
      fs::remove_file(link_path)?;
    }
  }
  symlink(target, link_path)?;
  Ok(())
}

fn ensure_parent_dir(path: &Path) -> Result<()> {
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)?;
  }
  Ok(())
}

fn is_symlink(path: &Path) -> Result<bool> {
  match fs::symlink_metadata(path) {
    Ok(meta) => Ok(meta.file_type().is_symlink()),
    Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
    Err(err) => Err(err.into()),
  }
}

fn cycle_background(ctx: &CommandContext<'_>, theme_path: &Path) -> Result<()> {
  let backgrounds_dir = theme_path.join("backgrounds");
  if !backgrounds_dir.is_dir() {
    return Ok(());
  }

  let mut images: Vec<PathBuf> = fs::read_dir(&backgrounds_dir)?
    .filter_map(|entry| entry.ok().map(|e| e.path()))
    .filter(|path| {
      path.is_file()
        && path
          .extension()
          .and_then(|ext| ext.to_str())
          .map(|ext| {
            matches!(
              ext.to_ascii_lowercase().as_str(),
              "png" | "jpg" | "jpeg" | "webp"
            )
          })
          .unwrap_or(false)
    })
    .collect();
  images.sort();
  if images.is_empty() {
    return Ok(());
  }

  let current_link = &ctx.config.current_background_link;
  let current_target = if current_link.exists() {
    if current_link.is_symlink() {
      Some(crate::paths::resolve_link_target(current_link)?)
    } else {
      Some(current_link.to_path_buf())
    }
  } else {
    None
  };

  let next_index = current_target
    .as_ref()
    .and_then(|target| images.iter().position(|img| img == target))
    .map(|idx| (idx + 1) % images.len())
    .unwrap_or(0);

  let next_image = &images[next_index];
  if let Some(parent) = current_link.parent() {
    fs::create_dir_all(parent)?;
  }
  if let Ok(meta) = fs::symlink_metadata(current_link) {
    if meta.file_type().is_dir() {
      fs::remove_dir_all(current_link)?;
    } else {
      fs::remove_file(current_link)?;
    }
  }
  #[cfg(unix)]
  {
    std::os::unix::fs::symlink(next_image, current_link)?;
  }
  Ok(())
}

fn is_broken_symlink(path: &Path) -> Result<bool> {
  if !is_symlink(path)? {
    return Ok(false);
  }
  Ok(fs::metadata(path).is_err())
}
