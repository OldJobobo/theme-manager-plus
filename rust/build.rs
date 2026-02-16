use std::fs;
use std::path::PathBuf;

fn main() {
  let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
  let version_file = manifest_dir.join("..").join("VERSION");

  println!("cargo:rerun-if-changed={}", version_file.display());
  let version = fs::read_to_string(&version_file)
    .ok()
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty())
    .unwrap_or_else(|| std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string()));

  println!("cargo:rustc-env=THEME_MANAGER_VERSION={version}");
}
