use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub fn normalize_theme_name(input: &str) -> String {
  let mut out = String::new();
  let mut in_tag = false;
  for ch in input.chars() {
    match ch {
      '<' => in_tag = true,
      '>' => in_tag = false,
      _ if in_tag => {}
      _ => out.push(ch),
    }
  }
  out = out.trim().to_lowercase().replace(' ', "-");
  out
}

pub fn title_case_theme(name: &str) -> String {
  name
    .split('-')
    .map(|part| {
      let mut chars = part.chars();
      match chars.next() {
        Some(first) => {
          let rest = chars.as_str().to_lowercase();
          format!("{}{}", first.to_ascii_uppercase(), rest)
        }
        None => String::new(),
      }
    })
    .collect::<Vec<_>>()
    .join(" ")
}

pub fn resolve_link_target(link_path: &Path) -> Result<PathBuf> {
  if !link_path.is_symlink() {
    return Ok(link_path.canonicalize()?);
  }
  let target = fs::read_link(link_path)?;
  if target.is_absolute() {
    return Ok(target);
  }
  let parent = link_path
    .parent()
    .ok_or_else(|| anyhow!("failed to resolve link parent"))?;
  Ok(parent.join(target))
}

pub fn current_theme_name(current_link: &Path) -> Result<Option<String>> {
  let link_target_name = if current_link.is_symlink() {
    resolve_link_target(current_link)?
      .file_name()
      .and_then(|n| n.to_str())
      .map(|n| n.to_string())
  } else {
    None
  };

  if let Some(parent) = current_link.parent() {
    let name_path = parent.join("theme.name");
    if name_path.is_file() {
      let name = fs::read_to_string(&name_path)?.trim().to_string();
      if !name.is_empty() {
        if let Some(target_name) = link_target_name.as_deref() {
          if target_name != name {
            return Ok(Some(target_name.to_string()));
          }
        }
        return Ok(Some(name));
      }
    }
  }

  if let Some(target_name) = link_target_name {
    return Ok(Some(target_name));
  }

  if current_link.exists() {
    let name = current_link.file_name().and_then(|n| n.to_str()).map(|n| n.to_string());
    if let Some(name) = &name {
      if name == "theme" {
        return Ok(None);
      }
    }
    return Ok(name);
  }

  Ok(None)
}

pub fn current_theme_dir(current_link: &Path) -> Result<PathBuf> {
  if !current_link.exists() {
    return Err(anyhow!(
      "current theme not set: {}",
      current_link.to_string_lossy()
    ));
  }
  resolve_link_target(current_link)
}

pub fn is_symlink(path: &Path) -> Result<bool> {
  match fs::symlink_metadata(path) {
    Ok(meta) => Ok(meta.file_type().is_symlink()),
    Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
    Err(err) => Err(err.into()),
  }
}
