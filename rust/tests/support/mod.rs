#![allow(dead_code)]

use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub struct TestEnv {
  pub temp: TempDir,
  pub home: PathBuf,
  pub bin: PathBuf,
}

pub fn setup_env() -> TestEnv {
  let temp = TempDir::new().expect("temp dir");
  let home = temp.path().join("home");
  fs::create_dir_all(&home).expect("home dir");
  let bin = temp.path().join("bin");
  fs::create_dir_all(&bin).expect("bin dir");
  // Safety guard: never let tests spawn the user's live Waybar session.
  write_stub_ok(&bin.join("waybar"));
  write_stub_ok(&bin.join("uwsm-app"));
  // Safety guard: never emit desktop notifications or wallpaper transitions.
  write_stub_ok(&bin.join("notify-send"));
  write_stub_ok(&bin.join("awww"));
  write_stub_ok(&bin.join("awww-daemon"));
  TestEnv { temp, home, bin }
}

pub fn cmd_with_env(env: &TestEnv) -> Command {
  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("theme-manager"));
  cmd.env("HOME", &env.home);
  cmd.env("THEME_MANAGER_SKIP_APPS", "1");
  cmd.env("THEME_MANAGER_SKIP_HOOK", "1");
  cmd.env("THEME_MANAGER_AWWW_TRANSITION", "0");
  // Prevent host Omarchy env leakage from prepending real command paths.
  cmd.env_remove("OMARCHY_PATH");
  cmd.env_remove("OMARCHY_BIN_DIR");
  cmd.env("PATH", format!("{}:/usr/bin:/bin", env.bin.display()));
  cmd
}

pub fn omarchy_dir(home: &Path) -> PathBuf {
  home.join(".config/omarchy")
}

pub fn write_script(path: &Path, content: &str) {
  fs::write(path, content).expect("write script");
  let mut perms = fs::metadata(path).expect("metadata").permissions();
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).expect("chmod");
  }
}

pub fn write_stub_ok(path: &Path) {
  write_script(path, "#!/usr/bin/env bash\n\nexit 0\n");
}

pub fn add_omarchy_stubs(bin: &Path) {
  let cmds = [
    "omarchy-restart-waybar",
    "omarchy-restart-walker",
    "omarchy-restart-hyprlock",
    "omarchy-restart-terminal",
    "omarchy-restart-swayosd",
    "omarchy-theme-bg-next",
    "omarchy-theme-set-gnome",
    "omarchy-theme-set-browser",
    "omarchy-theme-set-vscode",
    "omarchy-theme-set-cursor",
    "omarchy-theme-set-obsidian",
    "hyprctl",
    "makoctl",
    "pkill",
  ];
  for cmd in cmds {
    write_stub_ok(&bin.join(cmd));
  }
}

pub fn write_toml(path: &Path, content: &str) {
  fs::write(path, content).expect("write toml");
}
