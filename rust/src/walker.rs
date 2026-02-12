use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::theme_ops::{CommandContext, WalkerMode};

const WALKER_LINKS_FILE: &str = ".theme-manager-walker-links";

pub fn prepare_walker(ctx: &CommandContext<'_>, theme_dir: &Path) -> Result<()> {
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
  cleanup_walker_links(&ctx.config.walker_dir, ctx.quiet)?;

  let layout_path = walker_theme_dir.join("layout.xml");
  let apply_mode = ctx.config.walker_apply_mode.as_str();
  if apply_mode == "copy" {
    return apply_copy(ctx, &walker_theme_dir, &style_path, &layout_path);
  }

  apply_symlink(ctx, &walker_theme_dir, &style_path, &layout_path)
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
    if line.trim_start().starts_with("theme") && line.contains('=') {
      new_lines.push(format!("theme = \"{}\"", theme_name));
      found_theme = true;
    } else {
      new_lines.push(line.to_string());
    }
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
  let temp_theme_name = "theme-manager-auto";
  let dest_theme_dir = ctx.config.walker_themes_dir.join(temp_theme_name);
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
  update_walker_config(ctx, temp_theme_name)?;

  Ok(())
}

fn apply_symlink(
  ctx: &CommandContext<'_>,
  theme_dir: &Path,
  style_path: &Path,
  layout_path: &Path,
) -> Result<()> {
  // Create a temporary theme directory in walker themes with symlinks
  let temp_theme_name = "theme-manager-auto";
  let dest_theme_dir = ctx.config.walker_themes_dir.join(temp_theme_name);

  // Remove old auto theme if it exists
  if dest_theme_dir.exists() {
    fs::remove_dir_all(&dest_theme_dir)?;
  }
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
  update_walker_config(ctx, temp_theme_name)?;

  // Record that we created this theme dir for cleanup
  let manifest_path = ctx.config.walker_dir.join(WALKER_LINKS_FILE);
  fs::write(&manifest_path, temp_theme_name)?;

  Ok(())
}

fn cleanup_walker_links(walker_dir: &Path, quiet: bool) -> Result<()> {
  let manifest_path = walker_dir.join(WALKER_LINKS_FILE);
  if !manifest_path.is_file() {
    return Ok(());
  }

  let content = fs::read_to_string(&manifest_path)?;
  for line in content.lines() {
    let name = line.trim();
    if name.is_empty() {
      continue;
    }
    let path = walker_dir.join(name);
    let meta = match fs::symlink_metadata(&path) {
      Ok(meta) => meta,
      Err(_) => continue,
    };
    if !meta.file_type().is_symlink() {
      continue;
    }
    if !quiet {
      println!("theme-manager: removing walker link {}", path.to_string_lossy());
    }
    let _ = fs::remove_file(&path);
  }

  let _ = fs::remove_file(&manifest_path);
  Ok(())
}
