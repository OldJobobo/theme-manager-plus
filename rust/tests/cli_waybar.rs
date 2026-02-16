mod support;

use support::*;
use std::fs;
use std::path::Path;

fn assert_is_symlink(path: &Path) {
  let meta = fs::symlink_metadata(path).expect("symlink metadata");
  assert!(meta.file_type().is_symlink());
}

#[test]
fn waybar_apply_symlink_named() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  let waybar_theme = env.home.join(".config/waybar/themes/shared");
  fs::create_dir_all(&waybar_theme).unwrap();
  fs::write(waybar_theme.join("config.jsonc"), "cfg").unwrap();
  fs::write(waybar_theme.join("style.css"), "style").unwrap();

  let cfg_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&cfg_dir).unwrap();
  write_toml(
    &cfg_dir.join("config.toml"),
    r#"[waybar]
apply_mode = "symlink"
"#,
  );

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a", "-w", "shared"]);
  cmd.assert().success();

  let applied = env.home.join(".config/waybar/config.jsonc");
  assert_is_symlink(&applied);
  let target = fs::read_link(applied).unwrap();
  assert!(target.ends_with("themes/shared/config.jsonc"));
}

#[test]
fn waybar_apply_copy_mode() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  let theme_dir = themes.join("theme-a/waybar-theme");
  fs::create_dir_all(&theme_dir).unwrap();
  fs::write(theme_dir.join("config.jsonc"), "cfg").unwrap();
  fs::write(theme_dir.join("style.css"), "style").unwrap();

  let cfg_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&cfg_dir).unwrap();
  write_toml(
    &cfg_dir.join("config.toml"),
    r#"[waybar]
apply_mode = "copy"
default_mode = "auto"
"#,
  );

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a", "-w"]);
  cmd.assert().success();

  let applied_config = env.home.join(".config/waybar/config.jsonc");
  let applied_style = env.home.join(".config/waybar/style.css");
  assert!(applied_config.exists());
  assert!(applied_style.exists());
  assert!(!fs::symlink_metadata(&applied_config)
    .unwrap()
    .file_type()
    .is_symlink());
  assert!(!fs::symlink_metadata(&applied_style)
    .unwrap()
    .file_type()
    .is_symlink());
}

#[test]
fn waybar_symlink_links_subdirs_and_cleans_up_on_switch() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  let waybar_root = env.home.join(".config/waybar/themes");
  let shared = waybar_root.join("shared");
  fs::create_dir_all(shared.join("assets")).unwrap();
  fs::create_dir_all(shared.join("scripts")).unwrap();
  fs::write(shared.join("config.jsonc"), "cfg").unwrap();
  fs::write(shared.join("style.css"), "style").unwrap();

  let alt = waybar_root.join("alt");
  fs::create_dir_all(alt.join("scripts")).unwrap();
  fs::create_dir_all(alt.join("fonts")).unwrap();
  fs::write(alt.join("config.jsonc"), "cfg2").unwrap();
  fs::write(alt.join("style.css"), "style2").unwrap();

  let cfg_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&cfg_dir).unwrap();
  write_toml(
    &cfg_dir.join("config.toml"),
    r#"[waybar]
apply_mode = "symlink"
"#,
  );

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a", "-w", "shared"]);
  cmd.assert().success();

  let waybar_dir = env.home.join(".config/waybar");
  let assets_link = waybar_dir.join("assets");
  let scripts_link = waybar_dir.join("scripts");
  let fonts_link = waybar_dir.join("fonts");
  let config_link = waybar_dir.join("config.jsonc");
  let style_link = waybar_dir.join("style.css");
  assert_is_symlink(&assets_link);
  assert_is_symlink(&scripts_link);
  assert_is_symlink(&config_link);
  assert_is_symlink(&style_link);

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a", "-w", "alt"]);
  cmd.assert().success();

  assert!(!assets_link.exists());
  assert_is_symlink(&scripts_link);
  assert_is_symlink(&fonts_link);
  let target = fs::read_link(&scripts_link).unwrap();
  assert!(target.ends_with("themes/alt/scripts"));
}

#[test]
fn waybar_symlink_backs_up_existing_non_symlinks() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  let waybar_theme = env.home.join(".config/waybar/themes/shared");
  fs::create_dir_all(waybar_theme.join("assets")).unwrap();
  fs::write(waybar_theme.join("config.jsonc"), "cfg").unwrap();
  fs::write(waybar_theme.join("style.css"), "style").unwrap();

  let waybar_dir = env.home.join(".config/waybar");
  fs::create_dir_all(&waybar_dir).unwrap();
  fs::write(waybar_dir.join("config.jsonc"), "old").unwrap();
  fs::write(waybar_dir.join("style.css"), "old-style").unwrap();
  fs::create_dir_all(waybar_dir.join("assets")).unwrap();

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a", "-w", "shared"]);
  cmd.assert().success();

  let backup_root = env.home.join(".config/waybar/themes/existing");
  assert!(backup_root.is_dir());
  assert!(backup_root.join("config.jsonc").is_file());
  assert!(backup_root.join("style.css").is_file());
  assert!(backup_root.join("assets").is_dir());
}

#[test]
fn waybar_links_omarchy_default_theme_when_missing() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  let omarchy_default = env.home.join(".local/share/omarchy/default/waybar");
  fs::create_dir_all(&omarchy_default).unwrap();
  fs::write(omarchy_default.join("config.jsonc"), "omarchy-cfg").unwrap();
  fs::write(omarchy_default.join("style.css"), "omarchy-style").unwrap();

  let cfg_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&cfg_dir).unwrap();
  write_toml(
    &cfg_dir.join("config.toml"),
    r#"[waybar]
apply_mode = "symlink"
default_mode = "named"
default_name = "omarchy-default"
"#,
  );

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a"]);
  cmd.assert().success();

  let link_path = env.home.join(".config/waybar/themes/omarchy-default");
  let meta = fs::symlink_metadata(&link_path).unwrap();
  assert!(meta.file_type().is_symlink());
  let target = fs::read_link(&link_path).unwrap();
  assert_eq!(target, omarchy_default);

  let applied = env.home.join(".config/waybar/config.jsonc");
  assert_is_symlink(&applied);
  let applied_target = fs::read_link(applied).unwrap();
  assert!(applied_target.ends_with("themes/omarchy-default/config.jsonc"));
}
