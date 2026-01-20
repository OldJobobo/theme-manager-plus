use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;

use crate::omarchy;
use crate::theme_ops::{CommandContext, StarshipMode};

pub fn apply_starship(ctx: &CommandContext<'_>, theme_dir: &Path) -> Result<()> {
  let config_path = &ctx.config.starship_config;
  let themes_dir = &ctx.config.starship_themes_dir;

  fs::create_dir_all(
    config_path
      .parent()
      .ok_or_else(|| anyhow!("invalid starship config path"))?,
  )?;
  fs::create_dir_all(themes_dir)?;

  match &ctx.starship_mode {
    StarshipMode::None => Ok(()),
    StarshipMode::Preset { preset } => apply_preset(ctx, config_path, preset),
    StarshipMode::Named { name } => apply_named(ctx, config_path, themes_dir, name),
    StarshipMode::Theme { path } => {
      let theme_path = match path {
        Some(path) => path.clone(),
        None => theme_dir.join("starship.yaml"),
      };
      copy_theme(ctx, config_path, &theme_path)
    }
  }
}

fn apply_preset(ctx: &CommandContext<'_>, config_path: &Path, preset: &str) -> Result<()> {
  if !omarchy::command_exists("starship") {
    return Err(anyhow!("starship not found in PATH"));
  }
  if !ctx.quiet {
    println!("theme-manager: applying starship preset {preset}");
  }
  let output = std::process::Command::new("starship")
    .args(["preset", preset])
    .output()?;
  if !output.status.success() {
    return Err(anyhow!("failed to apply starship preset {preset}"));
  }
  fs::write(config_path, output.stdout)?;
  Ok(())
}

fn apply_named(
  ctx: &CommandContext<'_>,
  config_path: &Path,
  themes_dir: &Path,
  name: &str,
) -> Result<()> {
  let mut theme_path = themes_dir.join(name);
  if theme_path.extension().is_none() {
    theme_path.set_extension("toml");
  }
  if !theme_path.is_file() {
    return Err(anyhow!(
      "starship theme not found: {}",
      theme_path.to_string_lossy()
    ));
  }
  if !ctx.quiet {
    println!(
      "theme-manager: applying starship theme {}",
      theme_path.to_string_lossy()
    );
  }
  fs::copy(&theme_path, config_path)?;
  Ok(())
}

fn copy_theme(ctx: &CommandContext<'_>, config_path: &Path, theme_path: &Path) -> Result<()> {
  if !theme_path.is_file() {
    return Err(anyhow!(
      "starship theme file not found: {}",
      theme_path.to_string_lossy()
    ));
  }
  if !ctx.quiet {
    println!(
      "theme-manager: applying starship theme {}",
      theme_path.to_string_lossy()
    );
  }
  fs::copy(theme_path, config_path)?;
  Ok(())
}
