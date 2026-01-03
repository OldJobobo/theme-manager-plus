#!/usr/bin/env bats

setup() {
  export HOME="${BATS_TEST_TMPDIR}/home"
  export THEME_MANAGER_SKIP_APPS=1
  export THEME_MANAGER_SKIP_HOOK=1
  mkdir -p "${HOME}/.config/omarchy/themes"
  mkdir -p "${HOME}/.config/omarchy/current"

  export PATH="${BATS_TEST_TMPDIR}/bin:${PATH}"
  mkdir -p "${BATS_TEST_TMPDIR}/bin"
  for cmd in \
    omarchy-restart-waybar \
    omarchy-restart-terminal \
    omarchy-restart-swayosd \
    omarchy-theme-bg-next \
    omarchy-theme-set-gnome \
    omarchy-theme-set-browser \
    omarchy-theme-set-vscode \
    omarchy-theme-set-cursor \
    omarchy-theme-set-obsidian \
    hyprctl \
    makoctl \
    pkill; do
    cat > "${BATS_TEST_TMPDIR}/bin/${cmd}" <<'SCRIPT'
#!/usr/bin/env bash
exit 0
SCRIPT
    chmod +x "${BATS_TEST_TMPDIR}/bin/${cmd}"
  done

  export PWD="${BATS_TEST_TMPDIR}/project"
  mkdir -p "${PWD}"
  cd "${PWD}"

  export BIN="${BATS_TEST_DIRNAME}/../bin/theme-manager"
}

@test "prints usage when no args" {
  run "${BIN}"
  [ "$status" -eq 2 ]
  [[ "$output" == *"Usage: theme-manager"* ]]
}

@test "rejects unknown command" {
  run "${BIN}" nope
  [ "$status" -eq 2 ]
  [[ "$output" == *"unknown command"* ]]
}

@test "lists themes in title case" {
  mkdir -p "${HOME}/.config/omarchy/themes/tokyo-night"
  mkdir -p "${HOME}/.config/omarchy/themes/gruvbox"

  run "${BIN}" list
  [ "$status" -eq 0 ]
  [[ "$output" == *"Tokyo Night"* ]]
  [[ "$output" == *"Gruvbox"* ]]
}

@test "set updates current theme link" {
  mkdir -p "${HOME}/.config/omarchy/themes/tokyo-night"

  run "${BIN}" set "Tokyo Night"
  [ "$status" -eq 0 ]
  [ -L "${HOME}/.config/omarchy/current/theme" ]
  [[ "$(readlink "${HOME}/.config/omarchy/current/theme")" == *"tokyo-night" ]]
}

@test "current prints the active theme" {
  mkdir -p "${HOME}/.config/omarchy/themes/tokyo-night"
  ln -sfn "${HOME}/.config/omarchy/themes/tokyo-night" "${HOME}/.config/omarchy/current/theme"

  run "${BIN}" current
  [ "$status" -eq 0 ]
  [ "$output" = "Tokyo Night" ]
}

@test "next cycles to the next theme" {
  mkdir -p "${HOME}/.config/omarchy/themes/alpha"
  mkdir -p "${HOME}/.config/omarchy/themes/bravo"
  ln -sfn "${HOME}/.config/omarchy/themes/alpha" "${HOME}/.config/omarchy/current/theme"

  run "${BIN}" next
  [ "$status" -eq 0 ]

  run "${BIN}" current
  [ "$status" -eq 0 ]
  [ "$output" = "Bravo" ]
}

@test "bg-next cycles backgrounds" {
  cat <<'SCRIPT' > "${BATS_TEST_TMPDIR}/bin/omarchy-theme-bg-next"
#!/usr/bin/env bash
echo "ok" > "${BATS_TEST_TMPDIR}/bg-next-called"
SCRIPT
  chmod +x "${BATS_TEST_TMPDIR}/bin/omarchy-theme-bg-next"

  run "${BIN}" bg-next
  [ "$status" -eq 0 ]
  [ -f "${BATS_TEST_TMPDIR}/bg-next-called" ]
}

@test "install clones a local repo and sets the theme" {
  if ! command -v git >/dev/null 2>&1; then
    skip "git is required for install tests"
  fi

  local repo_dir="${BATS_TEST_TMPDIR}/omarchy-nord-theme"
  mkdir -p "${repo_dir}"
  git -C "${repo_dir}" init -q
  echo "test" > "${repo_dir}/README.md"
  git -C "${repo_dir}" add README.md
  git -C "${repo_dir}" -c user.email="test@example.com" -c user.name="Test" commit -m "init" -q

  run "${BIN}" install "${repo_dir}"
  [ "$status" -eq 0 ]
  [ -d "${HOME}/.config/omarchy/themes/nord" ]

  run "${BIN}" current
  [ "$status" -eq 0 ]
  [ "$output" = "Nord" ]
}

@test "remove deletes current theme and advances" {
  mkdir -p "${HOME}/.config/omarchy/themes/alpha"
  mkdir -p "${HOME}/.config/omarchy/themes/bravo"
  ln -sfn "${HOME}/.config/omarchy/themes/alpha" "${HOME}/.config/omarchy/current/theme"

  run "${BIN}" remove alpha
  [ "$status" -eq 0 ]
  [ ! -e "${HOME}/.config/omarchy/themes/alpha" ]

  run "${BIN}" current
  [ "$status" -eq 0 ]
  [ "$output" = "Bravo" ]
}

@test "set rejects broken theme symlink" {
  ln -sfn "${HOME}/.config/omarchy/themes/missing-target" "${HOME}/.config/omarchy/themes/broken"

  run "${BIN}" set broken
  [ "$status" -eq 1 ]
  [[ "$output" == *"theme symlink is broken"* ]]
}

@test "print-config shows resolved values" {
  run "${BIN}" print-config
  [ "$status" -eq 0 ]
  [[ "$output" == *"THEME_ROOT_DIR="* ]]
  [[ "$output" == *"CURRENT_THEME_LINK="* ]]
}

@test "version prints a value" {
  run "${BIN}" version
  [ "$status" -eq 0 ]
  [[ "$output" == *"."* ]]
}

@test "config overrides theme root" {
  mkdir -p "${HOME}/.config/theme-manager"
  cat > "${HOME}/.config/theme-manager/config" <<EOF
THEME_ROOT_DIR="${HOME}/.config/omarchy/themes-alt"
EOF

  mkdir -p "${HOME}/.config/omarchy/themes-alt/oasis"

  run "${BIN}" set oasis
  [ "$status" -eq 0 ]
  [ -L "${HOME}/.config/omarchy/current/theme" ]
  [[ "$(readlink "${HOME}/.config/omarchy/current/theme")" == *"/themes-alt/oasis" ]]
}

@test "local config overrides user config" {
  mkdir -p "${HOME}/.config/theme-manager"
  cat > "${HOME}/.config/theme-manager/config" <<EOF
THEME_ROOT_DIR="${HOME}/.config/omarchy/themes-user"
EOF
  mkdir -p "${HOME}/.config/omarchy/themes-user/user-theme"

  cat > "${PWD}/.theme-manager.conf" <<EOF
THEME_ROOT_DIR="${HOME}/.config/omarchy/themes-local"
EOF
  mkdir -p "${HOME}/.config/omarchy/themes-local/local-theme"

  run "${BIN}" set local-theme
  [ "$status" -eq 0 ]
  [[ "$(readlink "${HOME}/.config/omarchy/current/theme")" == *"/themes-local/local-theme" ]]
}

@test "unknown config keys warn" {
  mkdir -p "${HOME}/.config/theme-manager"
  cat > "${HOME}/.config/theme-manager/config" <<EOF
UNKNOWN_KEY="value"
EOF

  run "${BIN}" print-config
  [ "$status" -eq 0 ]
  [[ "$output" == *"ignoring unknown config key: UNKNOWN_KEY"* ]]
}

@test "CLI flags override config defaults" {
  mkdir -p "${HOME}/.config/theme-manager"
  cat > "${HOME}/.config/theme-manager/config" <<EOF
DEFAULT_WAYBAR_MODE="named"
DEFAULT_WAYBAR_NAME="shared"
EOF

  mkdir -p "${HOME}/.config/omarchy/themes/theme-a/waybar-theme"
  echo "a" > "${HOME}/.config/omarchy/themes/theme-a/waybar-theme/config.jsonc"
  echo "a" > "${HOME}/.config/omarchy/themes/theme-a/waybar-theme/style.css"

  mkdir -p "${HOME}/.config/waybar/themes/shared"
  echo "b" > "${HOME}/.config/waybar/themes/shared/config.jsonc"
  echo "b" > "${HOME}/.config/waybar/themes/shared/style.css"

  export THEME_MANAGER_SKIP_APPS=
  run "${BIN}" set theme-a -w
  [ "$status" -eq 0 ]
  [ -f "${HOME}/.config/waybar/config.jsonc" ]
  [[ "$(cat "${HOME}/.config/waybar/config.jsonc")" == "a" ]]
}
