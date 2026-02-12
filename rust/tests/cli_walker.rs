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
