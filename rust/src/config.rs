use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize, Default)]
pub struct FileConfig {
  pub paths: Option<PathsConfig>,
  pub waybar: Option<WaybarConfig>,
  pub walker: Option<WalkerConfig>,
  pub starship: Option<StarshipConfig>,
  pub tui: Option<TuiConfig>,
  pub behavior: Option<BehaviorConfig>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PathsConfig {
  pub theme_root_dir: Option<String>,
  pub current_theme_link: Option<String>,
  pub current_background_link: Option<String>,
  pub omarchy_bin_dir: Option<String>,
  pub waybar_dir: Option<String>,
  pub waybar_themes_dir: Option<String>,
  pub walker_dir: Option<String>,
  pub walker_themes_dir: Option<String>,
  pub starship_config: Option<String>,
  pub starship_themes_dir: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct WaybarConfig {
  pub apply_mode: Option<String>,
  pub restart_cmd: Option<String>,
  pub restart_logs: Option<bool>,
  pub default_mode: Option<String>,
  pub default_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct WalkerConfig {
  pub apply_mode: Option<String>,
  pub default_mode: Option<String>,
  pub default_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct StarshipConfig {
  pub default_mode: Option<String>,
  pub default_preset: Option<String>,
  pub default_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TuiConfig {
  pub apply_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct BehaviorConfig {
  pub quiet_default: Option<bool>,
  pub awww_transition: Option<bool>,
  pub awww_transition_type: Option<String>,
  pub awww_transition_duration: Option<f32>,
  pub awww_transition_angle: Option<f32>,
  pub awww_transition_fps: Option<u32>,
  pub awww_transition_pos: Option<String>,
  pub awww_transition_bezier: Option<String>,
  pub awww_transition_wave: Option<String>,
  pub awww_auto_start: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct ResolvedConfig {
  pub theme_root_dir: PathBuf,
  pub current_theme_link: PathBuf,
  pub current_background_link: PathBuf,
  pub omarchy_bin_dir: Option<PathBuf>,
  pub waybar_dir: PathBuf,
  pub waybar_themes_dir: PathBuf,
  pub waybar_apply_mode: String,
  pub waybar_restart_cmd: Option<String>,
  pub waybar_restart_logs: bool,
  pub default_waybar_mode: Option<String>,
  pub default_waybar_name: Option<String>,
  pub walker_dir: PathBuf,
  pub walker_themes_dir: PathBuf,
  pub walker_apply_mode: String,
  pub default_walker_mode: Option<String>,
  pub default_walker_name: Option<String>,
  pub starship_config: PathBuf,
  pub starship_themes_dir: PathBuf,
  pub default_starship_mode: Option<String>,
  pub default_starship_preset: Option<String>,
  pub default_starship_name: Option<String>,
  pub tui_apply_key: Option<String>,
  pub quiet_default: bool,
  pub awww_transition: bool,
  pub awww_transition_type: String,
  pub awww_transition_duration: f32,
  pub awww_transition_angle: f32,
  pub awww_transition_fps: u32,
  pub awww_transition_pos: String,
  pub awww_transition_bezier: String,
  pub awww_transition_wave: String,
  pub awww_auto_start: bool,
}

impl ResolvedConfig {
  pub fn load() -> Result<Self> {
    let home = env::var("HOME").map_err(|_| anyhow!("HOME is not set"))?;
    let home_path = PathBuf::from(&home);

    let mut config = ResolvedConfig::defaults(&home_path);

    if let Some(user_cfg) = load_toml(&home_path.join(".config/theme-manager/config.toml"))? {
      config.apply_file_config(&user_cfg, &home_path);
    }
    if let Some(local_cfg) = load_toml(&current_dir()?.join(".theme-manager.toml"))? {
      config.apply_file_config(&local_cfg, &home_path);
    }

    config.apply_env_overrides(&home_path)?;
    Ok(config)
  }

  fn defaults(home: &Path) -> Self {
    let theme_root_dir = home.join(".config/omarchy/themes");
    let current_theme_link = home.join(".config/omarchy/current/theme");
    let current_background_link = home.join(".config/omarchy/current/background");
    let default_omarchy_bin = home.join(".local/share/omarchy/bin");
    let waybar_dir = home.join(".config/waybar");
    let waybar_themes_dir = waybar_dir.join("themes");
    let walker_dir = home.join(".config/walker");
    let walker_themes_dir = walker_dir.join("themes");
    let starship_config = home.join(".config/starship.toml");
    let starship_themes_dir = home.join(".config/starship-themes");

    ResolvedConfig {
      theme_root_dir,
      current_theme_link,
      current_background_link,
      omarchy_bin_dir: if default_omarchy_bin.is_dir() {
        Some(default_omarchy_bin)
      } else {
        None
      },
      waybar_dir,
      waybar_themes_dir,
      waybar_apply_mode: "symlink".to_string(),
      waybar_restart_cmd: None,
      waybar_restart_logs: false,
      default_waybar_mode: None,
      default_waybar_name: None,
      walker_dir,
      walker_themes_dir,
      walker_apply_mode: "symlink".to_string(),
      default_walker_mode: None,
      default_walker_name: None,
      starship_config,
      starship_themes_dir,
      default_starship_mode: None,
      default_starship_preset: None,
      default_starship_name: None,
      tui_apply_key: None,
      quiet_default: false,
      awww_transition: true,
      awww_transition_type: "grow".to_string(),
      awww_transition_duration: 2.4,
      awww_transition_angle: 35.0,
      awww_transition_fps: 60,
      awww_transition_pos: "center".to_string(),
      awww_transition_bezier: ".42,0,.2,1".to_string(),
      awww_transition_wave: "28,12".to_string(),
      awww_auto_start: false,
    }
  }

  fn apply_file_config(&mut self, cfg: &FileConfig, home: &Path) {
    if let Some(paths) = &cfg.paths {
      if let Some(val) = &paths.theme_root_dir {
        self.theme_root_dir = expand_path(val, home);
      }
      if let Some(val) = &paths.current_theme_link {
        self.current_theme_link = expand_path(val, home);
      }
      if let Some(val) = &paths.current_background_link {
        self.current_background_link = expand_path(val, home);
      }
      if let Some(val) = &paths.omarchy_bin_dir {
        self.omarchy_bin_dir = Some(expand_path(val, home));
      }
      if let Some(val) = &paths.waybar_dir {
        self.waybar_dir = expand_path(val, home);
      }
      if let Some(val) = &paths.waybar_themes_dir {
        self.waybar_themes_dir = expand_path(val, home);
      } else {
        self.waybar_themes_dir = self.waybar_dir.join("themes");
      }
      if let Some(val) = &paths.walker_dir {
        self.walker_dir = expand_path(val, home);
      }
      if let Some(val) = &paths.walker_themes_dir {
        self.walker_themes_dir = expand_path(val, home);
      } else {
        self.walker_themes_dir = self.walker_dir.join("themes");
      }
      if let Some(val) = &paths.starship_config {
        self.starship_config = expand_path(val, home);
      }
      if let Some(val) = &paths.starship_themes_dir {
        self.starship_themes_dir = expand_path(val, home);
      }
    }

    if let Some(waybar) = &cfg.waybar {
      if let Some(val) = &waybar.apply_mode {
        self.waybar_apply_mode = val.clone();
      }
      if let Some(val) = &waybar.restart_cmd {
        self.waybar_restart_cmd = Some(val.clone());
      }
      if let Some(val) = waybar.restart_logs {
        self.waybar_restart_logs = val;
      }
      if let Some(val) = &waybar.default_mode {
        self.default_waybar_mode = Some(val.clone());
      }
      if let Some(val) = &waybar.default_name {
        self.default_waybar_name = Some(val.clone());
      }
    }

    if let Some(starship) = &cfg.starship {
      if let Some(val) = &starship.default_mode {
        self.default_starship_mode = Some(val.clone());
      }
      if let Some(val) = &starship.default_preset {
        self.default_starship_preset = Some(val.clone());
      }
      if let Some(val) = &starship.default_name {
        self.default_starship_name = Some(val.clone());
      }
    }

    if let Some(walker) = &cfg.walker {
      if let Some(val) = &walker.apply_mode {
        self.walker_apply_mode = val.clone();
      }
      if let Some(val) = &walker.default_mode {
        self.default_walker_mode = Some(val.clone());
      }
      if let Some(val) = &walker.default_name {
        self.default_walker_name = Some(val.clone());
      }
    }

    if let Some(tui) = &cfg.tui {
      if let Some(val) = &tui.apply_key {
        self.tui_apply_key = Some(val.clone());
      }
    }

    if let Some(behavior) = &cfg.behavior {
      if let Some(val) = behavior.quiet_default {
        self.quiet_default = val;
      }
      if let Some(val) = behavior.awww_transition {
        self.awww_transition = val;
      }
      if let Some(val) = &behavior.awww_transition_type {
        self.awww_transition_type = val.clone();
      }
      if let Some(val) = behavior.awww_transition_duration {
        self.awww_transition_duration = val;
      }
      if let Some(val) = behavior.awww_transition_angle {
        self.awww_transition_angle = val;
      }
      if let Some(val) = behavior.awww_transition_fps {
        self.awww_transition_fps = val;
      }
      if let Some(val) = &behavior.awww_transition_pos {
        self.awww_transition_pos = val.clone();
      }
      if let Some(val) = &behavior.awww_transition_bezier {
        self.awww_transition_bezier = val.clone();
      }
      if let Some(val) = &behavior.awww_transition_wave {
        self.awww_transition_wave = val.clone();
      }
      if let Some(val) = behavior.awww_auto_start {
        self.awww_auto_start = val;
      }
    }
  }

  fn apply_env_overrides(&mut self, home: &Path) -> Result<()> {
    if let Ok(val) = env::var("THEME_ROOT_DIR") {
      self.theme_root_dir = expand_path(&val, home);
    }
    if let Ok(val) = env::var("CURRENT_THEME_LINK") {
      self.current_theme_link = expand_path(&val, home);
    }
    if let Ok(val) = env::var("CURRENT_BACKGROUND_LINK") {
      self.current_background_link = expand_path(&val, home);
    }
    if let Ok(val) = env::var("OMARCHY_BIN_DIR") {
      self.omarchy_bin_dir = Some(expand_path(&val, home));
    }
    if self.omarchy_bin_dir.is_none() {
      if let Ok(val) = env::var("OMARCHY_PATH") {
        if !val.trim().is_empty() {
          let candidate = expand_path(&format!("{val}/bin"), home);
          if candidate.is_dir() {
            self.omarchy_bin_dir = Some(candidate);
          }
        }
      }
    }
    if let Ok(val) = env::var("WAYBAR_DIR") {
      self.waybar_dir = expand_path(&val, home);
    }
    if let Ok(val) = env::var("WAYBAR_THEMES_DIR") {
      self.waybar_themes_dir = expand_path(&val, home);
    }
    if let Ok(val) = env::var("WALKER_DIR") {
      self.walker_dir = expand_path(&val, home);
    }
    if let Ok(val) = env::var("WALKER_THEMES_DIR") {
      self.walker_themes_dir = expand_path(&val, home);
    }
    if let Ok(val) = env::var("WALKER_APPLY_MODE") {
      self.walker_apply_mode = val;
    }
    if let Ok(val) = env::var("DEFAULT_WALKER_MODE") {
      self.default_walker_mode = Some(val);
    }
    if let Ok(val) = env::var("DEFAULT_WALKER_NAME") {
      self.default_walker_name = Some(val);
    }
    if let Ok(val) = env::var("WAYBAR_APPLY_MODE") {
      self.waybar_apply_mode = val;
    }
    if let Ok(val) = env::var("WAYBAR_RESTART_CMD") {
      self.waybar_restart_cmd = Some(val);
    }
    if let Ok(val) = env::var("WAYBAR_RESTART_LOGS") {
      if val == "1" || val.eq_ignore_ascii_case("true") {
        self.waybar_restart_logs = true;
      } else if val == "0" || val.eq_ignore_ascii_case("false") {
        self.waybar_restart_logs = false;
      }
    }
    if let Ok(val) = env::var("DEFAULT_WAYBAR_MODE") {
      self.default_waybar_mode = Some(val);
    }
    if let Ok(val) = env::var("DEFAULT_WAYBAR_NAME") {
      self.default_waybar_name = Some(val);
    }
    if let Ok(val) = env::var("STARSHIP_CONFIG") {
      self.starship_config = expand_path(&val, home);
    }
    if let Ok(val) = env::var("STARSHIP_THEMES_DIR") {
      self.starship_themes_dir = expand_path(&val, home);
    }
    if let Ok(val) = env::var("DEFAULT_STARSHIP_MODE") {
      self.default_starship_mode = Some(val);
    }
    if let Ok(val) = env::var("DEFAULT_STARSHIP_PRESET") {
      self.default_starship_preset = Some(val);
    }
    if let Ok(val) = env::var("DEFAULT_STARSHIP_NAME") {
      self.default_starship_name = Some(val);
    }
    if let Ok(val) = env::var("QUIET_MODE_DEFAULT") {
      if val == "1" || val.eq_ignore_ascii_case("true") {
        self.quiet_default = true;
      }
    }
    if env::var("QUIET_MODE").is_ok() {
      self.quiet_default = true;
    }
    if let Ok(val) = env::var("THEME_MANAGER_AWWW_TRANSITION") {
      if val == "0" || val.eq_ignore_ascii_case("false") {
        self.awww_transition = false;
      } else {
        self.awww_transition = true;
      }
    }
    if let Ok(val) = env::var("THEME_MANAGER_AWWW_AUTO_START") {
      if val == "1" || val.eq_ignore_ascii_case("true") {
        self.awww_auto_start = true;
      }
    }
    if let Ok(val) = env::var("THEME_MANAGER_AWWW_TRANSITION_POS") {
      if !val.is_empty() {
        self.awww_transition_pos = val;
      }
    }
    if let Ok(val) = env::var("THEME_MANAGER_AWWW_TRANSITION_BEZIER") {
      if !val.is_empty() {
        self.awww_transition_bezier = val;
      }
    }
    if let Ok(val) = env::var("THEME_MANAGER_AWWW_TRANSITION_WAVE") {
      if !val.is_empty() {
        self.awww_transition_wave = val;
      }
    }
    Ok(())
  }
}

fn load_toml(path: &Path) -> Result<Option<FileConfig>> {
  if !path.is_file() {
    return Ok(None);
  }
  let content = fs::read_to_string(path)?;
  let cfg: FileConfig = toml::from_str(&content)?;
  Ok(Some(cfg))
}

fn expand_path(path: &str, home: &Path) -> PathBuf {
  let mut expanded = path.replace("${HOME}", &home.to_string_lossy());
  expanded = expanded.replace("$HOME", &home.to_string_lossy());
  if expanded.starts_with("~/") {
    return home.join(expanded.trim_start_matches("~/"));
  }
  if expanded == "~" {
    return home.to_path_buf();
  }
  PathBuf::from(expanded)
}

pub fn prepend_to_path(dir: &Path) {
  if let Some(dir_str) = dir.to_str() {
    let current = env::var("PATH").unwrap_or_default();
    let new_path = format!("{dir_str}:{current}");
    env::set_var("PATH", new_path);
  }
}

fn current_dir() -> Result<PathBuf> {
  env::current_dir().map_err(|err| anyhow!("failed to get current dir: {err}"))
}

pub fn print_config(config: &ResolvedConfig) {
  println!(
    "THEME_ROOT_DIR={}",
    config.theme_root_dir.to_string_lossy()
  );
  println!(
    "CURRENT_THEME_LINK={}",
    config.current_theme_link.to_string_lossy()
  );
  println!(
    "CURRENT_BACKGROUND_LINK={}",
    config.current_background_link.to_string_lossy()
  );
  println!(
    "OMARCHY_BIN_DIR={}",
    config
      .omarchy_bin_dir
      .as_ref()
      .map(|p| p.to_string_lossy().to_string())
      .unwrap_or_default()
  );
  println!("WAYBAR_DIR={}", config.waybar_dir.to_string_lossy());
  println!(
    "WAYBAR_THEMES_DIR={}",
    config.waybar_themes_dir.to_string_lossy()
  );
  println!("WAYBAR_APPLY_MODE={}", config.waybar_apply_mode);
  println!(
    "WAYBAR_RESTART_CMD={}",
    config.waybar_restart_cmd.as_deref().unwrap_or("")
  );
  println!(
    "WAYBAR_RESTART_LOGS={}",
    if config.waybar_restart_logs { "1" } else { "" }
  );
  println!("WALKER_DIR={}", config.walker_dir.to_string_lossy());
  println!(
    "WALKER_THEMES_DIR={}",
    config.walker_themes_dir.to_string_lossy()
  );
  println!("WALKER_APPLY_MODE={}", config.walker_apply_mode);
  println!(
    "DEFAULT_WALKER_MODE={}",
    config.default_walker_mode.as_deref().unwrap_or("")
  );
  println!(
    "DEFAULT_WALKER_NAME={}",
    config.default_walker_name.as_deref().unwrap_or("")
  );
  println!(
    "STARSHIP_CONFIG={}",
    config.starship_config.to_string_lossy()
  );
  println!(
    "STARSHIP_THEMES_DIR={}",
    config.starship_themes_dir.to_string_lossy()
  );
  println!(
    "DEFAULT_WAYBAR_MODE={}",
    config.default_waybar_mode.as_deref().unwrap_or("")
  );
  println!(
    "DEFAULT_WAYBAR_NAME={}",
    config.default_waybar_name.as_deref().unwrap_or("")
  );
  println!(
    "DEFAULT_STARSHIP_MODE={}",
    config.default_starship_mode.as_deref().unwrap_or("")
  );
  println!(
    "DEFAULT_STARSHIP_PRESET={}",
    config.default_starship_preset.as_deref().unwrap_or("")
  );
  println!(
    "DEFAULT_STARSHIP_NAME={}",
    config.default_starship_name.as_deref().unwrap_or("")
  );
  println!(
    "TUI_APPLY_KEY={}",
    config.tui_apply_key.as_deref().unwrap_or("")
  );
  println!(
    "QUIET_MODE_DEFAULT={}",
    if config.quiet_default { "1" } else { "" }
  );
  println!(
    "QUIET_MODE={}",
    if config.quiet_default { "1" } else { "" }
  );
  println!(
    "AWWW_TRANSITION={}",
    if config.awww_transition { "1" } else { "" }
  );
  println!("AWWW_TRANSITION_TYPE={}", config.awww_transition_type);
  println!(
    "AWWW_TRANSITION_DURATION={}",
    config.awww_transition_duration
  );
  println!("AWWW_TRANSITION_ANGLE={}", config.awww_transition_angle);
  println!("AWWW_TRANSITION_FPS={}", config.awww_transition_fps);
  println!("AWWW_TRANSITION_POS={}", config.awww_transition_pos);
  println!("AWWW_TRANSITION_BEZIER={}", config.awww_transition_bezier);
  println!("AWWW_TRANSITION_WAVE={}", config.awww_transition_wave);
  println!(
    "AWWW_AUTO_START={}",
    if config.awww_auto_start { "1" } else { "" }
  );
}
