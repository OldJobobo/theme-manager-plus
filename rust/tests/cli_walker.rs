mod support;

use support::*;
use std::fs;

#[test]
fn walker_apply_named_updates_config() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  // Create a walker theme
  let walker_theme = env.home.join(".config/walker/themes/shared");
  fs::create_dir_all(&walker_theme).unwrap();
  fs::write(walker_theme.join("style.css"), "style").unwrap();

  // Create walker config
  let walker_dir = env.home.join(".config/walker");
  fs::create_dir_all(&walker_dir).unwrap();
  fs::write(walker_dir.join("config.toml"), "theme = \"old\"\n").unwrap();

  let cfg_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&cfg_dir).unwrap();
  write_toml(
    &cfg_dir.join("config.toml"),
    r#"[walker]
default_mode = "named"
default_name = "shared"
"#,
  );

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a"]);
  cmd.assert().success();

  // Verify the walker config was updated
  let config_content = fs::read_to_string(walker_dir.join("config.toml")).unwrap();
  assert!(config_content.contains("theme = \"shared\""));
}

#[test]
fn walker_named_updates_only_theme_key() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  let walker_theme = env.home.join(".config/walker/themes/shared");
  fs::create_dir_all(&walker_theme).unwrap();
  fs::write(walker_theme.join("style.css"), "style").unwrap();

  let walker_dir = env.home.join(".config/walker");
  fs::create_dir_all(&walker_dir).unwrap();
  fs::write(
    walker_dir.join("config.toml"),
    "theme_name = \"keep\"\ntheme_variant = \"keep\"\ntheme = \"old\"\n",
  )
  .unwrap();

  let cfg_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&cfg_dir).unwrap();
  write_toml(
    &cfg_dir.join("config.toml"),
    r#"[walker]
default_mode = "named"
default_name = "shared"
"#,
  );

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a"]);
  cmd.assert().success();

  let config_content = fs::read_to_string(walker_dir.join("config.toml")).unwrap();
  assert!(config_content.contains("theme = \"shared\""));
  assert!(config_content.contains("theme_name = \"keep\""));
  assert!(config_content.contains("theme_variant = \"keep\""));
}

#[test]
fn walker_apply_auto_creates_theme_dir() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  let theme_dir = themes.join("theme-a/walker-theme");
  fs::create_dir_all(&theme_dir).unwrap();
  fs::write(theme_dir.join("style.css"), "style").unwrap();
  fs::write(theme_dir.join("layout.xml"), "<layout/>").unwrap();

  // Create walker config
  let walker_dir = env.home.join(".config/walker");
  fs::create_dir_all(&walker_dir).unwrap();
  fs::write(walker_dir.join("config.toml"), "theme = \"old\"\n").unwrap();

  // Create walker themes dir
  let walker_themes = walker_dir.join("themes");
  fs::create_dir_all(&walker_themes).unwrap();

  let cfg_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&cfg_dir).unwrap();
  write_toml(
    &cfg_dir.join("config.toml"),
    r#"[walker]
apply_mode = "symlink"
default_mode = "auto"
"#,
  );

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a"]);
  cmd.assert().success();

  // Verify the auto theme was created
  let auto_theme = walker_themes.join("theme-manager-auto");
  assert!(auto_theme.is_dir());
  assert!(auto_theme.join("style.css").exists());

  // Verify config was updated
  let config_content = fs::read_to_string(walker_dir.join("config.toml")).unwrap();
  assert!(config_content.contains("theme = \"theme-manager-auto\""));
}

#[test]
fn walker_auto_cleans_stale_files() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  let theme_dir = themes.join("theme-a/walker-theme");
  fs::create_dir_all(&theme_dir).unwrap();
  fs::write(theme_dir.join("style.css"), "new-style").unwrap();
  fs::write(theme_dir.join("layout.xml"), "<layout/>").unwrap();

  let walker_dir = env.home.join(".config/walker");
  fs::create_dir_all(&walker_dir).unwrap();
  fs::write(walker_dir.join("config.toml"), "theme = \"old\"\n").unwrap();

  let walker_themes = walker_dir.join("themes");
  let auto_theme = walker_themes.join("theme-manager-auto");
  fs::create_dir_all(&auto_theme).unwrap();
  fs::write(auto_theme.join("style.css"), "old-style").unwrap();
  fs::write(auto_theme.join("stale.txt"), "stale").unwrap();

  let cfg_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&cfg_dir).unwrap();
  write_toml(
    &cfg_dir.join("config.toml"),
    r#"[walker]
apply_mode = "copy"
default_mode = "auto"
"#,
  );

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a"]);
  cmd.assert().success();

  assert!(auto_theme.is_dir());
  assert!(auto_theme.join("style.css").exists());
  assert_eq!(
    fs::read_to_string(auto_theme.join("style.css")).unwrap(),
    "new-style"
  );
  assert!(!auto_theme.join("stale.txt").exists());
}

#[test]
fn walker_standalone_command() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  // Set up current theme link
  let current_dir = env.home.join(".config/omarchy/current");
  fs::create_dir_all(&current_dir).unwrap();
  #[cfg(unix)]
  std::os::unix::fs::symlink(themes.join("theme-a"), current_dir.join("theme")).unwrap();
  fs::write(current_dir.join("theme.name"), "theme-a").unwrap();

  // Create a walker theme
  let walker_themes = env.home.join(".config/walker/themes");
  let walker_theme = walker_themes.join("minimal");
  fs::create_dir_all(&walker_theme).unwrap();
  fs::write(walker_theme.join("style.css"), "minimal-style").unwrap();

  // Create walker config
  let walker_dir = env.home.join(".config/walker");
  fs::write(walker_dir.join("config.toml"), "theme = \"old\"\n").unwrap();
  let marker = env.temp.path().join("walker-restart-called");
  write_script(
    &env.bin.join("omarchy-restart-walker"),
    &format!("#!/usr/bin/env bash\n\necho ok > {}\n", marker.display()),
  );

  let cfg_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&cfg_dir).unwrap();
  write_toml(&cfg_dir.join("config.toml"), "");

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["walker", "minimal"]);
  cmd.assert().success();

  // Verify config was updated
  let config_content = fs::read_to_string(walker_dir.join("config.toml")).unwrap();
  assert!(config_content.contains("theme = \"minimal\""));
  assert!(marker.exists());
}

#[test]
fn walker_none_skips_theme() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  // Create walker config with existing theme
  let walker_dir = env.home.join(".config/walker");
  fs::create_dir_all(&walker_dir).unwrap();
  fs::write(walker_dir.join("config.toml"), "theme = \"original\"\n").unwrap();

  let cfg_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&cfg_dir).unwrap();
  write_toml(
    &cfg_dir.join("config.toml"),
    r#"[walker]
default_mode = ""
"#,
  );

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a"]);
  cmd.assert().success();

  // Verify config was NOT changed
  let config_content = fs::read_to_string(walker_dir.join("config.toml")).unwrap();
  assert!(config_content.contains("theme = \"original\""));
}

#[test]
fn set_walker_flag_overrides_defaults() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  let walker_dir = env.home.join(".config/walker");
  fs::create_dir_all(&walker_dir).unwrap();
  fs::write(walker_dir.join("config.toml"), "theme = \"old\"\n").unwrap();

  let walker_theme = walker_dir.join("themes/shared");
  fs::create_dir_all(&walker_theme).unwrap();
  fs::write(walker_theme.join("style.css"), "style").unwrap();

  let cfg_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&cfg_dir).unwrap();
  write_toml(
    &cfg_dir.join("config.toml"),
    r#"[walker]
default_mode = "none"
"#,
  );

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a", "--walker", "shared"]);
  cmd.assert().success();

  let config_content = fs::read_to_string(walker_dir.join("config.toml")).unwrap();
  assert!(config_content.contains("theme = \"shared\""));
}

#[test]
fn next_walker_auto_flag_uses_theme_walker() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("alpha/walker-theme")).unwrap();
  fs::create_dir_all(themes.join("bravo/walker-theme")).unwrap();
  fs::write(themes.join("bravo/walker-theme/style.css"), "style").unwrap();

  let current_dir = omarchy_dir(&env.home).join("current");
  fs::create_dir_all(current_dir.join("theme")).unwrap();
  fs::write(current_dir.join("theme.name"), "alpha").unwrap();

  let walker_dir = env.home.join(".config/walker");
  fs::create_dir_all(walker_dir.join("themes")).unwrap();
  fs::write(walker_dir.join("config.toml"), "theme = \"old\"\n").unwrap();

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["next", "--walker"]);
  cmd.assert().success();

  let content = fs::read_to_string(walker_dir.join("config.toml")).unwrap();
  assert!(content.contains("theme = \"theme-manager-auto\""));
}

#[test]
fn walker_links_omarchy_default_theme_when_missing() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  let omarchy_default = env
    .home
    .join(".local/share/omarchy/default/walker/themes/omarchy-default");
  fs::create_dir_all(&omarchy_default).unwrap();
  fs::write(omarchy_default.join("style.css"), "default-style").unwrap();

  let walker_dir = env.home.join(".config/walker");
  fs::create_dir_all(&walker_dir).unwrap();
  fs::write(walker_dir.join("config.toml"), "theme = \"old\"\n").unwrap();

  let cfg_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&cfg_dir).unwrap();
  write_toml(
    &cfg_dir.join("config.toml"),
    r#"[walker]
default_mode = "named"
default_name = "omarchy-default"
"#,
  );

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a"]);
  cmd.assert().success();

  let link_path = walker_dir.join("themes/omarchy-default");
  let meta = fs::symlink_metadata(&link_path).unwrap();
  assert!(meta.file_type().is_symlink());
  let target = fs::read_link(&link_path).unwrap();
  assert_eq!(target, omarchy_default);

  let config_content = fs::read_to_string(walker_dir.join("config.toml")).unwrap();
  assert!(config_content.contains("theme = \"omarchy-default\""));
}

#[test]
fn walker_links_omarchy_default_from_base_default_walker_dir() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  let omarchy_default = env.home.join(".local/share/omarchy/default/walker");
  fs::create_dir_all(&omarchy_default).unwrap();
  fs::write(omarchy_default.join("style.css"), "default-style").unwrap();

  let walker_dir = env.home.join(".config/walker");
  fs::create_dir_all(&walker_dir).unwrap();
  fs::write(walker_dir.join("config.toml"), "theme = \"old\"\n").unwrap();

  let cfg_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&cfg_dir).unwrap();
  write_toml(
    &cfg_dir.join("config.toml"),
    r#"[walker]
default_mode = "named"
default_name = "omarchy-default"
"#,
  );

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a"]);
  cmd.assert().success();

  let link_path = walker_dir.join("themes/omarchy-default");
  let meta = fs::symlink_metadata(&link_path).unwrap();
  assert!(meta.file_type().is_symlink());
  let target = fs::read_link(&link_path).unwrap();
  assert_eq!(target, omarchy_default);
}

#[test]
fn walker_prefers_named_default_over_base_default() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  let base_default = env.home.join(".local/share/omarchy/default/walker");
  fs::create_dir_all(&base_default).unwrap();
  fs::write(base_default.join("style.css"), "base-style").unwrap();

  let named_default = env
    .home
    .join(".local/share/omarchy/default/walker/themes/omarchy-default");
  fs::create_dir_all(&named_default).unwrap();
  fs::write(named_default.join("style.css"), "named-style").unwrap();

  let walker_dir = env.home.join(".config/walker");
  fs::create_dir_all(&walker_dir).unwrap();
  fs::write(walker_dir.join("config.toml"), "theme = \"old\"\n").unwrap();

  let cfg_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&cfg_dir).unwrap();
  write_toml(
    &cfg_dir.join("config.toml"),
    r#"[walker]
default_mode = "named"
default_name = "omarchy-default"
"#,
  );

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a"]);
  cmd.assert().success();

  let link_path = walker_dir.join("themes/omarchy-default");
  let target = fs::read_link(&link_path).unwrap();
  assert_eq!(target, named_default);
}

#[test]
fn walker_missing_default_style_does_not_create_default_link() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  let invalid_base = env.home.join(".local/share/omarchy/default/walker");
  fs::create_dir_all(&invalid_base).unwrap();
  fs::write(invalid_base.join("layout.xml"), "<layout/>").unwrap();

  let walker_dir = env.home.join(".config/walker");
  fs::create_dir_all(&walker_dir).unwrap();
  fs::write(walker_dir.join("config.toml"), "theme = \"old\"\n").unwrap();

  let cfg_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&cfg_dir).unwrap();
  write_toml(
    &cfg_dir.join("config.toml"),
    r#"[walker]
default_mode = "named"
default_name = "omarchy-default"
"#,
  );

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a"]);
  cmd.assert().success();

  let link_path = walker_dir.join("themes/omarchy-default");
  assert!(!link_path.exists());
}
