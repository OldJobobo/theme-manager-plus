use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;

use crate::omarchy::{self, RestartAction, RestartCommand};
use crate::theme_ops::{CommandContext, WaybarMode};

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

  let apply_mode = ctx.config.waybar_apply_mode.as_str();
  if apply_mode == "exec" {
    if ctx.config.waybar_restart_cmd.is_none()
      && needs_waybar_copy_for_import(&ctx.config.waybar_dir, &style_path)?
    {
      if !ctx.quiet {
        eprintln!(
          "theme-manager: waybar style uses relative omarchy import; falling back to copy mode"
        );
      }
      return apply_copy(ctx, &config_path, &style_path);
    }

    if let Some(restart_cmd) = &ctx.config.waybar_restart_cmd {
      let mut parts = restart_cmd.split_whitespace();
      let cmd = match parts.next() {
        Some(cmd) => cmd,
        None => return Err(anyhow!("invalid waybar restart command")),
      };
      if !omarchy::command_exists(cmd) {
        return Err(anyhow!("waybar restart command not found: {cmd}"));
      }
      if !ctx.quiet {
        println!("theme-manager: applying waybar config via {cmd}");
      }
      let mut args: Vec<String> = parts.map(|part| part.to_string()).collect();
      let config_str = config_path.to_string_lossy().to_string();
      let style_str = style_path.to_string_lossy().to_string();
      args.push("-c".to_string());
      args.push(config_str);
      args.push("-s".to_string());
      args.push(style_str);
      return Ok(Some(RestartAction::Command(RestartCommand {
        cmd: cmd.to_string(),
        args,
      })));
    }

    if !ctx.quiet {
      println!("theme-manager: applying waybar config via built-in restart");
    }
    return Ok(Some(RestartAction::WaybarExec {
      config_path: config_path.to_path_buf(),
      style_path: style_path.to_path_buf(),
    }));
  }

  apply_copy(ctx, &config_path, &style_path)
}

fn needs_waybar_copy_for_import(waybar_dir: &Path, style_path: &Path) -> Result<bool> {
  if style_path.starts_with(waybar_dir) {
    return Ok(false);
  }
  let content = fs::read_to_string(style_path)?;
  Ok(content.contains("../omarchy/current/theme/waybar.css"))
}

fn apply_copy(
  ctx: &CommandContext<'_>,
  config_path: &Path,
  style_path: &Path,
) -> Result<Option<RestartAction>> {
  fs::create_dir_all(&ctx.config.waybar_dir)?;

  if !ctx.quiet {
    println!(
      "theme-manager: applying waybar config from {}",
      config_path.to_string_lossy()
    );
    println!(
      "theme-manager: applying waybar style from {}",
      style_path.to_string_lossy()
    );
  }

  let dest_config = ctx.config.waybar_dir.join("config.jsonc");
  let dest_style = ctx.config.waybar_dir.join("style.css");
  fs::copy(config_path, dest_config)?;
  fs::copy(style_path, dest_style)?;

  Ok(Some(RestartAction::Command(RestartCommand {
    cmd: "omarchy-restart-waybar".to_string(),
    args: Vec::new(),
  })))
}
