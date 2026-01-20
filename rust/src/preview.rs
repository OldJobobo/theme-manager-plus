use std::fs;
use std::path::{Path, PathBuf};

pub fn find_theme_preview(theme_dir: &Path) -> Option<PathBuf> {
  find_named_file(theme_dir, "preview.png")
    .or_else(|| find_named_file(theme_dir, "theme.png"))
    .or_else(|| find_named_file(&theme_dir.join("waybar-theme"), "preview.png"))
    .or_else(|| find_first_image(&theme_dir.join("backgrounds")))
}

pub fn find_waybar_preview(waybar_dir: &Path) -> Option<PathBuf> {
  find_first_png(waybar_dir)
}

fn find_named_file(dir: &Path, name: &str) -> Option<PathBuf> {
  if !dir.is_dir() {
    return None;
  }
  let name_lower = name.to_lowercase();
  for entry in fs::read_dir(dir).ok()? {
    let entry = entry.ok()?;
    let path = entry.path();
    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
      if file_name.to_lowercase() == name_lower {
        if path.is_file() {
          return Some(path);
        }
      }
    }
  }
  None
}

fn find_first_png(dir: &Path) -> Option<PathBuf> {
  find_first_by_exts(dir, &["png"])
}

fn find_first_image(dir: &Path) -> Option<PathBuf> {
  find_first_by_exts(dir, &["png", "jpg", "jpeg", "webp"])
}

fn find_first_by_exts(dir: &Path, exts: &[&str]) -> Option<PathBuf> {
  if !dir.is_dir() {
    return None;
  }
  let mut files: Vec<PathBuf> = fs::read_dir(dir)
    .ok()?
    .filter_map(|entry| entry.ok().map(|e| e.path()))
    .filter(|path| {
      path.is_file()
        && path
          .extension()
          .and_then(|ext| ext.to_str())
          .map(|ext| exts.iter().any(|wanted| ext.eq_ignore_ascii_case(wanted)))
          .unwrap_or(false)
    })
    .collect();
  files.sort();
  files.into_iter().next()
}
