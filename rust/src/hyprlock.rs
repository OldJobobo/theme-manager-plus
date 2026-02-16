use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::ResolvedConfig;
use crate::omarchy;
use crate::paths::current_theme_name;
use crate::theme_ops::{CommandContext, HyprlockMode};

const OMARCHY_DEFAULT_THEME_NAME: &str = "omarchy-default";
const CURRENT_THEME_SOURCE_SUFFIX: &str = "/.config/omarchy/current/theme/hyprlock.conf";
const MINIMAL_SOURCE_ONLY_HYPRLOCK: &str = r#"source = ~/.config/omarchy/current/theme/hyprlock.conf

general {
    ignore_empty_input = true
}

animations {
    enabled = false
}

auth {
    fingerprint:enabled = true
}
"#;

pub fn prepare_hyprlock(ctx: &CommandContext<'_>, theme_dir: &Path) -> Result<()> {
  ensure_omarchy_default_theme_link(ctx.config, ctx.quiet)?;

  if matches!(ctx.hyprlock_mode, HyprlockMode::Named)
    && ctx.hyprlock_name.as_deref() == Some(OMARCHY_DEFAULT_THEME_NAME)
  {
    return apply_omarchy_default_theme_hyprlock(ctx, theme_dir);
  }

  let hyprlock_theme_dir = match ctx.hyprlock_mode {
    HyprlockMode::None => return Ok(()),
    HyprlockMode::Auto => theme_dir.join("hyprlock-theme"),
    HyprlockMode::Named => match &ctx.hyprlock_name {
      Some(name) => ctx.config.hyprlock_themes_dir.join(name),
      None => return Ok(()),
    },
  };

  if !hyprlock_theme_dir.is_dir() {
    if !ctx.quiet {
      eprintln!(
        "theme-manager: hyprlock theme directory not found: {}",
        hyprlock_theme_dir.to_string_lossy()
      );
    }
    return Ok(());
  }

  let source_config = hyprlock_theme_dir.join("hyprlock.conf");
  if !source_config.is_file() {
    if !ctx.quiet {
      eprintln!(
        "theme-manager: hyprlock theme missing hyprlock.conf in {}",
        hyprlock_theme_dir.to_string_lossy()
      );
    }
    return Ok(());
  }

  ensure_main_hyprlock_mode(ctx, &source_config)?;
  warn_if_hyprlock_source_mismatch(ctx, &ctx.config.current_theme_link.join("hyprlock.conf"))?;

  let apply_mode = ctx.config.hyprlock_apply_mode.as_str();
  if apply_mode == "copy" {
    return apply_copy(ctx, &source_config);
  }

  apply_symlink(ctx, &source_config)
}

fn apply_omarchy_default_theme_hyprlock(ctx: &CommandContext<'_>, theme_dir: &Path) -> Result<()> {
  let mut candidates = Vec::new();

  // In set/next/preset flows this is the selected theme source directory.
  candidates.push(theme_dir.join("hyprlock.conf"));

  // In standalone hyprlock flow, recover source from current theme name if possible.
  if let Some(theme_name) = current_theme_name(&ctx.config.current_theme_link)? {
    candidates.push(ctx.config.theme_root_dir.join(theme_name).join("hyprlock.conf"));
  }

  let Some(source_config) = candidates.into_iter().find(|p| p.is_file()) else {
    if !ctx.quiet {
      eprintln!(
        "theme-manager: omarchy-default hyprlock source not found; expected hyprlock.conf in active theme"
      );
    }
    return Ok(());
  };

  ensure_main_hyprlock_mode(ctx, &source_config)?;
  warn_if_hyprlock_source_mismatch(ctx, &ctx.config.current_theme_link.join("hyprlock.conf"))?;
  if ctx.config.hyprlock_apply_mode.as_str() == "copy" {
    return apply_copy(ctx, &source_config);
  }
  apply_symlink(ctx, &source_config)
}

fn ensure_main_hyprlock_mode(ctx: &CommandContext<'_>, source_config: &Path) -> Result<()> {
  let hyprlock_main = ctx.config.hyprlock_dir.join("hyprlock.conf");
  if let Some(parent) = hyprlock_main.parent() {
    fs::create_dir_all(parent)?;
  }

  // Only manage the host file when it participates in theme-manager source flow.
  let existing = fs::read_to_string(&hyprlock_main).unwrap_or_default();
  if !existing.is_empty() && !existing.contains(CURRENT_THEME_SOURCE_SUFFIX) {
    if !ctx.quiet {
      eprintln!(
        "theme-manager: warning: preserving custom {}; it does not source current theme hyprlock config",
        hyprlock_main.to_string_lossy()
      );
    }
    return Ok(());
  }

  let desired = if is_style_only_hyprlock_config(source_config)? {
    omarchy_base_hyprlock_wrapper(ctx.config).unwrap_or_else(|| MINIMAL_SOURCE_ONLY_HYPRLOCK.to_string())
  } else {
    MINIMAL_SOURCE_ONLY_HYPRLOCK.to_string()
  };

  if existing != desired {
    fs::write(&hyprlock_main, desired)?;
  }
  Ok(())
}

pub fn omarchy_default_theme_available(config: &ResolvedConfig) -> bool {
  omarchy_default_hyprlock_theme_dir(config).is_some()
}

fn is_style_only_hyprlock_config(path: &Path) -> Result<bool> {
  let content = fs::read_to_string(path)?;
  let has_widgets = ["background {", "input-field {", "label {", "image {", "shape {"]
    .iter()
    .any(|token| content.contains(token));
  Ok(!has_widgets)
}

fn omarchy_base_hyprlock_wrapper(config: &ResolvedConfig) -> Option<String> {
  let omarchy_root = omarchy::detect_omarchy_root(config)?;
  let wrapper = omarchy_root.join("config/hypr/hyprlock.conf");
  fs::read_to_string(wrapper).ok()
}

pub fn ensure_omarchy_default_theme_link(config: &ResolvedConfig, quiet: bool) -> Result<()> {
  let Some(default_theme_dir) = omarchy_default_hyprlock_theme_dir(config) else {
    return Ok(());
  };

  let link_path = config.hyprlock_themes_dir.join(OMARCHY_DEFAULT_THEME_NAME);
  if link_path.exists() {
    return Ok(());
  }

  fs::create_dir_all(&config.hyprlock_themes_dir)?;
  #[cfg(unix)]
  {
    std::os::unix::fs::symlink(&default_theme_dir, &link_path)?;
  }
  #[cfg(not(unix))]
  {
    return Ok(());
  }

  if !quiet {
    println!(
      "theme-manager: linked Omarchy default Hyprlock theme {} -> {}",
      link_path.to_string_lossy(),
      default_theme_dir.to_string_lossy()
    );
  }
  Ok(())
}

fn omarchy_default_hyprlock_theme_dir(config: &ResolvedConfig) -> Option<PathBuf> {
  let mut candidates = Vec::new();

  if let Some(omarchy_root) = omarchy::detect_omarchy_root(config) {
    candidates.push(
      omarchy_root
        .join("default/hyprlock/themes")
        .join(OMARCHY_DEFAULT_THEME_NAME),
    );
    candidates.push(omarchy_root.join("default/hyprlock"));
    candidates.push(omarchy_root.join("themes").join(OMARCHY_DEFAULT_THEME_NAME));
    candidates.push(omarchy_root.join("config/hypr"));
  }

  if let Ok(home) = std::env::var("HOME") {
    let home = PathBuf::from(home);
    candidates.push(
      home
        .join(".config/omarchy/default/hyprlock/themes")
        .join(OMARCHY_DEFAULT_THEME_NAME),
    );
    candidates.push(home.join(".config/omarchy/default/hyprlock"));
    candidates.push(
      home
        .join(".config/omarchy/themes")
        .join(OMARCHY_DEFAULT_THEME_NAME),
    );
    candidates.push(home.join(".config/omarchy/config/hypr"));
  }

  candidates
    .into_iter()
    .find(|candidate| candidate.join("hyprlock.conf").is_file())
}

fn apply_copy(ctx: &CommandContext<'_>, source_config: &Path) -> Result<()> {
  let dest = ctx.config.current_theme_link.join("hyprlock.conf");
  if let Some(parent) = dest.parent() {
    fs::create_dir_all(parent)?;
  }
  remove_existing(&dest)?;
  if !ctx.quiet {
    println!(
      "theme-manager: copying hyprlock config {} -> {}",
      source_config.to_string_lossy(),
      dest.to_string_lossy()
    );
  }
  fs::copy(source_config, dest)?;
  Ok(())
}

fn apply_symlink(ctx: &CommandContext<'_>, source_config: &Path) -> Result<()> {
  let dest = ctx.config.current_theme_link.join("hyprlock.conf");
  if let Some(parent) = dest.parent() {
    fs::create_dir_all(parent)?;
  }
  remove_existing(&dest)?;
  if !ctx.quiet {
    println!(
      "theme-manager: linking hyprlock config {} -> {}",
      source_config.to_string_lossy(),
      dest.to_string_lossy()
    );
  }
  #[cfg(unix)]
  std::os::unix::fs::symlink(source_config, &dest)?;
  #[cfg(not(unix))]
  fs::copy(source_config, &dest)?;
  Ok(())
}

fn warn_if_hyprlock_source_mismatch(ctx: &CommandContext<'_>, expected_target: &Path) -> Result<()> {
  let hyprlock_main = ctx.config.hyprlock_dir.join("hyprlock.conf");
  if !hyprlock_main.is_file() {
    return Ok(());
  }

  let content = fs::read_to_string(&hyprlock_main)?;
  let expected_abs = expected_target.to_string_lossy();
  let expected_suffix = CURRENT_THEME_SOURCE_SUFFIX;
  let source_ok = content.contains(expected_abs.as_ref()) || content.contains(expected_suffix);
  if !source_ok && !ctx.quiet {
    eprintln!(
      "theme-manager: warning: {} does not source current theme hyprlock config (expected {})",
      hyprlock_main.to_string_lossy(),
      expected_target.to_string_lossy()
    );
  }
  Ok(())
}

fn remove_existing(path: &Path) -> Result<()> {
  if let Ok(meta) = fs::symlink_metadata(path) {
    if meta.file_type().is_dir() {
      fs::remove_dir_all(path)?;
    } else {
      fs::remove_file(path)?;
    }
  }
  Ok(())
}
