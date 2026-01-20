mod support;

use support::*;
use std::fs;

#[test]
fn local_config_overrides_user_config() {
  let env = setup_env();
  let project = env.temp.path().join("project");
  fs::create_dir_all(&project).unwrap();

  let user_cfg_dir = env.home.join(".config/theme-manager");
  fs::create_dir_all(&user_cfg_dir).unwrap();
  write_toml(
    &user_cfg_dir.join("config.toml"),
    r#"[paths]
theme_root_dir = "~/.config/omarchy/themes-user"
"#,
  );
  fs::create_dir_all(env.home.join(".config/omarchy/themes-user/user-theme")).unwrap();
  fs::write(
    env.home.join(".config/omarchy/themes-user/user-theme/marker.txt"),
    "user",
  )
  .unwrap();

  write_toml(
    &project.join(".theme-manager.toml"),
    r#"[paths]
theme_root_dir = "~/.config/omarchy/themes-local"
"#,
  );
  fs::create_dir_all(env.home.join(".config/omarchy/themes-local/local-theme")).unwrap();
  fs::write(
    env.home.join(".config/omarchy/themes-local/local-theme/marker.txt"),
    "local",
  )
  .unwrap();

  let mut cmd = cmd_with_env(&env);
  cmd.current_dir(&project);
  cmd.args(["set", "local-theme"]);
  cmd.assert().success();

  let theme_dir = omarchy_dir(&env.home).join("current/theme");
  let marker = fs::read_to_string(theme_dir.join("marker.txt")).unwrap();
  assert_eq!(marker, "local");
}
