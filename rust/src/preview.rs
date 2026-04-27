use std::fs;
use std::path::{Path, PathBuf};

pub fn find_theme_preview(theme_dir: &Path) -> Option<PathBuf> {
    find_named_image(theme_dir, "preview")
        .or_else(|| find_named_image(theme_dir, "theme"))
        .or_else(|| find_named_file(&theme_dir.join("waybar-theme"), "preview.png"))
        .or_else(|| find_first_image(&theme_dir.join("backgrounds")))
}

pub fn find_waybar_preview(waybar_dir: &Path) -> Option<PathBuf> {
    find_named_image(waybar_dir, "preview").or_else(|| find_first_image(waybar_dir))
}

pub fn find_walker_preview(walker_dir: &Path) -> Option<PathBuf> {
    find_named_image(walker_dir, "preview").or_else(|| find_first_image(walker_dir))
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

fn find_first_image(dir: &Path) -> Option<PathBuf> {
    find_first_by_exts(dir, &["png", "jpg", "jpeg", "webp"])
}

fn find_named_image(dir: &Path, stem: &str) -> Option<PathBuf> {
    if !dir.is_dir() {
        return None;
    }
    let stem_lower = stem.to_lowercase();
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .ok()?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            path.is_file()
                && path
                    .file_stem()
                    .and_then(|name| name.to_str())
                    .map(|name| name.to_lowercase() == stem_lower)
                    .unwrap_or(false)
                && path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| {
                        ["png", "jpg", "jpeg", "webp"]
                            .iter()
                            .any(|wanted| ext.eq_ignore_ascii_case(wanted))
                    })
                    .unwrap_or(false)
        })
        .collect();
    files.sort();
    files.into_iter().next()
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn theme_preview_accepts_common_image_extensions() {
        let temp = TempDir::new().unwrap();
        let theme_dir = temp.path().join("theme");
        fs::create_dir_all(&theme_dir).unwrap();
        let preview = theme_dir.join("preview.webp");
        fs::write(&preview, b"test").unwrap();

        assert_eq!(find_theme_preview(&theme_dir), Some(preview));
    }

    #[test]
    fn walker_preview_prefers_named_image_before_fallback() {
        let temp = TempDir::new().unwrap();
        let walker_dir = temp.path().join("walker-theme");
        fs::create_dir_all(&walker_dir).unwrap();
        let fallback = walker_dir.join("aaa.png");
        let preferred = walker_dir.join("preview.jpg");
        fs::write(&fallback, b"test").unwrap();
        fs::write(&preferred, b"test").unwrap();

        assert_eq!(find_walker_preview(&walker_dir), Some(preferred));
    }
}
