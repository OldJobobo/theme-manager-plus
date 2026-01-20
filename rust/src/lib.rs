use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

pub mod cli;
pub mod config;
pub mod git_ops;
pub mod omarchy;
pub mod paths;
pub mod presets;
pub mod preview;
pub mod starship;
pub mod theme_ops;
pub mod tui;
pub mod waybar;

use cli::{Command, PresetCommand};
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
        debug_awww: cli.debug_awww,
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
        debug_awww: cli.debug_awww,
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
          debug_awww: cli.debug_awww,
        };
        theme_ops::cmd_set(&ctx, &selection.theme)?;
      }
    }
    Command::Current => {
      theme_ops::cmd_current(&config)?;
    }
    Command::BgNext => {
      theme_ops::cmd_bg_next(&config, cli.debug_awww)?;
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
    Command::Preset(args) => match args.command {
      PresetCommand::Save(save_args) => {
        let entry = build_preset_entry(&config, &save_args)?;
        presets::save_preset(&save_args.name, entry, &config)?;
      }
      PresetCommand::Load(load_args) => {
        let preset = presets::load_preset_definition(&config, &load_args.name)?;
        let quiet = load_args.quiet || config.quiet_default;

        let (waybar_mode, waybar_name) = if load_args.waybar.is_some() {
          parse_waybar(&config, load_args.waybar)?
        } else {
          preset_waybar(&preset)
        };

        let starship_mode = preset_starship(&preset);

        let ctx = theme_ops::CommandContext {
          config: &config,
          quiet,
          skip_apps,
          skip_hook,
          waybar_mode,
          waybar_name,
          starship_mode,
          debug_awww: cli.debug_awww,
        };
        theme_ops::cmd_set(&ctx, &preset.theme)?;
      }
      PresetCommand::List => {
        for name in presets::list_preset_names()? {
          println!("{name}");
        }
      }
      PresetCommand::Remove(remove_args) => {
        presets::remove_preset(&remove_args.name)?;
      }
    },
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

fn preset_waybar(preset: &presets::PresetDefinition) -> (WaybarMode, Option<String>) {
  match &preset.waybar {
    presets::PresetWaybarValue::None => (WaybarMode::None, None),
    presets::PresetWaybarValue::Auto => (WaybarMode::Auto, None),
    presets::PresetWaybarValue::Named(name) => (WaybarMode::Named, Some(name.clone())),
  }
}

fn preset_starship(preset: &presets::PresetDefinition) -> StarshipMode {
  match &preset.starship {
    presets::PresetStarshipValue::None => StarshipMode::None,
    presets::PresetStarshipValue::Preset(preset) => StarshipMode::Preset {
      preset: preset.clone(),
    },
    presets::PresetStarshipValue::Named(name) => StarshipMode::Named { name: name.clone() },
    presets::PresetStarshipValue::Theme => StarshipMode::Theme { path: None },
  }
}

fn build_preset_entry(
  config: &ResolvedConfig,
  args: &cli::PresetSaveArgs,
) -> Result<presets::PresetEntry> {
  let theme = match &args.theme {
    Some(theme) => {
      let normalized = paths::normalize_theme_name(theme);
      let theme_path = config.theme_root_dir.join(&normalized);
      if !theme_path.is_dir() && !paths::is_symlink(&theme_path)? {
        return Err(anyhow!("theme not found: {normalized}"));
      }
      normalized
    }
    None => paths::current_theme_name(&config.current_theme_link)?
      .ok_or_else(|| anyhow!("current theme not set: invalid link target"))?,
  };

  let waybar_value = match args.waybar.as_deref() {
    Some(spec) => parse_waybar_spec(spec)?,
    None => preset_waybar_defaults(config),
  };

  let starship_value = match args.starship.as_deref() {
    Some(spec) => parse_starship_spec(spec, config)?,
    None => preset_starship_defaults(config),
  };

  let waybar = match waybar_value {
    presets::PresetWaybarValue::None => presets::PresetWaybarEntry {
      mode: Some("none".to_string()),
      name: None,
    },
    presets::PresetWaybarValue::Auto => presets::PresetWaybarEntry {
      mode: Some("auto".to_string()),
      name: None,
    },
    presets::PresetWaybarValue::Named(name) => presets::PresetWaybarEntry {
      mode: Some("named".to_string()),
      name: Some(name),
    },
  };

  let starship = match starship_value {
    presets::PresetStarshipValue::None => presets::PresetStarshipEntry {
      mode: Some("none".to_string()),
      preset: None,
      name: None,
    },
    presets::PresetStarshipValue::Preset(preset) => presets::PresetStarshipEntry {
      mode: Some("preset".to_string()),
      preset: Some(preset),
      name: None,
    },
    presets::PresetStarshipValue::Named(name) => presets::PresetStarshipEntry {
      mode: Some("named".to_string()),
      preset: None,
      name: Some(name),
    },
    presets::PresetStarshipValue::Theme => presets::PresetStarshipEntry {
      mode: Some("theme".to_string()),
      preset: None,
      name: None,
    },
  };

  Ok(presets::PresetEntry {
    theme: Some(theme),
    waybar: Some(waybar),
    starship: Some(starship),
  })
}

fn preset_waybar_defaults(config: &ResolvedConfig) -> presets::PresetWaybarValue {
  match waybar_from_defaults(config) {
    (WaybarMode::Auto, _) => presets::PresetWaybarValue::Auto,
    (WaybarMode::Named, Some(name)) => presets::PresetWaybarValue::Named(name),
    _ => presets::PresetWaybarValue::None,
  }
}

fn preset_starship_defaults(config: &ResolvedConfig) -> presets::PresetStarshipValue {
  match starship_from_defaults(config) {
    StarshipMode::Preset { preset } => presets::PresetStarshipValue::Preset(preset),
    StarshipMode::Named { name } => presets::PresetStarshipValue::Named(name),
    _ => presets::PresetStarshipValue::None,
  }
}

fn parse_waybar_spec(spec: &str) -> Result<presets::PresetWaybarValue> {
  let cleaned = spec.trim();
  if cleaned.is_empty() {
    return Err(anyhow!("--waybar requires a value"));
  }
  match cleaned {
    "none" => Ok(presets::PresetWaybarValue::None),
    "auto" => Ok(presets::PresetWaybarValue::Auto),
    _ => Ok(presets::PresetWaybarValue::Named(cleaned.to_string())),
  }
}

fn parse_starship_spec(
  spec: &str,
  config: &ResolvedConfig,
) -> Result<presets::PresetStarshipValue> {
  let cleaned = spec.trim();
  if cleaned.is_empty() {
    return Err(anyhow!("--starship requires a value"));
  }
  if cleaned.eq_ignore_ascii_case("none") {
    return Ok(presets::PresetStarshipValue::None);
  }
  if cleaned.eq_ignore_ascii_case("theme") {
    return Ok(presets::PresetStarshipValue::Theme);
  }
  if let Some(rest) = cleaned.strip_prefix("preset:") {
    if rest.trim().is_empty() {
      return Err(anyhow!("--starship preset requires a name"));
    }
    return Ok(presets::PresetStarshipValue::Preset(rest.trim().to_string()));
  }
  if let Some(rest) = cleaned.strip_prefix("named:") {
    if rest.trim().is_empty() {
      return Err(anyhow!("--starship named requires a name"));
    }
    return Ok(presets::PresetStarshipValue::Named(rest.trim().to_string()));
  }

  let theme_path = config
    .starship_themes_dir
    .join(format!("{cleaned}.toml"));
  if theme_path.is_file() {
    return Ok(presets::PresetStarshipValue::Named(cleaned.to_string()));
  }

  Ok(presets::PresetStarshipValue::Preset(cleaned.to_string()))
}

#[allow(dead_code)]
fn to_path_string(path: &Path) -> String {
  path.to_string_lossy().to_string()
}

#[allow(dead_code)]
fn home_join(home: &Path, tail: &str) -> PathBuf {
  home.join(tail.trim_start_matches('/'))
}
