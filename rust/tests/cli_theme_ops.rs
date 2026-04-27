mod support;

use predicates::prelude::PredicateBooleanExt;
use std::fs;
use support::*;

#[test]
fn list_titles() {
    let env = setup_env();
    let themes = omarchy_dir(&env.home).join("themes");
    fs::create_dir_all(themes.join("tokyo-night")).unwrap();
    fs::create_dir_all(themes.join("gruvbox")).unwrap();

    let mut cmd = cmd_with_env(&env);
    cmd.arg("list");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Tokyo Night"));
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Gruvbox"));
}

#[test]
fn set_updates_current_theme_dir() {
    let env = setup_env();
    let themes = omarchy_dir(&env.home).join("themes");
    fs::create_dir_all(themes.join("tokyo-night")).unwrap();

    let mut cmd = cmd_with_env(&env);
    cmd.args(["set", "Tokyo Night"]);
    cmd.assert().success();

    let theme_dir = omarchy_dir(&env.home).join("current/theme");
    assert!(theme_dir.is_dir());
    let name = fs::read_to_string(omarchy_dir(&env.home).join("current/theme.name")).unwrap();
    assert_eq!(name.trim(), "tokyo-night");
}

#[test]
fn set_generates_templates_from_colors() {
    let env = setup_env();
    let themes = omarchy_dir(&env.home).join("themes");
    let theme_dir = themes.join("template-test");
    fs::create_dir_all(&theme_dir).unwrap();
    fs::write(
        theme_dir.join("colors.toml"),
        "background = \"#112233\"\nforeground = \"#aabbcc\"\n",
    )
    .unwrap();
    fs::write(
        theme_dir.join("alacritty.toml"),
        "[colors.primary]\nbackground = \"#\"\nforeground = \"#\"\n",
    )
    .unwrap();

    let template_script = env.bin.join("omarchy-theme-set-templates");
    write_script(
        &template_script,
        r#"#!/usr/bin/env bash
set -euo pipefail

colors="$HOME/.config/omarchy/current/next-theme/colors.toml"
output="$HOME/.config/omarchy/current/next-theme/alacritty.toml"

background=$(awk -F '=' '/^background/ { gsub(/[ "]/, "", $2); print $2 }' "$colors")
foreground=$(awk -F '=' '/^foreground/ { gsub(/[ "]/, "", $2); print $2 }' "$colors")

cat > "$output" <<EOF
[colors.primary]
background = "$background"
foreground = "$foreground"
EOF
"#,
    );

    let mut cmd = cmd_with_env(&env);
    cmd.args(["set", "template-test"]);
    cmd.assert().success();

    let rendered =
        fs::read_to_string(omarchy_dir(&env.home).join("current/theme/alacritty.toml")).unwrap();
    assert!(rendered.contains("#112233"));
    assert!(rendered.contains("#aabbcc"));
}

#[test]
fn current_errors_when_missing() {
    let env = setup_env();
    let mut cmd = cmd_with_env(&env);
    cmd.arg("current");
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("current theme not set"));
}

#[test]
fn next_cycles() {
    let env = setup_env();
    let themes = omarchy_dir(&env.home).join("themes");
    fs::create_dir_all(themes.join("alpha")).unwrap();
    fs::create_dir_all(themes.join("bravo")).unwrap();
    let current_dir = omarchy_dir(&env.home).join("current");
    fs::create_dir_all(current_dir.join("theme")).unwrap();
    fs::write(current_dir.join("theme.name"), "alpha").unwrap();

    let mut cmd = cmd_with_env(&env);
    cmd.arg("next");
    cmd.assert().success();

    let name = fs::read_to_string(current_dir.join("theme.name")).unwrap();
    assert_eq!(name.trim(), "bravo");
}

#[test]
fn bg_next_runs_command() {
    let env = setup_env();
    let marker = env.temp.path().join("bg-next-called");
    let script = env.bin.join("omarchy-theme-bg-next");
    write_script(
        &script,
        &format!("#!/usr/bin/env bash\n\necho ok > {}\n", marker.display()),
    );
    let current_dir = omarchy_dir(&env.home).join("current/theme");
    fs::create_dir_all(&current_dir).unwrap();
    fs::write(
        omarchy_dir(&env.home).join("current/theme.name"),
        "tokyo-night",
    )
    .unwrap();

    let mut cmd = cmd_with_env(&env);
    cmd.env("THEME_MANAGER_AWWW_TRANSITION", "0");
    cmd.arg("bg-next");
    cmd.assert().success();
    assert!(marker.exists());
}

#[test]
fn set_rejects_broken_symlink() {
    let env = setup_env();
    let themes = omarchy_dir(&env.home).join("themes");
    fs::create_dir_all(&themes).unwrap();

    let broken = themes.join("broken");
    #[cfg(unix)]
    std::os::unix::fs::symlink(themes.join("missing-target"), &broken).unwrap();

    let mut cmd = cmd_with_env(&env);
    cmd.args(["set", "broken"]);
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("theme symlink is broken"));
}

#[test]
fn set_rejects_empty_waybar_name() {
    let env = setup_env();
    let themes = omarchy_dir(&env.home).join("themes");
    fs::create_dir_all(themes.join("theme-a")).unwrap();

    let mut cmd = cmd_with_env(&env);
    cmd.args(["set", "theme-a", "--waybar="]);
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("--waybar requires a name"));
}

#[test]
fn set_rejects_empty_hyprlock_name() {
    let env = setup_env();
    let themes = omarchy_dir(&env.home).join("themes");
    fs::create_dir_all(themes.join("theme-a")).unwrap();

    let mut cmd = cmd_with_env(&env);
    cmd.args(["set", "theme-a", "--hyprlock="]);
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("--hyprlock requires a name"));
}

#[test]
fn set_succeeds_when_mako_is_missing_but_swaync_is_available() {
    let env = setup_env();
    let themes = omarchy_dir(&env.home).join("themes");
    fs::create_dir_all(themes.join("theme-a")).unwrap();
    add_omarchy_stubs(&env.bin);

    let marker = env.temp.path().join("swaync-reloaded");
    write_script(
        &env.bin.join("swaync-client"),
        &format!(
            "#!/usr/bin/env bash\nset -euo pipefail\nprintf ok > {}\n",
            marker.display()
        ),
    );
    write_script(
        &env.bin.join("makoctl"),
        "#!/usr/bin/env bash\nset -euo pipefail\nexit 1\n",
    );

    let mut cmd = cmd_with_apps_env(&env);
    cmd.args(["set", "theme-a"]);
    cmd.assert().success();
    assert!(marker.exists());
}

#[test]
fn set_reloads_running_mako_when_swaync_client_is_installed() {
    let env = setup_env();
    let themes = omarchy_dir(&env.home).join("themes");
    fs::create_dir_all(themes.join("theme-a")).unwrap();
    add_omarchy_stubs(&env.bin);

    write_script(
        &env.bin.join("pgrep"),
        "#!/usr/bin/env bash\nset -euo pipefail\nif [[ \"${2:-}\" == mako ]]; then echo 1234; exit 0; fi\nexit 1\n",
    );

    let swaync_marker = env.temp.path().join("swaync-reloaded");
    write_script(
        &env.bin.join("swaync-client"),
        &format!(
            "#!/usr/bin/env bash\nset -euo pipefail\nprintf ok > {}\nexit 1\n",
            swaync_marker.display()
        ),
    );
    let mako_marker = env.temp.path().join("mako-reloaded");
    write_script(
        &env.bin.join("makoctl"),
        &format!(
            "#!/usr/bin/env bash\nset -euo pipefail\nprintf ok > {}\n",
            mako_marker.display()
        ),
    );

    let mut cmd = cmd_with_apps_env(&env);
    cmd.args(["set", "theme-a"]);
    cmd.assert().success();
    assert!(mako_marker.exists());
    assert!(!swaync_marker.exists());
}

#[test]
fn set_silences_mako_fallback_when_only_makoctl_is_installed() {
    let env = setup_env();
    let themes = omarchy_dir(&env.home).join("themes");
    fs::create_dir_all(themes.join("theme-a")).unwrap();
    add_omarchy_stubs(&env.bin);
    write_script(
        &env.bin.join("pgrep"),
        "#!/usr/bin/env bash\nset -euo pipefail\nexit 1\n",
    );

    let mako_marker = env.temp.path().join("mako-reloaded");
    write_script(
        &env.bin.join("makoctl"),
        &format!(
            "#!/usr/bin/env bash\nset -euo pipefail\nprintf ok > {}\necho 'Object does not exist at path /fr/emersion/Mako' >&2\nexit 1\n",
            mako_marker.display()
        ),
    );

    let mut cmd = cmd_with_apps_env(&env);
    cmd.args(["set", "theme-a"]);
    cmd.assert()
        .success()
        .stderr(predicates::str::contains("Object does not exist").not());
    assert!(mako_marker.exists());
}
