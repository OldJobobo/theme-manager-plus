mod support;

use support::*;
use predicates::prelude::*;
use std::fs;

#[test]
fn preset_save_list_load_remove() {
  let env = setup_env();
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("noir")).unwrap();

  let mut cmd = cmd_with_env(&env);
  cmd.args([
    "preset",
    "save",
    "Daily",
    "--theme",
    "noir",
    "--waybar",
    "auto",
    "--starship",
    "none",
  ]);
  cmd.assert().success();

  let mut cmd = cmd_with_env(&env);
  cmd.args(["preset", "list"]);
  cmd
    .assert()
    .success()
    .stdout(predicates::str::contains("Daily"));

  let mut cmd = cmd_with_env(&env);
  cmd.args(["preset", "load", "Daily"]);
  cmd.assert().success();

  let name = fs::read_to_string(omarchy_dir(&env.home).join("current/theme.name")).unwrap();
  assert_eq!(name.trim(), "noir");

  let mut cmd = cmd_with_env(&env);
  cmd.args(["preset", "remove", "Daily"]);
  cmd.assert().success();

  let mut cmd = cmd_with_env(&env);
  cmd.args(["preset", "list"]);
  cmd
    .assert()
    .success()
    .stdout(predicates::str::contains("Daily").not());
}

#[test]
fn preset_load_errors_on_missing_theme() {
  let env = setup_env();
  let preset_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&preset_dir).unwrap();
  write_toml(
    &preset_dir.join("presets.toml"),
    r#"[preset."Missing"]
theme = "missing-theme"
waybar.mode = "none"
starship.mode = "none"
"#,
  );

  let mut cmd = cmd_with_env(&env);
  cmd.args(["preset", "load", "Missing"]);
  cmd
    .assert()
    .failure()
    .stderr(predicates::str::contains("theme not found"));
}

#[test]
fn preset_load_errors_on_theme_starship_missing() {
  let env = setup_env();
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("noir")).unwrap();

  let preset_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&preset_dir).unwrap();
  write_toml(
    &preset_dir.join("presets.toml"),
    r#"[preset."Starship Missing"]
theme = "noir"
waybar.mode = "none"
starship.mode = "theme"
"#,
  );

  let mut cmd = cmd_with_env(&env);
  cmd.args(["preset", "load", "Starship Missing"]);
  cmd
    .assert()
    .failure()
    .stderr(predicates::str::contains("starship.toml"));
}

#[test]
fn preset_load_waybar_override() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);

  let themes = omarchy_dir(&env.home).join("themes");
  let theme_dir = themes.join("noir/waybar-theme");
  fs::create_dir_all(&theme_dir).unwrap();
  fs::write(theme_dir.join("config.jsonc"), "{ \"theme\": true }").unwrap();
  fs::write(theme_dir.join("style.css"), "/* theme */").unwrap();

  let named_dir = env.home.join(".config/waybar/themes/work");
  fs::create_dir_all(&named_dir).unwrap();
  fs::write(named_dir.join("config.jsonc"), "{ \"named\": true }").unwrap();
  fs::write(named_dir.join("style.css"), "/* named */").unwrap();

  let preset_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&preset_dir).unwrap();
  write_toml(
    &preset_dir.join("presets.toml"),
    r#"[preset."Work"]
theme = "noir"
waybar.mode = "named"
waybar.name = "work"
starship.mode = "none"
"#,
  );

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.env("WAYBAR_APPLY_MODE", "copy");
  cmd.args(["preset", "load", "Work", "-w"]);
  cmd.assert().success();

  let applied = fs::read_to_string(env.home.join(".config/waybar/config.jsonc")).unwrap();
  assert!(applied.contains("\"theme\": true"));
}

#[test]
fn preset_save_persists_walker_value() {
  let env = setup_env();
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("noir")).unwrap();

  let mut cmd = cmd_with_env(&env);
  cmd.args([
    "preset",
    "save",
    "WalkerDaily",
    "--theme",
    "noir",
    "--waybar",
    "none",
    "--walker",
    "named-theme",
    "--starship",
    "none",
  ]);
  cmd.assert().success();

  let presets = fs::read_to_string(env.home.join(".config/theme-manager/presets.toml")).unwrap();
  assert!(presets.contains("[preset.WalkerDaily.walker]"));
  assert!(presets.contains("mode = \"named\""));
  assert!(presets.contains("name = \"named-theme\""));
}

#[test]
fn preset_load_walker_override() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);

  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("noir")).unwrap();

  let shared = env.home.join(".config/walker/themes/shared");
  fs::create_dir_all(&shared).unwrap();
  fs::write(shared.join("style.css"), "shared-style").unwrap();

  let walker_dir = env.home.join(".config/walker");
  fs::create_dir_all(&walker_dir).unwrap();
  fs::write(walker_dir.join("config.toml"), "theme = \"old\"\n").unwrap();

  let preset_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&preset_dir).unwrap();
  write_toml(
    &preset_dir.join("presets.toml"),
    r#"[preset."Work"]
theme = "noir"
waybar.mode = "none"
walker.mode = "none"
starship.mode = "none"
"#,
  );

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["preset", "load", "Work", "--walker", "shared"]);
  cmd.assert().success();

  let config_content = fs::read_to_string(walker_dir.join("config.toml")).unwrap();
  assert!(config_content.contains("theme = \"shared\""));
}

#[test]
fn preset_save_persists_hyprlock_value() {
  let env = setup_env();
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("noir")).unwrap();

  let mut cmd = cmd_with_env(&env);
  cmd.args([
    "preset",
    "save",
    "HyprlockDaily",
    "--theme",
    "noir",
    "--hyprlock",
    "named-hl",
    "--starship",
    "none",
  ]);
  cmd.assert().success();

  let presets = fs::read_to_string(env.home.join(".config/theme-manager/presets.toml")).unwrap();
  assert!(presets.contains("[preset.HyprlockDaily.hyprlock]"));
  assert!(presets.contains("mode = \"named\""));
  assert!(presets.contains("name = \"named-hl\""));
}

#[test]
fn preset_load_hyprlock_override() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);

  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("noir")).unwrap();

  let shared = env.home.join(".config/hypr/themes/hyprlock/shared");
  fs::create_dir_all(&shared).unwrap();
  fs::write(shared.join("hyprlock.conf"), "shared-hyprlock").unwrap();

  let preset_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&preset_dir).unwrap();
  write_toml(
    &preset_dir.join("presets.toml"),
    r#"[preset."Work"]
theme = "noir"
waybar.mode = "none"
walker.mode = "none"
hyprlock.mode = "none"
starship.mode = "none"
"#,
  );

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["preset", "load", "Work", "--hyprlock", "shared"]);
  cmd.assert().success();

  let applied = env.home.join(".config/omarchy/current/theme/hyprlock.conf");
  assert!(applied.exists());
}
