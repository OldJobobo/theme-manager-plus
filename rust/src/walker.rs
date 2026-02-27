use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::config::ResolvedConfig;
use crate::omarchy_defaults;
use crate::omarchy_defaults::SymlinkEnsureResult;
use crate::theme_ops::{CommandContext, WalkerMode};

const AUTO_THEME_NAME: &str = "theme-manager-auto";
const OMARCHY_DEFAULT_THEME_NAME: &str = "omarchy-default";

pub fn prepare_walker(ctx: &CommandContext<'_>, theme_dir: &Path) -> Result<()> {
  ensure_omarchy_default_theme_link(ctx.config, ctx.quiet)?;

  let (walker_theme_dir, theme_name) = match ctx.walker_mode {
    WalkerMode::None => return Ok(()),
    WalkerMode::Auto => {
      let dir = theme_dir.join("walker-theme");
      (dir, None)
    }
    WalkerMode::Named => match &ctx.walker_name {
      Some(name) => {
        let dir = ctx.config.walker_themes_dir.join(name);
        (dir, Some(name.clone()))
      }
      None => return Ok(()),
    },
  };

  if !walker_theme_dir.is_dir() {
    if !ctx.quiet {
      eprintln!(
        "theme-manager: walker theme directory not found: {}",
        walker_theme_dir.to_string_lossy()
      );
    }
    return Ok(());
  }

  // Walker themes require style.css, layout.xml is optional
  let style_path = walker_theme_dir.join("style.css");
  if !style_path.is_file() {
    if !ctx.quiet {
      eprintln!(
        "theme-manager: walker theme missing style.css in {}",
        walker_theme_dir.to_string_lossy()
      );
    }
    return Ok(());
  }

  // For named themes that exist in walker_themes_dir, just update the config
  if let Some(name) = theme_name {
    return update_walker_config(ctx, &name);
  }

  // For auto mode (theme-bundled), we need to copy/link the theme files
  cleanup_auto_theme_dir(&ctx.config.walker_themes_dir, ctx.quiet)?;

  let layout_path = walker_theme_dir.join("layout.xml");
  let apply_mode = ctx.config.walker_apply_mode.as_str();
  if apply_mode == "copy" {
    return apply_copy(ctx, &walker_theme_dir, &style_path, &layout_path);
  }

  apply_symlink(ctx, &walker_theme_dir, &style_path, &layout_path)
}

pub fn ensure_omarchy_default_theme_link(config: &ResolvedConfig, quiet: bool) -> Result<()> {
  let Some(default_theme_dir) = omarchy_defaults::resolve_walker_default(config).map(|d| d.path) else {
    return Ok(());
  };

  let link_path = config.walker_themes_dir.join(OMARCHY_DEFAULT_THEME_NAME);
  match omarchy_defaults::ensure_symlink(&link_path, &default_theme_dir)? {
    SymlinkEnsureResult::Created => {
      if !quiet {
        println!(
          "theme-manager: linked Omarchy default Walker theme {} -> {}",
          link_path.to_string_lossy(),
          default_theme_dir.to_string_lossy()
        );
      }
    }
    SymlinkEnsureResult::Updated => {
      if !quiet {
        println!(
          "theme-manager: repaired Omarchy default Walker theme link {} -> {}",
          link_path.to_string_lossy(),
          default_theme_dir.to_string_lossy()
        );
      }
    }
    SymlinkEnsureResult::SkippedNonSymlink => {
      if !quiet {
        eprintln!(
          "theme-manager: warning: preserving non-symlink path {}; cannot link Omarchy default Walker theme",
          link_path.to_string_lossy()
        );
      }
    }
    SymlinkEnsureResult::Unchanged => {}
  }

  Ok(())
}

fn update_walker_config(ctx: &CommandContext<'_>, theme_name: &str) -> Result<()> {
  let config_path = ctx.config.walker_dir.join("config.toml");

  if !config_path.is_file() {
    if !ctx.quiet {
      eprintln!("theme-manager: walker config not found at {}", config_path.to_string_lossy());
    }
    return Ok(());
  }

  let content = fs::read_to_string(&config_path)?;
  let mut new_lines = Vec::new();
  let mut found_theme = false;

  for line in content.lines() {
    let is_theme_assignment = line
      .split_once('=')
      .map(|(lhs, _)| lhs.trim() == "theme")
      .unwrap_or(false);
    if is_theme_assignment {
      new_lines.push(format!("theme = \"{}\"", theme_name));
      found_theme = true;
      continue;
    }
    new_lines.push(line.to_string());
  }

  if !found_theme {
    // Insert theme setting near the top (after any initial comments)
    let insert_pos = new_lines.iter().position(|l| !l.trim().starts_with('#') && !l.trim().is_empty()).unwrap_or(0);
    new_lines.insert(insert_pos, format!("theme = \"{}\"", theme_name));
  }

  if !ctx.quiet {
    println!("theme-manager: setting walker theme to \"{}\"", theme_name);
  }

  fs::write(&config_path, new_lines.join("\n") + "\n")?;
  Ok(())
}

fn apply_copy(
  ctx: &CommandContext<'_>,
  theme_dir: &Path,
  style_path: &Path,
  layout_path: &Path,
) -> Result<()> {
  // Create a temporary theme directory in walker themes
  let dest_theme_dir = ctx.config.walker_themes_dir.join(AUTO_THEME_NAME);
  cleanup_auto_theme_dir(&ctx.config.walker_themes_dir, ctx.quiet)?;
  fs::create_dir_all(&dest_theme_dir)?;

  if !ctx.quiet {
    println!(
      "theme-manager: copying walker theme from {}",
      theme_dir.to_string_lossy()
    );
  }

  // Copy style.css
  let dest_style = dest_theme_dir.join("style.css");
  fs::copy(style_path, &dest_style)?;

  // Copy layout.xml if it exists
  if layout_path.is_file() {
    let dest_layout = dest_theme_dir.join("layout.xml");
    fs::copy(layout_path, &dest_layout)?;
  }

  // Copy any other theme files (like hyprland_animations.conf)
  for entry in fs::read_dir(theme_dir)? {
    let entry = entry?;
    let path = entry.path();
    if path.is_file() {
      let name = path.file_name().unwrap();
      let dest = dest_theme_dir.join(name);
      if !dest.exists() {
        fs::copy(&path, &dest)?;
      }
    }
  }

  // Update walker config to use this theme
  update_walker_config(ctx, AUTO_THEME_NAME)?;

  Ok(())
}

fn apply_symlink(
  ctx: &CommandContext<'_>,
  theme_dir: &Path,
  style_path: &Path,
  layout_path: &Path,
) -> Result<()> {
  // Create a temporary theme directory in walker themes with symlinks
  let dest_theme_dir = ctx.config.walker_themes_dir.join(AUTO_THEME_NAME);
  cleanup_auto_theme_dir(&ctx.config.walker_themes_dir, ctx.quiet)?;
  fs::create_dir_all(&dest_theme_dir)?;

  if !ctx.quiet {
    println!(
      "theme-manager: linking walker theme from {}",
      theme_dir.to_string_lossy()
    );
  }

  // Symlink style.css
  let dest_style = dest_theme_dir.join("style.css");
  std::os::unix::fs::symlink(style_path, &dest_style)?;

  // Symlink layout.xml if it exists
  if layout_path.is_file() {
    let dest_layout = dest_theme_dir.join("layout.xml");
    std::os::unix::fs::symlink(layout_path, &dest_layout)?;
  }

  // Symlink any other theme files
  for entry in fs::read_dir(theme_dir)? {
    let entry = entry?;
    let path = entry.path();
    if path.is_file() {
      let name = path.file_name().unwrap();
      let dest = dest_theme_dir.join(name);
      if !dest.exists() {
        std::os::unix::fs::symlink(&path, &dest)?;
      }
    }
  }

  // Update walker config to use this theme
  update_walker_config(ctx, AUTO_THEME_NAME)?;

  Ok(())
}

fn cleanup_auto_theme_dir(walker_themes_dir: &Path, quiet: bool) -> Result<()> {
  let auto_theme_dir = walker_themes_dir.join(AUTO_THEME_NAME);
  if !auto_theme_dir.exists() {
    return Ok(());
  }

  if !quiet {
    println!(
      "theme-manager: removing stale walker auto theme {}",
      auto_theme_dir.to_string_lossy()
    );
  }
  if auto_theme_dir.is_dir() {
    fs::remove_dir_all(&auto_theme_dir)?;
  } else {
    fs::remove_file(&auto_theme_dir)?;
  }
  Ok(())
}
