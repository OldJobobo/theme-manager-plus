use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::omarchy::{RestartAction, RestartCommand};
use crate::theme_ops::{CommandContext, WaybarMode};
use walkdir::WalkDir;

const WAYBAR_LINKS_FILE: &str = ".theme-manager-waybar-links";

pub fn prepare_waybar(ctx: &CommandContext<'_>, theme_dir: &Path) -> Result<Option<RestartAction>> {
  let waybar_dir = match ctx.waybar_mode {
    WaybarMode::None => return Ok(None),
    WaybarMode::Auto => theme_dir.join("waybar-theme"),
    WaybarMode::Named => match &ctx.waybar_name {
      Some(name) => ctx.config.waybar_themes_dir.join(name),
      None => return Ok(None),
    },
  };

  if !waybar_dir.is_dir() {
    if !ctx.quiet {
      eprintln!(
        "theme-manager: waybar theme directory not found: {}",
        waybar_dir.to_string_lossy()
      );
    }
    return Ok(None);
  }

  let config_path = waybar_dir.join("config.jsonc");
  let style_path = waybar_dir.join("style.css");
  if !config_path.is_file() || !style_path.is_file() {
    if !ctx.quiet {
      eprintln!(
        "theme-manager: waybar theme missing config.jsonc or style.css in {}",
        waybar_dir.to_string_lossy()
      );
    }
    return Ok(None);
  }

  cleanup_waybar_links(&ctx.config.waybar_dir, ctx.quiet)?;

  let apply_mode = ctx.config.waybar_apply_mode.as_str();
  if apply_mode == "copy" {
    return apply_copy(ctx, &config_path, &style_path);
  }

  apply_symlink(ctx, &config_path, &style_path)
}

fn apply_copy(
  ctx: &CommandContext<'_>,
  config_path: &Path,
  style_path: &Path,
) -> Result<Option<RestartAction>> {
  fs::create_dir_all(&ctx.config.waybar_dir)?;
  let theme_waybar_dir = config_path
    .parent()
    .ok_or_else(|| anyhow!("waybar config has no parent directory"))?;
  let mut backup_dir = None;

  if !ctx.quiet {
    println!(
      "theme-manager: copying waybar config from {}",
      config_path.to_string_lossy()
    );
    println!(
      "theme-manager: copying waybar style from {}",
      style_path.to_string_lossy()
    );
  }

  let dest_config = ctx.config.waybar_dir.join("config.jsonc");
  let dest_style = ctx.config.waybar_dir.join("style.css");
  replace_existing_path(
    &dest_config,
    "config.jsonc",
    &ctx.config.waybar_themes_dir,
    &mut backup_dir,
    ctx.quiet,
  )?;
  replace_existing_path(
    &dest_style,
    "style.css",
    &ctx.config.waybar_themes_dir,
    &mut backup_dir,
    ctx.quiet,
  )?;
  fs::copy(config_path, &dest_config)?;
  fs::copy(style_path, &dest_style)?;

  copy_waybar_subdirs(
    theme_waybar_dir,
    &ctx.config.waybar_dir,
    &ctx.config.waybar_themes_dir,
    &mut backup_dir,
    ctx.quiet,
  )?;

  Ok(Some(RestartAction::Command(RestartCommand {
    cmd: "omarchy-restart-waybar".to_string(),
    args: Vec::new(),
  })))
}

fn apply_symlink(
  ctx: &CommandContext<'_>,
  config_path: &Path,
  style_path: &Path,
) -> Result<Option<RestartAction>> {
  fs::create_dir_all(&ctx.config.waybar_dir)?;
  let theme_waybar_dir = config_path
    .parent()
    .ok_or_else(|| anyhow!("waybar config has no parent directory"))?;
  let mut backup_dir = None;

  if !ctx.quiet {
    println!(
      "theme-manager: linking waybar config from {}",
      config_path.to_string_lossy()
    );
    println!(
      "theme-manager: linking waybar style from {}",
      style_path.to_string_lossy()
    );
  }

  let dest_config = ctx.config.waybar_dir.join("config.jsonc");
  let dest_style = ctx.config.waybar_dir.join("style.css");
  replace_with_symlink(
    &dest_config,
    config_path,
    "config.jsonc",
    &ctx.config.waybar_themes_dir,
    &mut backup_dir,
    ctx.quiet,
  )?;
  replace_with_symlink(
    &dest_style,
    style_path,
    "style.css",
    &ctx.config.waybar_themes_dir,
    &mut backup_dir,
    ctx.quiet,
  )?;
  link_waybar_subdirs(
    theme_waybar_dir,
    &ctx.config.waybar_dir,
    &ctx.config.waybar_themes_dir,
    &mut backup_dir,
    ctx.quiet,
  )?;

  Ok(Some(RestartAction::Command(RestartCommand {
    cmd: "omarchy-restart-waybar".to_string(),
    args: Vec::new(),
  })))
}

fn cleanup_waybar_links(waybar_dir: &Path, quiet: bool) -> Result<()> {
  let manifest_path = waybar_dir.join(WAYBAR_LINKS_FILE);
  if !manifest_path.is_file() {
    return Ok(());
  }

  let content = fs::read_to_string(&manifest_path)?;
  for line in content.lines() {
    let name = line.trim();
    if name.is_empty() {
      continue;
    }
    let path = waybar_dir.join(name);
    let meta = match fs::symlink_metadata(&path) {
      Ok(meta) => meta,
      Err(_) => continue,
    };
    if !meta.file_type().is_symlink() {
      continue;
    }
    if !quiet {
      println!("theme-manager: removing waybar link {}", path.to_string_lossy());
    }
    let _ = fs::remove_file(&path);
  }

  let _ = fs::remove_file(&manifest_path);
  Ok(())
}

fn link_waybar_subdirs(
  theme_waybar_dir: &Path,
  waybar_dir: &Path,
  waybar_themes_dir: &Path,
  backup_dir: &mut Option<PathBuf>,
  quiet: bool,
) -> Result<()> {
  let mut linked = Vec::new();
  for entry in fs::read_dir(theme_waybar_dir)? {
    let entry = entry?;
    let name = entry.file_name();
    let name_str = name.to_string_lossy();
    if name_str == "config.jsonc" || name_str == "style.css" {
      continue;
    }
    let file_type = entry.file_type()?;
    let entry_path = entry.path();
    let is_dir = if file_type.is_dir() {
      true
    } else if file_type.is_symlink() {
      fs::metadata(&entry_path).map(|meta| meta.is_dir()).unwrap_or(false)
    } else {
      false
    };
    if !is_dir {
      continue;
    }

    let dest = waybar_dir.join(&name);
    replace_existing_path(&dest, &name_str, waybar_themes_dir, backup_dir, quiet)?;

    std::os::unix::fs::symlink(&entry_path, &dest)?;
    if !quiet {
      println!("theme-manager: linking waybar subdir {}", dest.to_string_lossy());
    }
    linked.push(name_str.to_string());
  }

  let manifest_path = waybar_dir.join(WAYBAR_LINKS_FILE);
  if linked.is_empty() {
    let _ = fs::remove_file(&manifest_path);
    return Ok(());
  }

  let mut manifest = String::new();
  for name in linked {
    manifest.push_str(&name);
    manifest.push('\n');
  }
  fs::write(manifest_path, manifest)?;
  Ok(())
}

fn copy_waybar_subdirs(
  theme_waybar_dir: &Path,
  waybar_dir: &Path,
  waybar_themes_dir: &Path,
  backup_dir: &mut Option<PathBuf>,
  quiet: bool,
) -> Result<()> {
  for entry in fs::read_dir(theme_waybar_dir)? {
    let entry = entry?;
    let name = entry.file_name();
    let name_str = name.to_string_lossy();
    if name_str == "config.jsonc" || name_str == "style.css" {
      continue;
    }
    let file_type = entry.file_type()?;
    let entry_path = entry.path();
    let is_dir = if file_type.is_dir() {
      true
    } else if file_type.is_symlink() {
      fs::metadata(&entry_path).map(|meta| meta.is_dir()).unwrap_or(false)
    } else {
      false
    };
    if !is_dir {
      continue;
    }

    let dest = waybar_dir.join(&name);
    replace_existing_path(&dest, &name_str, waybar_themes_dir, backup_dir, quiet)?;
    copy_dir_recursive(&entry_path, &dest)?;
    if !quiet {
      println!("theme-manager: copying waybar subdir {}", dest.to_string_lossy());
    }
  }
  Ok(())
}

fn copy_dir_recursive(source: &Path, dest: &Path) -> Result<()> {
  for entry in WalkDir::new(source).follow_links(false) {
    let entry = entry?;
    let entry_path = entry.path();
    let rel = entry_path.strip_prefix(source)?;
    if rel.as_os_str().is_empty() {
      continue;
    }
    let target_path = dest.join(rel);
    let file_type = entry.file_type();
    if file_type.is_dir() {
      fs::create_dir_all(&target_path)?;
    } else if file_type.is_symlink() {
      let link_target = fs::read_link(entry_path)?;
      if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)?;
      }
      #[cfg(unix)]
      std::os::unix::fs::symlink(link_target, &target_path)?;
    } else {
      if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)?;
      }
      fs::copy(entry_path, &target_path)?;
    }
  }
  Ok(())
}

fn replace_with_symlink(
  dest: &Path,
  source: &Path,
  name: &str,
  waybar_themes_dir: &Path,
  backup_dir: &mut Option<PathBuf>,
  quiet: bool,
) -> Result<()> {
  replace_existing_path(dest, name, waybar_themes_dir, backup_dir, quiet)?;
  std::os::unix::fs::symlink(source, dest)?;
  Ok(())
}

fn replace_existing_path(
  dest: &Path,
  name: &str,
  waybar_themes_dir: &Path,
  backup_dir: &mut Option<PathBuf>,
  quiet: bool,
) -> Result<()> {
  let meta = match fs::symlink_metadata(dest) {
    Ok(meta) => meta,
    Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
    Err(err) => return Err(err.into()),
  };

  if meta.file_type().is_symlink() {
    fs::remove_file(dest)?;
    return Ok(());
  }

  let backup_root = ensure_backup_dir(waybar_themes_dir, backup_dir)?;
  let backup_target = unique_backup_target(&backup_root, name)?;
  if !quiet {
    println!(
      "theme-manager: backing up existing waybar path {} -> {}",
      dest.to_string_lossy(),
      backup_target.to_string_lossy()
    );
  }
  fs::rename(dest, backup_target)?;
  Ok(())
}

fn ensure_backup_dir(
  waybar_themes_dir: &Path,
  backup_dir: &mut Option<PathBuf>,
) -> Result<PathBuf> {
  if let Some(existing) = backup_dir {
    return Ok(existing.clone());
  }

  let base = waybar_themes_dir.join("existing");
  let chosen = if base.exists() {
    let stamp = timestamp_suffix()?;
    waybar_themes_dir.join(format!("existing-{stamp}"))
  } else {
    base
  };
  fs::create_dir_all(&chosen)?;
  *backup_dir = Some(chosen.clone());
  Ok(chosen)
}

fn unique_backup_target(dir: &Path, name: &str) -> Result<PathBuf> {
  let candidate = dir.join(name);
  if !candidate.exists() {
    return Ok(candidate);
  }
  let stamp = timestamp_suffix()?;
  Ok(dir.join(format!("{name}-{stamp}")))
}

fn timestamp_suffix() -> Result<u64> {
  Ok(SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_err(|err| anyhow!("time error: {err}"))?
    .as_secs())
}
