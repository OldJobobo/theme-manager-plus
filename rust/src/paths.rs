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
  let target = fs::read_link(link_path)?;
  if target.is_absolute() {
    return Ok(target);
  }
  let parent = link_path
    .parent()
    .ok_or_else(|| anyhow!("failed to resolve link parent"))?;
  Ok(parent.join(target))
}

pub fn current_theme_dir(current_link: &Path) -> Result<PathBuf> {
  if !is_symlink(current_link)? {
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
