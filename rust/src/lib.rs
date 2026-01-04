use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

pub mod cli;
pub mod config;
pub mod git_ops;
pub mod omarchy;
pub mod paths;
pub mod preview;
pub mod starship;
pub mod theme_ops;
pub mod tui;
pub mod waybar;

use cli::Command;
use config::ResolvedConfig;
use theme_ops::{starship_from_defaults, waybar_from_defaults, StarshipMode, WaybarMode};

pub fn run(cli: cli::Cli) -> Result<()> {
  let config = ResolvedConfig::load()?;
  if let Some(bin_dir) = &config.omarchy_bin_dir {
    config::prepend_to_path(bin_dir);
  }

  let skip_apps = std::env::var("THEME_MANAGER_SKIP_APPS").is_ok();
  let skip_hook = std::env::var("THEME_MANAGER_SKIP_HOOK").is_ok();

  match cli.command {
    Command::List => {
      theme_ops::cmd_list(&config)?;
    }
    Command::Set(args) => {
      let (waybar_mode, waybar_name) = parse_waybar(&config, args.waybar)?;
      let starship_mode = starship_from_defaults(&config);
      let quiet = args.quiet || config.quiet_default;
      let ctx = theme_ops::CommandContext {
        config: &config,
        quiet,
        skip_apps,
        skip_hook,
        waybar_mode,
        waybar_name,
        starship_mode,
      };
      theme_ops::cmd_set(&ctx, &args.theme)?;
    }
    Command::Next(args) => {
      let (waybar_mode, waybar_name) = parse_waybar(&config, args.waybar)?;
      let starship_mode = starship_from_defaults(&config);
      let quiet = args.quiet || config.quiet_default;
      let ctx = theme_ops::CommandContext {
        config: &config,
        quiet,
        skip_apps,
        skip_hook,
        waybar_mode,
        waybar_name,
        starship_mode,
      };
      theme_ops::cmd_next(&ctx)?;
    }
    Command::Browse(args) => {
      let quiet = args.quiet || config.quiet_default;
      if let Some(selection) = tui::browse(&config, quiet)? {
        let (waybar_mode, waybar_name) = match selection.waybar {
          tui::WaybarSelection::UseDefaults => waybar_from_defaults(&config),
          tui::WaybarSelection::None => (WaybarMode::None, None),
          tui::WaybarSelection::Auto => (WaybarMode::Auto, None),
          tui::WaybarSelection::Named(name) => (WaybarMode::Named, Some(name)),
        };
        let starship_mode = match selection.starship {
          tui::StarshipSelection::UseDefaults => starship_from_defaults(&config),
          tui::StarshipSelection::None => StarshipMode::None,
          tui::StarshipSelection::Preset(preset) => StarshipMode::Preset { preset },
          tui::StarshipSelection::Named(name) => StarshipMode::Named { name },
          tui::StarshipSelection::Theme(path) => StarshipMode::Theme { path: Some(path) },
        };
        let ctx = theme_ops::CommandContext {
          config: &config,
          quiet,
          skip_apps,
          skip_hook,
          waybar_mode,
          waybar_name,
          starship_mode,
        };
        theme_ops::cmd_set(&ctx, &selection.theme)?;
      }
    }
    Command::Current => {
      theme_ops::cmd_current(&config)?;
    }
    Command::BgNext => {
      theme_ops::cmd_bg_next(&config)?;
    }
    Command::PrintConfig => {
      config::print_config(&config);
    }
    Command::Version => {
      theme_ops::cmd_version();
    }
    Command::Install(args) => {
      let ctx = git_ops::GitContext {
        config: &config,
      };
      git_ops::cmd_install(&ctx, &args.git_url)?;
    }
    Command::Update => {
      let ctx = git_ops::GitContext {
        config: &config,
      };
      git_ops::cmd_update(&ctx)?;
    }
    Command::Remove(args) => {
      let ctx = git_ops::GitContext {
        config: &config,
      };
      git_ops::cmd_remove(&ctx, args.theme.as_deref())?;
    }
  }

  Ok(())
}

fn parse_waybar(
  config: &ResolvedConfig,
  flag: Option<Option<String>>,
) -> Result<(WaybarMode, Option<String>)> {
  if let Some(flag_value) = flag {
    match flag_value {
      None => return Ok((WaybarMode::Auto, None)),
      Some(name) => {
        if name.trim().is_empty() {
          return Err(anyhow!("--waybar requires a name when used with ="));
        }
        return Ok((WaybarMode::Named, Some(name)));
      }
    }
  }

  Ok(waybar_from_defaults(config))
}

#[allow(dead_code)]
fn to_path_string(path: &Path) -> String {
  path.to_string_lossy().to_string()
}

#[allow(dead_code)]
fn home_join(home: &Path, tail: &str) -> PathBuf {
  home.join(tail.trim_start_matches('/'))
}
