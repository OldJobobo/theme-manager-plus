use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;

use crate::omarchy::{self, RestartCommand};
use crate::theme_ops::{CommandContext, WaybarMode};

pub fn prepare_waybar(ctx: &CommandContext<'_>, theme_dir: &Path) -> Result<Option<RestartCommand>> {
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
    let restart_cmd = ctx
      .config
      .waybar_restart_cmd
      .clone()
      .unwrap_or_else(|| "tmplus-restart-waybar".to_string());
    let mut parts = restart_cmd.split_whitespace();
    let cmd = match parts.next() {
      Some(cmd) => cmd,
      None => return Err(anyhow!("invalid waybar restart command")),
    };
    if !omarchy::command_exists(cmd) {
      if !ctx.quiet {
        eprintln!(
          "theme-manager: waybar restart helper not found: {cmd}; falling back to copy mode"
        );
      }
    } else {
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
      return Ok(Some(RestartCommand {
        cmd: cmd.to_string(),
        args,
      }));
    }
  }

  apply_copy(ctx, &config_path, &style_path)
}

fn apply_copy(
  ctx: &CommandContext<'_>,
  config_path: &Path,
  style_path: &Path,
) -> Result<Option<RestartCommand>> {
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

  Ok(Some(RestartCommand {
    cmd: "omarchy-restart-waybar".to_string(),
    args: Vec::new(),
  }))
}
