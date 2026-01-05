mod support;

use support::*;
use std::fs;

#[test]
fn waybar_apply_copy_named() {
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
apply_mode = "copy"
"#,
  );

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a", "-w", "shared"]);
  cmd.assert().success();

  let applied = env.home.join(".config/waybar/config.jsonc");
  assert!(applied.exists());
  let content = fs::read_to_string(applied).unwrap();
  assert_eq!(content, "cfg");
}

#[test]
fn waybar_apply_exec_uses_builtin_restart() {
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
apply_mode = "exec"
default_mode = "auto"
"#,
  );

  let marker = env.temp.path().join("waybar-args");
  let script = env.bin.join("setsid");
  write_script(
    &script,
    &format!(
      "#!/usr/bin/env bash\n\necho \"$@\" > {}\n",
      marker.display()
    ),
  );
  write_stub_ok(&env.bin.join("uwsm-app"));

  let mut cmd = cmd_with_env(&env);
  cmd.env_remove("THEME_MANAGER_SKIP_APPS");
  cmd.args(["set", "theme-a", "-w"]);
  cmd.assert().success();

  for _ in 0..20 {
    if marker.exists() {
      break;
    }
    std::thread::sleep(std::time::Duration::from_millis(10));
  }
  let args = fs::read_to_string(marker).unwrap();
  assert!(args.contains("uwsm-app"));
  assert!(args.contains("waybar"));
  assert!(args.contains("-c"));
  assert!(args.contains("waybar-theme/config.jsonc"));
  assert!(args.contains("-s"));
  assert!(args.contains("waybar-theme/style.css"));
}
