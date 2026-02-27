use anyhow::Result;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::config::ResolvedConfig;
use crate::omarchy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultModule {
  Waybar,
  Walker,
  Hyprlock,
  Starship,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultSourceKind {
  OmarchyDefaultNamed,
  OmarchyDefaultBase,
  OmarchyThemeStoreDefault,
  OmarchyConfigFallback,
  OmarchyUserConfigFallback,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedOmarchyDefault {
  pub module: DefaultModule,
  pub path: PathBuf,
  pub kind: DefaultSourceKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymlinkEnsureResult {
  Created,
  Updated,
  Unchanged,
  SkippedNonSymlink,
}

pub fn resolve_waybar_default(config: &ResolvedConfig) -> Option<ResolvedOmarchyDefault> {
  let root = omarchy::detect_omarchy_root(config)?;

  let named = root.join("default/waybar/themes/omarchy-default");
  if is_waybar_theme_dir(&named) {
    return Some(ResolvedOmarchyDefault {
      module: DefaultModule::Waybar,
      path: named,
      kind: DefaultSourceKind::OmarchyDefaultNamed,
    });
  }

  let base = root.join("default/waybar");
  if is_waybar_theme_dir(&base) {
    return Some(ResolvedOmarchyDefault {
      module: DefaultModule::Waybar,
      path: base,
      kind: DefaultSourceKind::OmarchyDefaultBase,
    });
  }

  let config_fallback = root.join("config/waybar");
  if is_waybar_theme_dir(&config_fallback) {
    return Some(ResolvedOmarchyDefault {
      module: DefaultModule::Waybar,
      path: config_fallback,
      kind: DefaultSourceKind::OmarchyConfigFallback,
    });
  }

  None
}

pub fn resolve_walker_default(config: &ResolvedConfig) -> Option<ResolvedOmarchyDefault> {
  let root = omarchy::detect_omarchy_root(config)?;

  let named = root.join("default/walker/themes/omarchy-default");
  if is_walker_theme_dir(&named) {
    return Some(ResolvedOmarchyDefault {
      module: DefaultModule::Walker,
      path: named,
      kind: DefaultSourceKind::OmarchyDefaultNamed,
    });
  }

  let base = root.join("default/walker");
  if is_walker_theme_dir(&base) {
    return Some(ResolvedOmarchyDefault {
      module: DefaultModule::Walker,
      path: base,
      kind: DefaultSourceKind::OmarchyDefaultBase,
    });
  }

  None
}

pub fn resolve_hyprlock_default(config: &ResolvedConfig) -> Option<ResolvedOmarchyDefault> {
  let mut candidates: Vec<(PathBuf, DefaultSourceKind)> = Vec::new();

  if let Some(root) = omarchy::detect_omarchy_root(config) {
    candidates.push((
      root.join("default/hyprlock/themes/omarchy-default"),
      DefaultSourceKind::OmarchyDefaultNamed,
    ));
    candidates.push((
      root.join("default/hyprlock"),
      DefaultSourceKind::OmarchyDefaultBase,
    ));
    candidates.push((
      root.join("themes/omarchy-default"),
      DefaultSourceKind::OmarchyThemeStoreDefault,
    ));
    candidates.push((
      root.join("config/hypr"),
      DefaultSourceKind::OmarchyConfigFallback,
    ));
  }

  if let Ok(home) = std::env::var("HOME") {
    let home = PathBuf::from(home);
    candidates.push((
      home.join(".config/omarchy/default/hyprlock/themes/omarchy-default"),
      DefaultSourceKind::OmarchyUserConfigFallback,
    ));
    candidates.push((
      home.join(".config/omarchy/default/hyprlock"),
      DefaultSourceKind::OmarchyUserConfigFallback,
    ));
    candidates.push((
      home.join(".config/omarchy/themes/omarchy-default"),
      DefaultSourceKind::OmarchyUserConfigFallback,
    ));
    candidates.push((
      home.join(".config/omarchy/config/hypr"),
      DefaultSourceKind::OmarchyUserConfigFallback,
    ));
  }

  for (path, kind) in candidates {
    if path.join("hyprlock.conf").is_file() {
      return Some(ResolvedOmarchyDefault {
        module: DefaultModule::Hyprlock,
        path,
        kind,
      });
    }
  }

  None
}

pub fn resolve_starship_default(config: &ResolvedConfig) -> Option<ResolvedOmarchyDefault> {
  let root = omarchy::detect_omarchy_root(config)?;

  let named = root.join("default/starship/themes/omarchy-default.toml");
  if named.is_file() {
    return Some(ResolvedOmarchyDefault {
      module: DefaultModule::Starship,
      path: named,
      kind: DefaultSourceKind::OmarchyDefaultNamed,
    });
  }

  let base = root.join("default/starship.toml");
  if base.is_file() {
    return Some(ResolvedOmarchyDefault {
      module: DefaultModule::Starship,
      path: base,
      kind: DefaultSourceKind::OmarchyDefaultBase,
    });
  }

  let nested = root.join("default/starship/starship.toml");
  if nested.is_file() {
    return Some(ResolvedOmarchyDefault {
      module: DefaultModule::Starship,
      path: nested,
      kind: DefaultSourceKind::OmarchyConfigFallback,
    });
  }

  let config_fallback = root.join("config/starship.toml");
  if config_fallback.is_file() {
    return Some(ResolvedOmarchyDefault {
      module: DefaultModule::Starship,
      path: config_fallback,
      kind: DefaultSourceKind::OmarchyConfigFallback,
    });
  }

  None
}

pub fn ensure_symlink(link_path: &Path, target: &Path) -> Result<SymlinkEnsureResult> {
  match fs::symlink_metadata(link_path) {
    Ok(meta) => {
      if !meta.file_type().is_symlink() {
        return Ok(SymlinkEnsureResult::SkippedNonSymlink);
      }

      let current_target = fs::read_link(link_path)?;
      if current_target == target {
        return Ok(SymlinkEnsureResult::Unchanged);
      }

      fs::remove_file(link_path)?;
      #[cfg(unix)]
      {
        std::os::unix::fs::symlink(target, link_path)?;
      }
      #[cfg(not(unix))]
      {
        fs::copy(target, link_path)?;
      }
      Ok(SymlinkEnsureResult::Updated)
    }
    Err(err) if err.kind() == ErrorKind::NotFound => {
      if let Some(parent) = link_path.parent() {
        fs::create_dir_all(parent)?;
      }
      #[cfg(unix)]
      {
        std::os::unix::fs::symlink(target, link_path)?;
      }
      #[cfg(not(unix))]
      {
        fs::copy(target, link_path)?;
      }
      Ok(SymlinkEnsureResult::Created)
    }
    Err(err) => Err(err.into()),
  }
}

fn is_waybar_theme_dir(path: &Path) -> bool {
  path.join("config.jsonc").is_file() && path.join("style.css").is_file()
}

fn is_walker_theme_dir(path: &Path) -> bool {
  path.join("style.css").is_file()
}
