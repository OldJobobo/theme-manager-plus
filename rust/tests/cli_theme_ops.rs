mod support;

use support::*;
use std::fs;

#[test]
fn list_titles() {
  let env = setup_env();
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("tokyo-night")).unwrap();
  fs::create_dir_all(themes.join("gruvbox")).unwrap();

  let mut cmd = cmd_with_env(&env);
  cmd.arg("list");
  cmd.assert().success().stdout(predicates::str::contains("Tokyo Night"));
  cmd.assert().success().stdout(predicates::str::contains("Gruvbox"));
}

#[test]
fn set_updates_symlink() {
  let env = setup_env();
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("tokyo-night")).unwrap();

  let mut cmd = cmd_with_env(&env);
  cmd.args(["set", "Tokyo Night"]);
  cmd.assert().success();

  let link = omarchy_dir(&env.home).join("current/theme");
  let target = fs::read_link(&link).expect("symlink");
  assert!(target.to_string_lossy().contains("tokyo-night"));
}

#[test]
fn current_errors_when_missing() {
  let env = setup_env();
  let mut cmd = cmd_with_env(&env);
  cmd.arg("current");
  cmd
    .assert()
    .failure()
    .stderr(predicates::str::contains("current theme not set"));
}

#[test]
fn next_cycles() {
  let env = setup_env();
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("alpha")).unwrap();
  fs::create_dir_all(themes.join("bravo")).unwrap();
  let current = omarchy_dir(&env.home).join("current/theme");
  fs::create_dir_all(current.parent().unwrap()).unwrap();
  #[cfg(unix)]
  std::os::unix::fs::symlink(themes.join("alpha"), &current).unwrap();

  let mut cmd = cmd_with_env(&env);
  cmd.arg("next");
  cmd.assert().success();

  let target = fs::read_link(&current).expect("symlink");
  assert!(target.to_string_lossy().contains("bravo"));
}

#[test]
fn bg_next_runs_command() {
  let env = setup_env();
  let marker = env.temp.path().join("bg-next-called");
  let script = env.bin.join("omarchy-theme-bg-next");
  write_script(
    &script,
    &format!(
      "#!/usr/bin/env bash\n\necho ok > {}\n",
      marker.display()
    ),
  );

  let mut cmd = cmd_with_env(&env);
  cmd.arg("bg-next");
  cmd.assert().success();
  assert!(marker.exists());
}

#[test]
fn set_rejects_broken_symlink() {
  let env = setup_env();
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(&themes).unwrap();

  let broken = themes.join("broken");
  #[cfg(unix)]
  std::os::unix::fs::symlink(themes.join("missing-target"), &broken).unwrap();

  let mut cmd = cmd_with_env(&env);
  cmd.args(["set", "broken"]);
  cmd
    .assert()
    .failure()
    .stderr(predicates::str::contains("theme symlink is broken"));
}

#[test]
fn set_rejects_empty_waybar_name() {
  let env = setup_env();
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  let mut cmd = cmd_with_env(&env);
  cmd.args(["set", "theme-a", "--waybar="]);
  cmd
    .assert()
    .failure()
    .stderr(predicates::str::contains("--waybar requires a name"));
}
