#!/usr/bin/env bats

setup() {
  export HOME="${BATS_TEST_TMPDIR}/home"
  export THEME_MANAGER_SKIP_APPS=1
  export THEME_MANAGER_SKIP_HOOK=1
  mkdir -p "${HOME}/.config/omarchy/themes"
  mkdir -p "${HOME}/.config/omarchy/current"

  export PATH="${BATS_TEST_TMPDIR}/bin:${PATH}"
  mkdir -p "${BATS_TEST_TMPDIR}/bin"
}

@test "prints usage when no args" {
  run bin/theme-manager
  [ "$status" -eq 2 ]
  [[ "$output" == *"Usage: theme-manager"* ]]
}

@test "rejects unknown command" {
  run bin/theme-manager nope
  [ "$status" -eq 2 ]
  [[ "$output" == *"unknown command"* ]]
}

@test "lists themes in title case" {
  mkdir -p "${HOME}/.config/omarchy/themes/tokyo-night"
  mkdir -p "${HOME}/.config/omarchy/themes/gruvbox"

  run bin/theme-manager list
  [ "$status" -eq 0 ]
  [[ "$output" == *"Tokyo Night"* ]]
  [[ "$output" == *"Gruvbox"* ]]
}

@test "set updates current theme link" {
  mkdir -p "${HOME}/.config/omarchy/themes/tokyo-night"

  run bin/theme-manager set "Tokyo Night"
  [ "$status" -eq 0 ]
  [ -L "${HOME}/.config/omarchy/current/theme" ]
  [[ "$(readlink "${HOME}/.config/omarchy/current/theme")" == *"tokyo-night" ]]
}

@test "current prints the active theme" {
  mkdir -p "${HOME}/.config/omarchy/themes/tokyo-night"
  ln -sfn "${HOME}/.config/omarchy/themes/tokyo-night" "${HOME}/.config/omarchy/current/theme"

  run bin/theme-manager current
  [ "$status" -eq 0 ]
  [ "$output" = "Tokyo Night" ]
}

@test "next cycles to the next theme" {
  mkdir -p "${HOME}/.config/omarchy/themes/alpha"
  mkdir -p "${HOME}/.config/omarchy/themes/bravo"
  ln -sfn "${HOME}/.config/omarchy/themes/alpha" "${HOME}/.config/omarchy/current/theme"

  run bin/theme-manager next
  [ "$status" -eq 0 ]

  run bin/theme-manager current
  [ "$status" -eq 0 ]
  [ "$output" = "Bravo" ]
}

@test "bg-next cycles backgrounds" {
  cat <<'SCRIPT' > "${BATS_TEST_TMPDIR}/bin/omarchy-theme-bg-next"
#!/usr/bin/env bash
echo "ok" > "${BATS_TEST_TMPDIR}/bg-next-called"
SCRIPT
  chmod +x "${BATS_TEST_TMPDIR}/bin/omarchy-theme-bg-next"

  run bin/theme-manager bg-next
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

  run bin/theme-manager install "${repo_dir}"
  [ "$status" -eq 0 ]
  [ -d "${HOME}/.config/omarchy/themes/nord" ]

  run bin/theme-manager current
  [ "$status" -eq 0 ]
  [ "$output" = "Nord" ]
}

@test "remove deletes current theme and advances" {
  mkdir -p "${HOME}/.config/omarchy/themes/alpha"
  mkdir -p "${HOME}/.config/omarchy/themes/bravo"
  ln -sfn "${HOME}/.config/omarchy/themes/alpha" "${HOME}/.config/omarchy/current/theme"

  run bin/theme-manager remove alpha
  [ "$status" -eq 0 ]
  [ ! -e "${HOME}/.config/omarchy/themes/alpha" ]

  run bin/theme-manager current
  [ "$status" -eq 0 ]
  [ "$output" = "Bravo" ]
}

@test "set rejects broken theme symlink" {
  ln -sfn "${HOME}/.config/omarchy/themes/missing-target" "${HOME}/.config/omarchy/themes/broken"

  run bin/theme-manager set broken
  [ "$status" -eq 1 ]
  [[ "$output" == *"theme symlink is broken"* ]]
}
