mod support;

use predicates::prelude::PredicateBooleanExt;
use std::fs;
use support::*;

fn add_unlock_theme(env: &TestEnv, name: &str) {
    let theme_dir = omarchy_dir(&env.home).join("themes").join(name);
    fs::create_dir_all(&theme_dir).unwrap();
    fs::write(theme_dir.join("preview-unlock.png"), "preview").unwrap();
    fs::write(theme_dir.join("unlock.png"), "unlock").unwrap();
    fs::write(
        theme_dir.join("colors.toml"),
        "background = \"#112233\"\nforeground = \"#ddeeff\"\n",
    )
    .unwrap();
}

#[test]
fn unlock_list_shows_themes_with_unlock_previews_and_default() {
    let env = setup_env();
    add_unlock_theme(&env, "tokyo-night");
    fs::create_dir_all(omarchy_dir(&env.home).join("themes/no-preview")).unwrap();

    let mut cmd = cmd_with_env(&env);
    cmd.args(["unlock", "list"]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Tokyo Night"))
        .stdout(predicates::str::contains("Default"))
        .stdout(predicates::str::contains("No Preview").not());
}

#[test]
fn unlock_set_uses_legacy_plymouth_set_by_theme() {
    let env = setup_env();
    add_unlock_theme(&env, "tokyo-night");

    let marker = env.temp.path().join("unlock-called");
    write_script(
        &env.bin.join("omarchy-plymouth-set-by-theme"),
        &format!(
            "#!/usr/bin/env bash\nset -euo pipefail\nprintf \"$1\" > {}\n",
            marker.display()
        ),
    );

    let mut cmd = cmd_with_env(&env);
    cmd.args(["unlock", "set", "Tokyo Night"]);
    cmd.assert().success();
    assert_eq!(fs::read_to_string(marker).unwrap(), "tokyo-night");
}

#[test]
fn unlock_set_prefers_unified_omarchy_cli() {
    let env = setup_env();
    add_unlock_theme(&env, "tokyo-night");

    let marker = env.temp.path().join("unlock-unified-called");
    write_script(
        &env.bin.join("omarchy"),
        &format!(
            "#!/usr/bin/env bash\nset -euo pipefail\nif [[ \"$1 $2\" == \"plymouth set-by-theme\" ]]; then printf \"$3\" > {}; exit 0; fi\nexit 127\n",
            marker.display()
        ),
    );
    write_stub_ok(&env.bin.join("omarchy-plymouth-set-by-theme"));

    let mut cmd = cmd_with_env(&env);
    cmd.args(["unlock", "set", "Tokyo Night"]);
    cmd.assert().success();
    assert_eq!(fs::read_to_string(marker).unwrap(), "tokyo-night");
}

#[test]
fn unlock_reset_runs_plymouth_reset() {
    let env = setup_env();
    let marker = env.temp.path().join("unlock-reset-called");
    write_script(
        &env.bin.join("omarchy-plymouth-reset"),
        &format!(
            "#!/usr/bin/env bash\nset -euo pipefail\nprintf reset > {}\n",
            marker.display()
        ),
    );

    let mut cmd = cmd_with_env(&env);
    cmd.args(["unlock", "reset"]);
    cmd.assert().success();
    assert_eq!(fs::read_to_string(marker).unwrap(), "reset");
}

#[test]
fn unlock_set_default_resets_plymouth() {
    let env = setup_env();
    let marker = env.temp.path().join("unlock-default-called");
    write_script(
        &env.bin.join("omarchy-plymouth-reset"),
        &format!(
            "#!/usr/bin/env bash\nset -euo pipefail\nprintf reset > {}\n",
            marker.display()
        ),
    );

    let mut cmd = cmd_with_env(&env);
    cmd.args(["unlock", "set", "Default"]);
    cmd.assert().success();
    assert_eq!(fs::read_to_string(marker).unwrap(), "reset");
}
