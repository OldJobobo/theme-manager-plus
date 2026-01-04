mod support;

use support::*;
use std::fs;

#[test]
fn starship_preset_applies() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  let cfg_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&cfg_dir).unwrap();
  write_toml(
    &cfg_dir.join("config.toml"),
    r#"[starship]
default_mode = "preset"
default_preset = "tokyo-night"
"#,
  );

  let script = env.bin.join("starship");
  write_script(
    &script,
    "#!/usr/bin/env bash\n\nif [[ \"$1\" == \"preset\" && \"$2\" == \"tokyo-night\" ]]; then\n  echo preset-config\n  exit 0\nfi\nexit 1\n",
  );

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a"]);
  cmd.assert().success();

  let applied = env.home.join(".config/starship.toml");
  assert!(applied.exists());
  let content = fs::read_to_string(applied).unwrap();
  assert_eq!(content, "preset-config\n");
}

#[test]
fn starship_named_applies() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  let cfg_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&cfg_dir).unwrap();
  write_toml(
    &cfg_dir.join("config.toml"),
    r#"[starship]
default_mode = "named"
default_name = "rose-pine"
"#,
  );

  let themes_dir = env.home.join(".config/starship-themes");
  fs::create_dir_all(&themes_dir).unwrap();
  fs::write(themes_dir.join("rose-pine.toml"), "user-config").unwrap();

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a"]);
  cmd.assert().success();

  let applied = env.home.join(".config/starship.toml");
  assert!(applied.exists());
  let content = fs::read_to_string(applied).unwrap();
  assert_eq!(content, "user-config");
}

#[test]
fn starship_preset_missing_errors() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  let cfg_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&cfg_dir).unwrap();
  write_toml(
    &cfg_dir.join("config.toml"),
    r#"[starship]
default_mode = "preset"
default_preset = "missing"
"#,
  );

  let script = env.bin.join("starship");
  write_script(&script, "#!/usr/bin/env bash\n\nexit 1\n");

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a"]);
  cmd
    .assert()
    .failure()
    .stderr(predicates::str::contains("failed to apply starship preset"));
}

#[test]
fn starship_named_missing_errors() {
  let env = setup_env();
  add_omarchy_stubs(&env.bin);
  let themes = omarchy_dir(&env.home).join("themes");
  fs::create_dir_all(themes.join("theme-a")).unwrap();

  let cfg_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&cfg_dir).unwrap();
  write_toml(
    &cfg_dir.join("config.toml"),
    r#"[starship]
default_mode = "named"
default_name = "missing"
"#,
  );

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a"]);
  cmd
    .assert()
    .failure()
    .stderr(predicates::str::contains("starship theme not found"));
}
