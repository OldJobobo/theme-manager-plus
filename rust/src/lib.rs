use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

pub mod cli;
pub mod config;
pub mod git_ops;
pub mod hyprlock;
pub mod omarchy;
pub mod paths;
pub mod presets;
pub mod preview;
pub mod starship;
pub mod theme_ops;
pub mod tui;
pub mod walker;
pub mod waybar;

use cli::{Command, PresetCommand};
use config::ResolvedConfig;
use theme_ops::{
  hyprlock_from_defaults, starship_from_defaults, waybar_from_defaults, walker_from_defaults,
  HyprlockMode, StarshipMode, WaybarMode, WalkerMode,
};

enum NamedMode {
  None,
  Auto,
  Named(String),
}

pub fn run(cli: cli::Cli) -> Result<()> {
  let config = ResolvedConfig::load()?;
  if let Some(bin_dir) = &config.omarchy_bin_dir {
    config::prepend_to_path(bin_dir);
  }

  let skip_apps = std::env::var("THEME_MANAGER_SKIP_APPS").is_ok();
  let skip_hook = std::env::var("THEME_MANAGER_SKIP_HOOK").is_ok();

  let command = cli.command.unwrap_or(Command::Browse(cli::BrowseArgs { quiet: false }));
  match command {
    Command::List => {
      theme_ops::cmd_list(&config)?;
    }
    Command::Set(args) => {
      let (waybar_mode, waybar_name) = parse_waybar_flag(&config, args.waybar)?;
      let (walker_mode, walker_name) = parse_walker_flag(&config, args.walker)?;
      let (hyprlock_mode, hyprlock_name) = parse_hyprlock_flag(&config, args.hyprlock)?;
      let starship_mode = starship_from_defaults(&config);
      let quiet = args.quiet || config.quiet_default;
      let ctx = build_context(
        &config,
        quiet,
        skip_apps,
        skip_hook,
        (waybar_mode, waybar_name),
        (walker_mode, walker_name),
        (hyprlock_mode, hyprlock_name),
        starship_mode,
        cli.debug_awww,
      );
      theme_ops::cmd_set(&ctx, &args.theme)?;
    }
    Command::Next(args) => {
      let (waybar_mode, waybar_name) = parse_waybar_flag(&config, args.waybar)?;
      let (walker_mode, walker_name) = parse_walker_flag(&config, args.walker)?;
      let (hyprlock_mode, hyprlock_name) = parse_hyprlock_flag(&config, args.hyprlock)?;
      let starship_mode = starship_from_defaults(&config);
      let quiet = args.quiet || config.quiet_default;
      let ctx = build_context(
        &config,
        quiet,
        skip_apps,
        skip_hook,
        (waybar_mode, waybar_name),
        (walker_mode, walker_name),
        (hyprlock_mode, hyprlock_name),
        starship_mode,
        cli.debug_awww,
      );
      theme_ops::cmd_next(&ctx)?;
    }
    Command::Browse(args) => {
      let quiet = args.quiet || config.quiet_default;
      if let Some(selection) = tui::browse(&config, quiet)? {
        let (waybar_mode, waybar_name) = match selection.waybar {
          tui::WaybarSelection::NoChange => (WaybarMode::None, None),
          tui::WaybarSelection::None => (WaybarMode::None, None),
          tui::WaybarSelection::Auto => (WaybarMode::Auto, None),
          tui::WaybarSelection::Named(name) => (WaybarMode::Named, Some(name)),
        };
        let (walker_mode, walker_name) = match selection.walker {
          tui::WalkerSelection::NoChange => (WalkerMode::None, None),
          tui::WalkerSelection::None => (WalkerMode::None, None),
          tui::WalkerSelection::Auto => (WalkerMode::Auto, None),
          tui::WalkerSelection::Named(name) => (WalkerMode::Named, Some(name)),
        };
        let starship_mode = match selection.starship {
          tui::StarshipSelection::NoChange => StarshipMode::None,
          tui::StarshipSelection::None => StarshipMode::None,
          tui::StarshipSelection::Preset(preset) => StarshipMode::Preset { preset },
          tui::StarshipSelection::Named(name) => StarshipMode::Named { name },
          tui::StarshipSelection::Theme(path) => StarshipMode::Theme { path: Some(path) },
        };
        let (hyprlock_mode, hyprlock_name) = match selection.hyprlock {
          tui::HyprlockSelection::NoChange => (HyprlockMode::None, None),
          tui::HyprlockSelection::None => (HyprlockMode::None, None),
          tui::HyprlockSelection::Auto => (HyprlockMode::Auto, None),
          tui::HyprlockSelection::Named(name) => (HyprlockMode::Named, Some(name)),
        };
        let ctx = build_context(
          &config,
          quiet,
          skip_apps,
          skip_hook,
          (waybar_mode, waybar_name),
          (walker_mode, walker_name),
          (hyprlock_mode, hyprlock_name),
          starship_mode,
          cli.debug_awww,
        );
        if selection.no_theme_change {
          if !skip_apps {
            let current_theme = paths::current_theme_dir(&config.current_theme_link)?;
            let waybar_restart = waybar::prepare_waybar(&ctx, &current_theme)?;
            walker::prepare_walker(&ctx, &current_theme)?;
            hyprlock::prepare_hyprlock(&ctx, &current_theme)?;
            starship::apply_starship(&ctx, &current_theme)?;
            omarchy::reload_components(
              quiet,
              waybar_restart,
              config.waybar_restart_logs,
            )?;
            omarchy::apply_theme_setters(quiet)?;
          }
        } else {
          theme_ops::cmd_set(&ctx, &selection.theme)?;
        }
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
          parse_waybar_flag(&config, load_args.waybar)?
        } else {
          preset_waybar(&preset)
        };
        let (walker_mode, walker_name) = if load_args.walker.is_some() {
          parse_walker_flag(&config, load_args.walker)?
        } else {
          preset_walker(&preset)
        };
        let (hyprlock_mode, hyprlock_name) = if load_args.hyprlock.is_some() {
          parse_hyprlock_flag(&config, load_args.hyprlock)?
        } else {
          preset_hyprlock(&preset)
        };

        let starship_mode = preset_starship(&preset);
        let ctx = build_context(
          &config,
          quiet,
          skip_apps,
          skip_hook,
          (waybar_mode, waybar_name),
          (walker_mode, walker_name),
          (hyprlock_mode, hyprlock_name),
          starship_mode,
          cli.debug_awww,
        );
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
    Command::Waybar(args) => {
      let mode = parse_named_mode_spec(&args.mode, "--waybar")?;
      let (waybar_mode, waybar_name) = named_mode_to_waybar(mode);
      let quiet = args.quiet || config.quiet_default;
      apply_waybar_only(&config, waybar_mode, waybar_name, quiet, skip_apps, cli.debug_awww)?;
    }
    Command::Walker(args) => {
      let mode = parse_named_mode_spec(&args.mode, "--walker")?;
      let (walker_mode, walker_name) = named_mode_to_walker(mode);
      let quiet = args.quiet || config.quiet_default;
      apply_walker_only(&config, walker_mode, walker_name, quiet, skip_apps, cli.debug_awww)?;
    }
    Command::Hyprlock(args) => {
      let mode = parse_named_mode_spec(&args.mode, "--hyprlock")?;
      let (hyprlock_mode, hyprlock_name) = named_mode_to_hyprlock(mode);
      let quiet = args.quiet || config.quiet_default;
      apply_hyprlock_only(
        &config,
        hyprlock_mode,
        hyprlock_name,
        quiet,
        skip_apps,
        cli.debug_awww,
      )?;
    }
    Command::Starship(args) => {
      let mode = parse_starship_spec(&args.mode, &config)?;
      let starship_mode = match mode {
        presets::PresetStarshipValue::None => StarshipMode::None,
        presets::PresetStarshipValue::Preset(preset) => StarshipMode::Preset { preset },
        presets::PresetStarshipValue::Named(name) => StarshipMode::Named { name },
        presets::PresetStarshipValue::Theme => StarshipMode::Theme { path: None },
      };
      let quiet = args.quiet || config.quiet_default;
      apply_starship_only(&config, starship_mode, quiet, skip_apps, cli.debug_awww)?;
    }
  }

  Ok(())
}

fn parse_waybar_flag(
  config: &ResolvedConfig,
  flag: Option<Option<String>>,
) -> Result<(WaybarMode, Option<String>)> {
  if let Some(flag_value) = flag {
    return flag_to_named_mode(flag_value, "--waybar").map(named_mode_to_waybar);
  }
  Ok(waybar_from_defaults(config))
}

fn parse_walker_flag(
  config: &ResolvedConfig,
  flag: Option<Option<String>>,
) -> Result<(WalkerMode, Option<String>)> {
  if let Some(flag_value) = flag {
    return flag_to_named_mode(flag_value, "--walker").map(named_mode_to_walker);
  }
  Ok(walker_from_defaults(config))
}

fn parse_hyprlock_flag(
  config: &ResolvedConfig,
  flag: Option<Option<String>>,
) -> Result<(HyprlockMode, Option<String>)> {
  if let Some(flag_value) = flag {
    return flag_to_named_mode(flag_value, "--hyprlock").map(named_mode_to_hyprlock);
  }
  Ok(hyprlock_from_defaults(config))
}

fn build_context<'a>(
  config: &'a ResolvedConfig,
  quiet: bool,
  skip_apps: bool,
  skip_hook: bool,
  waybar: (WaybarMode, Option<String>),
  walker: (WalkerMode, Option<String>),
  hyprlock: (HyprlockMode, Option<String>),
  starship_mode: StarshipMode,
  debug_awww: bool,
) -> theme_ops::CommandContext<'a> {
  theme_ops::CommandContext {
    config,
    quiet,
    skip_apps,
    skip_hook,
    waybar_mode: waybar.0,
    waybar_name: waybar.1,
    walker_mode: walker.0,
    walker_name: walker.1,
    hyprlock_mode: hyprlock.0,
    hyprlock_name: hyprlock.1,
    starship_mode,
    debug_awww,
  }
}

fn flag_to_named_mode(flag: Option<String>, arg_name: &str) -> Result<NamedMode> {
  match flag {
    None => Ok(NamedMode::Auto),
    Some(name) => {
      if name.trim().is_empty() {
        return Err(anyhow!("{arg_name} requires a name when used with ="));
      }
      Ok(NamedMode::Named(name))
    }
  }
}

fn parse_named_mode_spec(spec: &str, arg_name: &str) -> Result<NamedMode> {
  let cleaned = spec.trim();
  if cleaned.is_empty() {
    return Err(anyhow!("{arg_name} requires a value"));
  }
  match cleaned {
    "none" => Ok(NamedMode::None),
    "auto" => Ok(NamedMode::Auto),
    _ => Ok(NamedMode::Named(cleaned.to_string())),
  }
}

fn named_mode_to_waybar(mode: NamedMode) -> (WaybarMode, Option<String>) {
  match mode {
    NamedMode::None => (WaybarMode::None, None),
    NamedMode::Auto => (WaybarMode::Auto, None),
    NamedMode::Named(name) => (WaybarMode::Named, Some(name)),
  }
}

fn named_mode_to_walker(mode: NamedMode) -> (WalkerMode, Option<String>) {
  match mode {
    NamedMode::None => (WalkerMode::None, None),
    NamedMode::Auto => (WalkerMode::Auto, None),
    NamedMode::Named(name) => (WalkerMode::Named, Some(name)),
  }
}

fn named_mode_to_hyprlock(mode: NamedMode) -> (HyprlockMode, Option<String>) {
  match mode {
    NamedMode::None => (HyprlockMode::None, None),
    NamedMode::Auto => (HyprlockMode::Auto, None),
    NamedMode::Named(name) => (HyprlockMode::Named, Some(name)),
  }
}

fn preset_waybar(preset: &presets::PresetDefinition) -> (WaybarMode, Option<String>) {
  match &preset.waybar {
    presets::PresetWaybarValue::None => (WaybarMode::None, None),
    presets::PresetWaybarValue::Auto => (WaybarMode::Auto, None),
    presets::PresetWaybarValue::Named(name) => (WaybarMode::Named, Some(name.clone())),
  }
}

fn preset_walker(preset: &presets::PresetDefinition) -> (WalkerMode, Option<String>) {
  match &preset.walker {
    presets::PresetWalkerValue::None => (WalkerMode::None, None),
    presets::PresetWalkerValue::Auto => (WalkerMode::Auto, None),
    presets::PresetWalkerValue::Named(name) => (WalkerMode::Named, Some(name.clone())),
  }
}

fn preset_hyprlock(preset: &presets::PresetDefinition) -> (HyprlockMode, Option<String>) {
  match &preset.hyprlock {
    presets::PresetHyprlockValue::None => (HyprlockMode::None, None),
    presets::PresetHyprlockValue::Auto => (HyprlockMode::Auto, None),
    presets::PresetHyprlockValue::Named(name) => (HyprlockMode::Named, Some(name.clone())),
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
      let theme_path = theme_ops::resolve_theme_path(config, &normalized)?;
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

  let walker_value = match args.walker.as_deref() {
    Some(spec) => parse_walker_spec(spec)?,
    None => preset_walker_defaults(config),
  };
  let hyprlock_value = match args.hyprlock.as_deref() {
    Some(spec) => parse_hyprlock_spec(spec)?,
    None => preset_hyprlock_defaults(config),
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

  let walker = match walker_value {
    presets::PresetWalkerValue::None => presets::PresetWalkerEntry {
      mode: Some("none".to_string()),
      name: None,
    },
    presets::PresetWalkerValue::Auto => presets::PresetWalkerEntry {
      mode: Some("auto".to_string()),
      name: None,
    },
    presets::PresetWalkerValue::Named(name) => presets::PresetWalkerEntry {
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
    walker: Some(walker),
    hyprlock: Some(match hyprlock_value {
      presets::PresetHyprlockValue::None => presets::PresetHyprlockEntry {
        mode: Some("none".to_string()),
        name: None,
      },
      presets::PresetHyprlockValue::Auto => presets::PresetHyprlockEntry {
        mode: Some("auto".to_string()),
        name: None,
      },
      presets::PresetHyprlockValue::Named(name) => presets::PresetHyprlockEntry {
        mode: Some("named".to_string()),
        name: Some(name),
      },
    }),
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

fn preset_walker_defaults(config: &ResolvedConfig) -> presets::PresetWalkerValue {
  match walker_from_defaults(config) {
    (WalkerMode::Auto, _) => presets::PresetWalkerValue::Auto,
    (WalkerMode::Named, Some(name)) => presets::PresetWalkerValue::Named(name),
    _ => presets::PresetWalkerValue::None,
  }
}

fn preset_starship_defaults(config: &ResolvedConfig) -> presets::PresetStarshipValue {
  match starship_from_defaults(config) {
    StarshipMode::Preset { preset } => presets::PresetStarshipValue::Preset(preset),
    StarshipMode::Named { name } => presets::PresetStarshipValue::Named(name),
    _ => presets::PresetStarshipValue::None,
  }
}

fn preset_hyprlock_defaults(config: &ResolvedConfig) -> presets::PresetHyprlockValue {
  match hyprlock_from_defaults(config) {
    (HyprlockMode::Auto, _) => presets::PresetHyprlockValue::Auto,
    (HyprlockMode::Named, Some(name)) => presets::PresetHyprlockValue::Named(name),
    _ => presets::PresetHyprlockValue::None,
  }
}

fn parse_waybar_spec(spec: &str) -> Result<presets::PresetWaybarValue> {
  let mode = parse_named_mode_spec(spec, "--waybar")?;
  Ok(match mode {
    NamedMode::None => presets::PresetWaybarValue::None,
    NamedMode::Auto => presets::PresetWaybarValue::Auto,
    NamedMode::Named(name) => presets::PresetWaybarValue::Named(name),
  })
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

fn apply_waybar_only(
  config: &ResolvedConfig,
  waybar_mode: WaybarMode,
  waybar_name: Option<String>,
  quiet: bool,
  skip_apps: bool,
  debug_awww: bool,
) -> Result<()> {
  if skip_apps {
    return Ok(());
  }
  let theme_dir = paths::current_theme_dir(&config.current_theme_link)?;
  let ctx = build_context(
    config,
    quiet,
    skip_apps,
    true,
    (waybar_mode, waybar_name),
    (WalkerMode::None, None),
    (HyprlockMode::None, None),
    StarshipMode::None,
    debug_awww,
  );
  let restart = waybar::prepare_waybar(&ctx, &theme_dir)?;
  omarchy::restart_waybar_only(quiet, restart, config.waybar_restart_logs)?;
  Ok(())
}

fn apply_walker_only(
  config: &ResolvedConfig,
  walker_mode: WalkerMode,
  walker_name: Option<String>,
  quiet: bool,
  skip_apps: bool,
  debug_awww: bool,
) -> Result<()> {
  if skip_apps {
    return Ok(());
  }
  let theme_dir = paths::current_theme_dir(&config.current_theme_link)?;
  let ctx = build_context(
    config,
    quiet,
    skip_apps,
    true,
    (WaybarMode::None, None),
    (walker_mode, walker_name),
    (HyprlockMode::None, None),
    StarshipMode::None,
    debug_awww,
  );
  walker::prepare_walker(&ctx, &theme_dir)?;
  omarchy::restart_walker_only(quiet)?;
  Ok(())
}

fn parse_walker_spec(spec: &str) -> Result<presets::PresetWalkerValue> {
  let mode = parse_named_mode_spec(spec, "--walker")?;
  Ok(match mode {
    NamedMode::None => presets::PresetWalkerValue::None,
    NamedMode::Auto => presets::PresetWalkerValue::Auto,
    NamedMode::Named(name) => presets::PresetWalkerValue::Named(name),
  })
}

fn parse_hyprlock_spec(spec: &str) -> Result<presets::PresetHyprlockValue> {
  let mode = parse_named_mode_spec(spec, "--hyprlock")?;
  Ok(match mode {
    NamedMode::None => presets::PresetHyprlockValue::None,
    NamedMode::Auto => presets::PresetHyprlockValue::Auto,
    NamedMode::Named(name) => presets::PresetHyprlockValue::Named(name),
  })
}

fn apply_starship_only(
  config: &ResolvedConfig,
  starship_mode: StarshipMode,
  quiet: bool,
  skip_apps: bool,
  debug_awww: bool,
) -> Result<()> {
  if skip_apps {
    return Ok(());
  }
  let theme_dir = paths::current_theme_dir(&config.current_theme_link)?;
  let ctx = build_context(
    config,
    quiet,
    skip_apps,
    true,
    (WaybarMode::None, None),
    (WalkerMode::None, None),
    (HyprlockMode::None, None),
    starship_mode,
    debug_awww,
  );
  starship::apply_starship(&ctx, &theme_dir)?;
  Ok(())
}

fn apply_hyprlock_only(
  config: &ResolvedConfig,
  hyprlock_mode: HyprlockMode,
  hyprlock_name: Option<String>,
  quiet: bool,
  skip_apps: bool,
  debug_awww: bool,
) -> Result<()> {
  if skip_apps {
    return Ok(());
  }
  let theme_dir = paths::current_theme_dir(&config.current_theme_link)?;
  let ctx = build_context(
    config,
    quiet,
    skip_apps,
    true,
    (WaybarMode::None, None),
    (WalkerMode::None, None),
    (hyprlock_mode, hyprlock_name),
    StarshipMode::None,
    debug_awww,
  );
  hyprlock::prepare_hyprlock(&ctx, &theme_dir)?;
  omarchy::restart_hyprlock_only(quiet)?;
  Ok(())
}

#[allow(dead_code)]
fn to_path_string(path: &Path) -> String {
  path.to_string_lossy().to_string()
}

#[allow(dead_code)]
fn home_join(home: &Path, tail: &str) -> PathBuf {
  home.join(tail.trim_start_matches('/'))
}
