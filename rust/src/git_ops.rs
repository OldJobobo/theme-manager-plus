use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::ResolvedConfig;
use crate::omarchy;
use crate::paths::normalize_theme_name;
use crate::theme_ops::{self, CommandContext};

pub struct GitContext<'a> {
  pub config: &'a ResolvedConfig,
}

pub fn cmd_install(ctx: &GitContext<'_>, git_url: &str) -> Result<()> {
  if git_url.trim().is_empty() {
    return Err(anyhow!("missing git URL"));
  }
  if !omarchy::command_exists("git") {
    return Err(anyhow!("git is required to install themes"));
  }

  let repo_name = derive_repo_name(git_url);
  let theme_name = normalize_theme_name(&repo_name);

  fs::create_dir_all(&ctx.config.theme_root_dir)?;
  let theme_path = ctx.config.theme_root_dir.join(&theme_name);
  if theme_path.exists() {
    return Err(anyhow!("theme already exists: {theme_name}"));
  }

  let status = Command::new("git")
    .args(["clone", git_url, theme_path.to_string_lossy().as_ref()])
    .status()?;
  if !status.success() {
    return Err(anyhow!("git clone failed"));
  }

  let command_ctx = default_command_context(ctx.config);
  theme_ops::cmd_set(&command_ctx, &theme_name)?;
  Ok(())
}

pub fn cmd_update(ctx: &GitContext<'_>) -> Result<()> {
  if !ctx.config.theme_root_dir.is_dir() {
    return Err(anyhow!(
      "themes directory not found: {}",
      ctx.config.theme_root_dir.to_string_lossy()
    ));
  }
  if !omarchy::command_exists("git") {
    return Err(anyhow!("git is required to update themes"));
  }

  let mut updated = 0;
  for entry in fs::read_dir(&ctx.config.theme_root_dir)? {
    let entry = entry?;
    let path = resolve_entry(entry.path());
    if path.join(".git").is_dir() {
      let status = Command::new("git")
        .args(["-C", path.to_string_lossy().as_ref(), "pull"])
        .status()?;
      if status.success() {
        updated += 1;
      }
    }
  }

  if updated == 0 {
    eprintln!("theme-manager: no git-based themes found");
  }
  Ok(())
}

pub fn cmd_remove(ctx: &GitContext<'_>, theme: Option<&str>) -> Result<()> {
  let theme_name = match theme {
    Some(name) => normalize_theme_name(name),
    None => select_removable_theme(&ctx.config.theme_root_dir)?,
  };

  let theme_path = ctx.config.theme_root_dir.join(&theme_name);
  if !theme_path.exists() && !is_symlink(&theme_path)? {
    return Err(anyhow!("theme not found: {theme_name}"));
  }

  if is_current_theme(ctx.config, &theme_name)? {
    let entries = theme_ops::list_theme_entries(&ctx.config.theme_root_dir)?;
    if entries.len() <= 1 {
      return Err(anyhow!("cannot remove the only theme"));
    }
    let command_ctx = default_command_context(ctx.config);
    theme_ops::cmd_next(&command_ctx)?;
  }

  remove_path(&theme_path)?;
  Ok(())
}

fn derive_repo_name(git_url: &str) -> String {
  let name = git_url
    .trim_end_matches('/')
    .split('/')
    .last()
    .unwrap_or(git_url);
  let name = name.trim_end_matches(".git");
  let name = name.strip_prefix("omarchy-").unwrap_or(name);
  let name = name.strip_suffix("-theme").unwrap_or(name);
  name.to_string()
}

fn resolve_entry(path: PathBuf) -> PathBuf {
  if let Ok(target) = fs::read_link(&path) {
    if target.is_absolute() {
      return target;
    }
    if let Some(parent) = path.parent() {
      return parent.join(target);
    }
  }
  path
}

fn default_command_context<'a>(config: &'a ResolvedConfig) -> CommandContext<'a> {
  let (waybar_mode, waybar_name) = theme_ops::waybar_from_defaults(config);
  let starship_mode = theme_ops::starship_from_defaults(config);
  let skip_apps = std::env::var("THEME_MANAGER_SKIP_APPS").is_ok();
  let skip_hook = std::env::var("THEME_MANAGER_SKIP_HOOK").is_ok();
  CommandContext {
    config,
    quiet: config.quiet_default,
    skip_apps,
    skip_hook,
    waybar_mode,
    waybar_name,
    starship_mode,
  }
}

fn select_removable_theme(theme_root: &Path) -> Result<String> {
  let mut extras = Vec::new();
  for entry in fs::read_dir(theme_root)? {
    let entry = entry?;
    let path = entry.path();
    if path.is_dir() && !is_symlink(&path)? {
      if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        extras.push(name.to_string());
      }
    }
  }

  if extras.is_empty() {
    return Err(anyhow!("no removable themes found"));
  }

  extras.sort();

  println!("Select a theme to remove:");
  for (idx, name) in extras.iter().enumerate() {
    println!("{:>2}) {}", idx + 1, name);
  }

  let mut input = String::new();
  std::io::stdin().read_line(&mut input)?;
  let choice: usize = input.trim().parse().map_err(|_| anyhow!("invalid choice"))?;
  if choice == 0 || choice > extras.len() {
    return Err(anyhow!("invalid choice"));
  }
  Ok(extras[choice - 1].clone())
}

fn is_current_theme(config: &ResolvedConfig, theme_name: &str) -> Result<bool> {
  let target = match fs::read_link(&config.current_theme_link) {
    Ok(target) => target,
    Err(_) => return Ok(false),
  };
  let name = target
    .file_name()
    .and_then(|n| n.to_str())
    .unwrap_or("");
  Ok(name == theme_name)
}

fn is_symlink(path: &Path) -> Result<bool> {
  match fs::symlink_metadata(path) {
    Ok(meta) => Ok(meta.file_type().is_symlink()),
    Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
    Err(err) => Err(err.into()),
  }
}

fn remove_path(path: &Path) -> Result<()> {
  if is_symlink(path)? {
    fs::remove_file(path)?;
  } else if path.is_dir() {
    fs::remove_dir_all(path)?;
  } else if path.exists() {
    fs::remove_file(path)?;
  }
  Ok(())
}
