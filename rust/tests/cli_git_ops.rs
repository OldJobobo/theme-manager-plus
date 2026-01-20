mod support;

use assert_cmd::Command;
use support::*;
use std::fs;

#[test]
fn install_clones_and_sets_theme() {
  let env = setup_env();
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(&themes).unwrap();

  let repo = env.temp.path().join("omarchy-nord-theme");
  fs::create_dir_all(&repo).unwrap();
  Command::new("git")
    .current_dir(&repo)
    .args(["init", "-q"])
    .assert()
    .success();
  fs::write(repo.join("README.md"), "test").unwrap();
  Command::new("git")
    .current_dir(&repo)
    .args(["add", "README.md"])
    .assert()
    .success();
  Command::new("git")
    .current_dir(&repo)
    .args([
      "-c",
      "user.email=test@example.com",
      "-c",
      "user.name=Test",
      "commit",
      "-m",
      "init",
      "-q",
    ])
    .assert()
    .success();

  let mut cmd = cmd_with_env(&env);
  cmd.args(["install", repo.to_string_lossy().as_ref()]);
  cmd.assert().success();

  let installed = themes.join("nord");
  assert!(installed.is_dir());

  let name = fs::read_to_string(omarchy_dir(&env.home).join("current/theme.name")).unwrap();
  assert_eq!(name.trim(), "nord");
}

#[test]
fn update_warns_when_no_git_themes() {
  let env = setup_env();
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  let mut cmd = cmd_with_env(&env);
  cmd.arg("update");
  cmd.assert().success();
}

#[test]
fn remove_deletes_current_and_advances() {
  let env = setup_env();
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("alpha")).unwrap();
  fs::create_dir_all(themes.join("bravo")).unwrap();
  let current = omarchy_dir(&env.home).join("current/theme");
  fs::create_dir_all(current.parent().unwrap()).unwrap();
  #[cfg(unix)]
  std::os::unix::fs::symlink(themes.join("alpha"), &current).unwrap();

  let mut cmd = cmd_with_env(&env);
  cmd.args(["remove", "alpha"]);
  cmd.assert().success();

  assert!(!themes.join("alpha").exists());
  let name = fs::read_to_string(omarchy_dir(&env.home).join("current/theme.name")).unwrap();
  assert_eq!(name.trim(), "bravo");
}

#[test]
fn remove_refuses_only_theme() {
  let env = setup_env();
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("alpha")).unwrap();
  let current = omarchy_dir(&env.home).join("current/theme");
  fs::create_dir_all(current.parent().unwrap()).unwrap();
  #[cfg(unix)]
  std::os::unix::fs::symlink(themes.join("alpha"), &current).unwrap();

  let mut cmd = cmd_with_env(&env);
  cmd.args(["remove", "alpha"]);
  cmd
    .assert()
    .failure()
    .stderr(predicates::str::contains("cannot remove the only theme"));
}

#[test]
fn remove_prompts_for_selection() {
  let env = setup_env();
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("alpha")).unwrap();
  fs::create_dir_all(themes.join("bravo")).unwrap();

  let mut cmd = cmd_with_env(&env);
  cmd.arg("remove");
  cmd.write_stdin("2\n");
  cmd.assert().success();
  assert!(!themes.join("bravo").exists());
}
