mod support;

use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use support::*;
use std::fs;
use std::path::Path;

fn assert_is_symlink(path: &Path) {
  let meta = fs::symlink_metadata(path).expect("symlink metadata");
  assert!(meta.file_type().is_symlink());
}

#[test]
fn hyprlock_apply_named_updates_config() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  let hyprlock_theme = env.home.join(".config/hypr/themes/hyprlock/shared");
  fs::create_dir_all(&hyprlock_theme).unwrap();
  fs::write(hyprlock_theme.join("hyprlock.conf"), "general { }").unwrap();

  let cfg_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&cfg_dir).unwrap();
  write_toml(
    &cfg_dir.join("config.toml"),
    r#"[hyprlock]
default_mode = "named"
default_name = "shared"
"#,
  );

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a"]);
  cmd.assert().success();

  let applied = env.home.join(".config/omarchy/current/theme/hyprlock.conf");
  assert_is_symlink(&applied);
  let target = fs::read_link(applied).unwrap();
  assert!(target.ends_with("themes/hyprlock/shared/hyprlock.conf"));
}

#[test]
fn hyprlock_apply_auto_uses_theme_hyprlock() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  let hyprlock_theme = themes.join("theme-a/hyprlock-theme");
  fs::create_dir_all(&hyprlock_theme).unwrap();
  fs::write(hyprlock_theme.join("hyprlock.conf"), "theme-auto").unwrap();

  let cfg_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&cfg_dir).unwrap();
  write_toml(
    &cfg_dir.join("config.toml"),
    r#"[hyprlock]
default_mode = "auto"
"#,
  );

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a"]);
  cmd.assert().success();

  let applied = env.home.join(".config/omarchy/current/theme/hyprlock.conf");
  assert_is_symlink(&applied);
  let target = fs::read_link(applied).unwrap();
  assert!(target.ends_with("theme-a/hyprlock-theme/hyprlock.conf"));
}

#[test]
fn hyprlock_none_leaves_existing_config() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  let hypr_dir = env.home.join(".config/hypr");
  fs::create_dir_all(&hypr_dir).unwrap();
  fs::write(hypr_dir.join("hyprlock.conf"), "keep").unwrap();

  let cfg_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&cfg_dir).unwrap();
  write_toml(
    &cfg_dir.join("config.toml"),
    r#"[hyprlock]
default_mode = ""
"#,
  );

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a"]);
  cmd.assert().success();

  let content = fs::read_to_string(hypr_dir.join("hyprlock.conf")).unwrap();
  assert_eq!(content, "keep");
}

#[test]
fn hyprlock_links_omarchy_default_theme_when_missing() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  let omarchy_default = env.home.join(".local/share/omarchy/default/hyprlock");
  fs::create_dir_all(&omarchy_default).unwrap();
  fs::write(omarchy_default.join("hyprlock.conf"), "omarchy").unwrap();

  let cfg_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&cfg_dir).unwrap();
  write_toml(
    &cfg_dir.join("config.toml"),
    r#"[hyprlock]
default_mode = "named"
default_name = "omarchy-default"
"#,
  );

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a"]);
  cmd.assert().success();

  let link_path = env.home.join(".config/hypr/themes/hyprlock/omarchy-default");
  let meta = fs::symlink_metadata(&link_path).unwrap();
  assert!(meta.file_type().is_symlink());
  let target = fs::read_link(&link_path).unwrap();
  assert_eq!(target, omarchy_default);
}

#[test]
fn hyprlock_links_omarchy_default_from_omarchy_root_themes_dir() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  let omarchy_default = env.home.join(".local/share/omarchy/themes/omarchy-default");
  fs::create_dir_all(&omarchy_default).unwrap();
  fs::write(omarchy_default.join("hyprlock.conf"), "omarchy-theme-root").unwrap();

  let cfg_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&cfg_dir).unwrap();
  write_toml(
    &cfg_dir.join("config.toml"),
    r#"[hyprlock]
default_mode = "named"
default_name = "omarchy-default"
"#,
  );

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a"]);
  cmd.assert().success();

  let link_path = env.home.join(".config/hypr/themes/hyprlock/omarchy-default");
  let meta = fs::symlink_metadata(&link_path).unwrap();
  assert!(meta.file_type().is_symlink());
  let target = fs::read_link(&link_path).unwrap();
  assert_eq!(target, omarchy_default);
}

#[test]
fn hyprlock_links_omarchy_default_from_config_omarchy_themes_dir() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  let omarchy_default = env.home.join(".config/omarchy/themes/omarchy-default");
  fs::create_dir_all(&omarchy_default).unwrap();
  fs::write(omarchy_default.join("hyprlock.conf"), "omarchy-theme-config").unwrap();

  let cfg_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&cfg_dir).unwrap();
  write_toml(
    &cfg_dir.join("config.toml"),
    r#"[hyprlock]
default_mode = "named"
default_name = "omarchy-default"
"#,
  );

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a"]);
  cmd.assert().success();

  let link_path = env.home.join(".config/hypr/themes/hyprlock/omarchy-default");
  let meta = fs::symlink_metadata(&link_path).unwrap();
  assert!(meta.file_type().is_symlink());
  let target = fs::read_link(&link_path).unwrap();
  assert_eq!(target, omarchy_default);
}

#[test]
fn hyprlock_links_omarchy_default_from_omarchy_root_config_hypr_dir() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  let omarchy_default = env.home.join(".local/share/omarchy/config/hypr");
  fs::create_dir_all(&omarchy_default).unwrap();
  fs::write(omarchy_default.join("hyprlock.conf"), "omarchy-config-hypr").unwrap();

  let cfg_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&cfg_dir).unwrap();
  write_toml(
    &cfg_dir.join("config.toml"),
    r#"[hyprlock]
default_mode = "named"
default_name = "omarchy-default"
"#,
  );

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a"]);
  cmd.assert().success();

  let link_path = env.home.join(".config/hypr/themes/hyprlock/omarchy-default");
  let meta = fs::symlink_metadata(&link_path).unwrap();
  assert!(meta.file_type().is_symlink());
  let target = fs::read_link(&link_path).unwrap();
  assert_eq!(target, omarchy_default);
}

#[test]
fn hyprlock_prefers_highest_precedence_default_candidate() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  let top_candidate = env
    .home
    .join(".local/share/omarchy/default/hyprlock/themes/omarchy-default");
  fs::create_dir_all(&top_candidate).unwrap();
  fs::write(top_candidate.join("hyprlock.conf"), "top").unwrap();

  let lower_candidate = env.home.join(".local/share/omarchy/default/hyprlock");
  fs::create_dir_all(&lower_candidate).unwrap();
  fs::write(lower_candidate.join("hyprlock.conf"), "lower").unwrap();

  let cfg_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&cfg_dir).unwrap();
  write_toml(
    &cfg_dir.join("config.toml"),
    r#"[hyprlock]
default_mode = "named"
default_name = "omarchy-default"
"#,
  );

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a"]);
  cmd.assert().success();

  let link_path = env.home.join(".config/hypr/themes/hyprlock/omarchy-default");
  let target = fs::read_link(&link_path).unwrap();
  assert_eq!(target, top_candidate);
}

#[test]
fn hyprlock_missing_default_file_does_not_create_default_link() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  let invalid_default = env.home.join(".local/share/omarchy/default/hyprlock");
  fs::create_dir_all(&invalid_default).unwrap();

  let cfg_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&cfg_dir).unwrap();
  write_toml(
    &cfg_dir.join("config.toml"),
    r#"[hyprlock]
default_mode = "named"
default_name = "omarchy-default"
"#,
  );

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a"]);
  cmd.assert().success();

  let link_path = env.home.join(".config/hypr/themes/hyprlock/omarchy-default");
  assert!(!link_path.exists());
}

#[test]
fn hyprlock_standalone_command() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  let current_dir = env.home.join(".config/omarchy/current");
  fs::create_dir_all(&current_dir).unwrap();
  #[cfg(unix)]
  std::os::unix::fs::symlink(themes.join("theme-a"), current_dir.join("theme")).unwrap();
  fs::write(current_dir.join("theme.name"), "theme-a").unwrap();

  let hyprlock_theme = env.home.join(".config/hypr/themes/hyprlock/minimal");
  fs::create_dir_all(&hyprlock_theme).unwrap();
  fs::write(hyprlock_theme.join("hyprlock.conf"), "minimal").unwrap();
  let marker = env.temp.path().join("hyprlock-restart-called");
  write_script(
    &env.bin.join("omarchy-restart-hyprlock"),
    &format!("#!/usr/bin/env bash\n\necho ok > {}\n", marker.display()),
  );

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["hyprlock", "minimal"]);
  cmd.assert().success();

  let applied = env.home.join(".config/omarchy/current/theme/hyprlock.conf");
  assert!(applied.exists());
  assert!(marker.exists());
}

#[test]
fn hyprlock_omarchy_default_uses_active_theme_hyprlock_config() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let theme_dir = omarchy_dir(&env.home).join("themes/theme-a");
  fs::create_dir_all(&theme_dir).unwrap();
  fs::write(theme_dir.join("hyprlock.conf"), "$color = rgba(1,2,3,1.0)\n").unwrap();

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a", "--hyprlock", "omarchy-default"]);
  cmd.assert().success();

  let applied = env.home.join(".config/omarchy/current/theme/hyprlock.conf");
  assert_is_symlink(&applied);
  let target = fs::read_link(applied).unwrap();
  assert!(target.ends_with("themes/theme-a/hyprlock.conf"));
}

#[test]
fn hyprlock_warns_when_main_config_not_sourcing_current_theme() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  let hyprlock_theme = env.home.join(".config/hypr/themes/hyprlock/minimal");
  fs::create_dir_all(&hyprlock_theme).unwrap();
  fs::write(hyprlock_theme.join("hyprlock.conf"), "minimal").unwrap();

  let hypr_dir = env.home.join(".config/hypr");
  fs::create_dir_all(&hypr_dir).unwrap();
  fs::write(hypr_dir.join("hyprlock.conf"), "source = ~/.config/hypr/other.conf\n").unwrap();

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a", "--hyprlock", "minimal"]);
  cmd
    .assert()
    .success()
    .stderr(predicates::str::contains("does not source current theme hyprlock config"));
}

#[test]
fn hyprlock_style_only_theme_restores_wrapper_host_config() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  let omarchy_wrapper = env.home.join(".local/share/omarchy/config/hypr");
  fs::create_dir_all(&omarchy_wrapper).unwrap();
  fs::write(
    omarchy_wrapper.join("hyprlock.conf"),
    "source = ~/.config/omarchy/current/theme/hyprlock.conf\n# WRAPPER_MARKER\n",
  )
  .unwrap();

  let hyprlock_theme = env.home.join(".config/hypr/themes/hyprlock/style-only");
  fs::create_dir_all(&hyprlock_theme).unwrap();
  fs::write(
    hyprlock_theme.join("hyprlock.conf"),
    "$color = rgba(1,2,3,1.0)\n$inner_color = rgba(1,2,3,0.8)\n",
  )
  .unwrap();

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a", "--hyprlock", "style-only"]);
  cmd.assert().success();

  let host = fs::read_to_string(env.home.join(".config/hypr/hyprlock.conf")).unwrap();
  assert!(host.contains("WRAPPER_MARKER"));
}

#[test]
fn hyprlock_full_layout_theme_writes_minimal_host_config() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  let hyprlock_theme = env.home.join(".config/hypr/themes/hyprlock/full-layout");
  fs::create_dir_all(&hyprlock_theme).unwrap();
  fs::write(
    hyprlock_theme.join("hyprlock.conf"),
    "background {\n  monitor =\n  path = /tmp/demo.png\n}\ninput-field {\n  monitor =\n}\n",
  )
  .unwrap();

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a", "--hyprlock", "full-layout"]);
  cmd.assert().success();

  let host = fs::read_to_string(env.home.join(".config/hypr/hyprlock.conf")).unwrap();
  assert!(host.contains("source = ~/.config/omarchy/current/theme/hyprlock.conf"));
  assert!(host.contains("auth {"));
  assert!(!host.contains("input-field {"));
  assert!(!host.contains("path = ~/.config/omarchy/current/background"));
}

#[test]
fn hyprlock_does_not_override_non_managed_host_config() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  let hypr_dir = env.home.join(".config/hypr");
  fs::create_dir_all(&hypr_dir).unwrap();
  fs::write(hypr_dir.join("hyprlock.conf"), "source = ~/.config/hypr/custom.conf\n").unwrap();

  let hyprlock_theme = env.home.join(".config/hypr/themes/hyprlock/full-layout");
  fs::create_dir_all(&hyprlock_theme).unwrap();
  fs::write(
    hyprlock_theme.join("hyprlock.conf"),
    "background {\n  monitor =\n  path = /tmp/demo.png\n}\n",
  )
  .unwrap();

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a", "--hyprlock", "full-layout"]);
  cmd
    .assert()
    .success()
    .stderr(contains("preserving custom").and(contains("does not source current theme hyprlock config")));

  let host = fs::read_to_string(hypr_dir.join("hyprlock.conf")).unwrap();
  assert_eq!(host, "source = ~/.config/hypr/custom.conf\n");
}
