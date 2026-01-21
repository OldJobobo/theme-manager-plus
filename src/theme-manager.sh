#!/usr/bin/env bash
set -euo pipefail

print_usage() {
  cat <<'USAGE'
Usage: theme-manager <command> [args]

Commands:
  list                  List available themes
  set <theme>           Switch to a theme (options: -w/--waybar [name], -q/--quiet)
  next                  Switch to the next theme in order
  browse                Interactive theme + waybar selection (fzf required)
  current               Print the current theme
  bg-next               Switch to the next background in the current theme
  waybar <mode>         Apply Waybar only (auto|none|<name>)
  starship <mode>       Apply Starship only (preset:<name>|named:<name>|theme|none)
  print-config          Show resolved configuration values
  version               Show version
  install <git-url>     Clone and activate a theme from git
  update                Pull updates for git-based themes
  remove [theme]        Remove a theme (prompts if omitted)
  help                  Show this help
USAGE
}

VERSION="0.2.6"

theme_root_dir() {
  echo "${THEME_ROOT_DIR:-${HOME}/.config/omarchy/themes}"
}

theme_root_dirs() {
  local roots=()
  local user_root
  user_root="$(theme_root_dir)"
  if [[ -d "${user_root}" ]]; then
    roots+=("${user_root}")
  fi

  local omarchy_root="${OMARCHY_PATH:-${HOME}/.local/share/omarchy}"
  local default_root="${omarchy_root}/themes"
  if [[ -d "${default_root}" && "${default_root}" != "${user_root}" ]]; then
    roots+=("${default_root}")
  fi

  printf '%s\n' "${roots[@]}"
}

current_theme_link() {
  echo "${CURRENT_THEME_LINK:-${HOME}/.config/omarchy/current/theme}"
}

waybar_dir() {
  echo "${WAYBAR_DIR:-${HOME}/.config/waybar}"
}

waybar_themes_dir() {
  if [[ -n "${WAYBAR_THEMES_DIR:-}" ]]; then
    echo "${WAYBAR_THEMES_DIR}"
  else
    echo "$(waybar_dir)/themes"
  fi
}

starship_config_path() {
  echo "${STARSHIP_CONFIG:-${HOME}/.config/starship.toml}"
}

starship_themes_dir() {
  if [[ -n "${STARSHIP_THEMES_DIR:-}" ]]; then
    echo "${STARSHIP_THEMES_DIR}"
  else
    echo "${HOME}/.config/starship-themes"
  fi
}

waybar_apply_mode() {
  echo "${WAYBAR_APPLY_MODE:-symlink}"
}

skip_apps() {
  [[ -n "${THEME_MANAGER_SKIP_APPS:-}" ]]
}

skip_hook() {
  [[ -n "${THEME_MANAGER_SKIP_HOOK:-}" ]]
}

normalize_theme_name() {
  local input="${1:-}"
  echo "${input}" \
    | sed -E 's/<[^>]+>//g' \
    | tr '[:upper:]' '[:lower:]' \
    | tr ' ' '-'
}

title_case_theme() {
  local name="${1:-}"
  echo "${name}" \
    | tr '-' ' ' \
    | awk '{
        for (i = 1; i <= NF; i++) {
          $i = toupper(substr($i, 1, 1)) tolower(substr($i, 2))
        }
        print
      }'
}

resolve_link_target() {
  local link_path="${1:-}"
  if readlink -f "${link_path}" >/dev/null 2>&1; then
    readlink -f "${link_path}"
    return 0
  fi

  local target
  target="$(readlink "${link_path}")"
  if [[ "${target}" = /* ]]; then
    echo "${target}"
    return 0
  fi

  local link_dir
  link_dir="$(cd "$(dirname "${link_path}")" && pwd)"
  echo "${link_dir}/${target}"
}

current_theme_dir() {
  local link_path
  link_path="$(current_theme_link)"
  if [[ ! -L "${link_path}" ]]; then
    return 1
  fi
  resolve_link_target "${link_path}"
}

list_theme_entries() {
  local roots
  roots="$(theme_root_dirs)"
  if [[ -z "${roots}" ]]; then
    return 1
  fi

  declare -A seen=()
  local root entry name
  while IFS= read -r root; do
    [[ -z "${root}" ]] && continue
    for entry in "${root}"/*; do
      if [[ -d "${entry}" || -L "${entry}" ]]; then
        name="$(basename "${entry}")"
        if [[ -z "${seen[${name}]:-}" ]]; then
          seen["${name}"]=1
          printf '%s\n' "${name}"
        fi
      fi
    done
  done <<< "${roots}"
}

resolve_theme_path() {
  local theme_name="${1:-}"
  local normalized
  normalized="$(normalize_theme_name "${theme_name}")"
  local root candidate
  while IFS= read -r root; do
    [[ -z "${root}" ]] && continue
    candidate="${root}/${normalized}"
    if [[ -d "${candidate}" || -L "${candidate}" ]]; then
      echo "${candidate}"
      return 0
    fi
  done <<< "$(theme_root_dirs)"
  return 1
}

resolve_current_theme_path() {
  local current_dir
  current_dir="$(current_theme_dir 2>/dev/null || true)"
  if [[ -z "${current_dir}" ]]; then
    echo "theme-manager: current theme not set" >&2
    return 1
  fi
  echo "${current_dir}"
}

sorted_theme_entries() {
  list_theme_entries | sort
}

command_exists() {
  command -v "$1" >/dev/null 2>&1
}

warn_missing_command() {
  local command_name="$1"
  if [[ -n "${QUIET_MODE:-}" ]]; then
    return 0
  fi
  echo "theme-manager: ${command_name} not found in PATH" >&2
}

run_or_warn() {
  local command_name="$1"
  shift || true
  if command_exists "${command_name}"; then
    "${command_name}" "$@" || true
  else
    warn_missing_command "${command_name}"
  fi
}

run_filtered() {
  local command_name="$1"
  local context="$2"
  shift 2 || true

  if [[ -n "${QUIET_MODE:-}" ]]; then
    if command_exists "${command_name}"; then
      "${command_name}" "$@" >/dev/null 2>&1 || true
    else
      warn_missing_command "${command_name}"
    fi
    return 0
  fi

  if ! command_exists "${command_name}"; then
    warn_missing_command "${command_name}"
    return 0
  fi

  local tmp
  tmp="$(mktemp)"
  "${command_name}" "$@" >"${tmp}" 2>&1 || true

  local saw_gtk_filter=0
  local saw_wayland_warn=0
  local saw_source_error=0
  local saw_browser_shutdown=0

  local line
  while IFS= read -r line; do
    case "${line}" in
      "Usage: hyprctl "*)
        ;;
      "Usage: makoctl "*)
        ;;
      *"Gtk-WARNING"*"filter"*"valid property name"*)
        saw_gtk_filter=1
        ;;
      warning:\ queue*"zwp_tablet_pad"*)
        saw_wayland_warn=1
        ;;
      "Opening in existing browser session."*)
        ;;
      *"Unchecked runtime.lastError: The browser is shutting down."*)
        saw_browser_shutdown=1
        ;;
      *"Error reading script file 'source'"*)
        saw_source_error=1
        ;;
      *"Something did not go right"*)
        echo "theme-manager: ${context} reported an error; check its config/output." >&2
        ;;
      [0-9][0-9][0-9][0-9]*)
        ;;
      *)
        printf '%s\n' "${line}"
        ;;
    esac
  done <"${tmp}"
  rm -f "${tmp}"

  if [[ ${saw_gtk_filter} -eq 1 ]]; then
    echo "theme-manager: GTK theme uses unsupported 'filter' CSS; warning suppressed."
  fi
  if [[ ${saw_wayland_warn} -eq 1 ]]; then
    echo "theme-manager: Wayland tablet pad cleanup warnings suppressed."
  fi
  if [[ ${saw_browser_shutdown} -eq 1 ]]; then
    echo "theme-manager: browser extension shutdown warning suppressed."
  fi
  if [[ ${saw_source_error} -eq 1 ]]; then
    echo "theme-manager: ${context} reported missing 'source' script; check your shell/theme hooks."
  fi
}

load_config_file() {
  local path="$1"
  local allow_keys=("THEME_ROOT_DIR" "CURRENT_THEME_LINK" "OMARCHY_BIN_DIR" "WAYBAR_DIR" "WAYBAR_THEMES_DIR" "WAYBAR_APPLY_MODE" "WAYBAR_RESTART_CMD" "STARSHIP_CONFIG" "STARSHIP_THEMES_DIR" "DEFAULT_WAYBAR_MODE" "DEFAULT_WAYBAR_NAME" "DEFAULT_STARSHIP_MODE" "DEFAULT_STARSHIP_PRESET" "DEFAULT_STARSHIP_NAME" "QUIET_MODE_DEFAULT")

  [[ -f "${path}" ]] || return 0

  local line key value
  while IFS= read -r line || [[ -n "${line}" ]]; do
    line="${line%%#*}"
    line="$(printf '%s' "${line}" | sed -E 's/^[[:space:]]+|[[:space:]]+$//g')"
    [[ -z "${line}" ]] && continue

    if [[ "${line}" != *"="* ]]; then
      continue
    fi

    key="${line%%=*}"
    value="${line#*=}"
    key="$(printf '%s' "${key}" | sed -E 's/^[[:space:]]+|[[:space:]]+$//g')"
    value="$(printf '%s' "${value}" | sed -E 's/^[[:space:]]+|[[:space:]]+$//g')"
    value="${value%\"}"
    value="${value#\"}"
    value="${value%\'}"
    value="${value#\'}"
    if [[ "${value}" == "~"* ]]; then
      value="${value/#\~/${HOME}}"
    fi
    value="${value//\$\{HOME\}/${HOME}}"
    value="${value//\$HOME/${HOME}}"

    local allowed=false
    local allowed_key
    for allowed_key in "${allow_keys[@]}"; do
      if [[ "${key}" == "${allowed_key}" ]]; then
        allowed=true
        break
      fi
    done

    if [[ "${allowed}" != true ]]; then
      echo "theme-manager: ignoring unknown config key: ${key}" >&2
      continue
    fi

    printf -v "${key}" '%s' "${value}"
  done <"${path}"
}

apply_env_overrides() {
  local key
  for key in THEME_ROOT_DIR CURRENT_THEME_LINK OMARCHY_BIN_DIR WAYBAR_DIR WAYBAR_THEMES_DIR WAYBAR_APPLY_MODE WAYBAR_RESTART_CMD STARSHIP_CONFIG STARSHIP_THEMES_DIR DEFAULT_WAYBAR_MODE DEFAULT_WAYBAR_NAME DEFAULT_STARSHIP_MODE DEFAULT_STARSHIP_PRESET DEFAULT_STARSHIP_NAME QUIET_MODE_DEFAULT QUIET_MODE; do
    if [[ -n "${!key-}" ]]; then
      printf -v "${key}" '%s' "${!key}"
    fi
  done
}

load_config() {
  local user_config="${HOME}/.config/theme-manager/config"
  local local_config="${PWD}/.theme-manager.conf"

  load_config_file "${user_config}"
  load_config_file "${local_config}"
  apply_env_overrides

  if [[ -n "${OMARCHY_BIN_DIR:-}" ]]; then
    if [[ -d "${OMARCHY_BIN_DIR}" ]]; then
      export PATH="${OMARCHY_BIN_DIR}:${PATH}"
    else
      echo "theme-manager: OMARCHY_BIN_DIR not found: ${OMARCHY_BIN_DIR}" >&2
    fi
  fi

  if [[ -z "${QUIET_MODE:-}" && -n "${QUIET_MODE_DEFAULT:-}" ]]; then
    QUIET_MODE=1
  fi
}

apply_default_waybar() {
  if [[ -n "${WAYBAR_MODE:-}" ]]; then
    return 0
  fi
  if [[ -z "${DEFAULT_WAYBAR_MODE:-}" ]]; then
    return 0
  fi

  WAYBAR_MODE="${DEFAULT_WAYBAR_MODE}"
  if [[ "${WAYBAR_MODE}" == "named" && -z "${WAYBAR_NAME:-}" ]]; then
    WAYBAR_NAME="${DEFAULT_WAYBAR_NAME:-}"
  fi
}

apply_default_starship() {
  if [[ -n "${STARSHIP_MODE:-}" ]]; then
    return 0
  fi
  if [[ -z "${DEFAULT_STARSHIP_MODE:-}" ]]; then
    return 0
  fi

  STARSHIP_MODE="${DEFAULT_STARSHIP_MODE}"
  if [[ "${STARSHIP_MODE}" == "preset" && -z "${STARSHIP_PRESET:-}" ]]; then
    STARSHIP_PRESET="${DEFAULT_STARSHIP_PRESET:-}"
  fi
  if [[ "${STARSHIP_MODE}" == "named" && -z "${STARSHIP_NAME:-}" ]]; then
    STARSHIP_NAME="${DEFAULT_STARSHIP_NAME:-}"
  fi
}

print_config() {
  cat <<EOF
THEME_ROOT_DIR=$(theme_root_dir)
CURRENT_THEME_LINK=$(current_theme_link)
OMARCHY_BIN_DIR=${OMARCHY_BIN_DIR:-}
WAYBAR_DIR=$(waybar_dir)
WAYBAR_THEMES_DIR=$(waybar_themes_dir)
WAYBAR_APPLY_MODE=$(waybar_apply_mode)
WAYBAR_RESTART_CMD=${WAYBAR_RESTART_CMD:-}
STARSHIP_CONFIG=$(starship_config_path)
STARSHIP_THEMES_DIR=$(starship_themes_dir)
DEFAULT_WAYBAR_MODE=${DEFAULT_WAYBAR_MODE:-}
DEFAULT_WAYBAR_NAME=${DEFAULT_WAYBAR_NAME:-}
DEFAULT_STARSHIP_MODE=${DEFAULT_STARSHIP_MODE:-}
DEFAULT_STARSHIP_PRESET=${DEFAULT_STARSHIP_PRESET:-}
DEFAULT_STARSHIP_NAME=${DEFAULT_STARSHIP_NAME:-}
QUIET_MODE_DEFAULT=${QUIET_MODE_DEFAULT:-}
QUIET_MODE=${QUIET_MODE:-}
EOF
}

print_version() {
  echo "${VERSION}"
}

apply_waybar_theme() {
  if skip_apps; then
    return 0
  fi

  local waybar_dir=""
  if [[ "${WAYBAR_MODE:-}" == "auto" ]]; then
    local theme_dir
    theme_dir="$(current_theme_dir 2>/dev/null || true)"
    if [[ -z "${theme_dir}" ]]; then
      return 0
    fi
    waybar_dir="${theme_dir}/waybar-theme"
  elif [[ "${WAYBAR_MODE:-}" == "named" ]]; then
    waybar_dir="$(waybar_themes_dir)/${WAYBAR_NAME}"
  else
    return 0
  fi

  if [[ ! -d "${waybar_dir}" ]]; then
    echo "theme-manager: waybar theme directory not found: ${waybar_dir}" >&2
    return 0
  fi

  local config_path="${waybar_dir}/config.jsonc"
  local style_path="${waybar_dir}/style.css"
  if [[ ! -f "${config_path}" || ! -f "${style_path}" ]]; then
    echo "theme-manager: waybar theme missing config.jsonc or style.css in ${waybar_dir}" >&2
    return 0
  fi

  local apply_mode
  apply_mode="$(waybar_apply_mode)"

  local waybar_config_dir
  waybar_config_dir="$(waybar_dir)"
  mkdir -p "${waybar_config_dir}"
  if [[ -z "${QUIET_MODE:-}" ]]; then
    echo "theme-manager: applying waybar config from ${config_path}"
    echo "theme-manager: applying waybar style from ${style_path}"
  fi

  if [[ "${apply_mode}" == "copy" ]]; then
    if ! cp -p -f "${config_path}" "${waybar_config_dir}/config.jsonc"; then
      echo "theme-manager: failed to copy waybar config to ${waybar_config_dir}/config.jsonc" >&2
      return 1
    fi
    if ! cp -p -f "${style_path}" "${waybar_config_dir}/style.css"; then
      echo "theme-manager: failed to copy waybar style to ${waybar_config_dir}/style.css" >&2
      return 1
    fi
  else
    if ! ln -sfn "${config_path}" "${waybar_config_dir}/config.jsonc"; then
      echo "theme-manager: failed to symlink waybar config to ${waybar_config_dir}/config.jsonc" >&2
      return 1
    fi
    if ! ln -sfn "${style_path}" "${waybar_config_dir}/style.css"; then
      echo "theme-manager: failed to symlink waybar style to ${waybar_config_dir}/style.css" >&2
      return 1
    fi
  fi
  run_filtered omarchy-restart-waybar "waybar"
}

list_starship_presets() {
  if ! command -v starship >/dev/null 2>&1; then
    return 0
  fi

  if starship preset --list >/dev/null 2>&1; then
    starship preset --list 2>/dev/null
    return 0
  fi

  starship preset -l 2>/dev/null || true
}

list_starship_themes() {
  local themes_dir
  themes_dir="$(starship_themes_dir)"
  if [[ ! -d "${themes_dir}" ]]; then
    return 0
  fi

  local file
  for file in "${themes_dir}"/*.toml; do
    [[ -f "${file}" ]] || continue
    basename "${file}" .toml
  done
}

apply_starship() {
  if skip_apps; then
    return 0
  fi

  if [[ -z "${STARSHIP_MODE:-}" ]]; then
    return 0
  fi

  local config_path
  config_path="$(starship_config_path)"
  mkdir -p "$(dirname "${config_path}")"

  local themes_dir
  themes_dir="$(starship_themes_dir)"
  mkdir -p "${themes_dir}"

  case "${STARSHIP_MODE}" in
    preset)
      if [[ -z "${STARSHIP_PRESET:-}" ]]; then
        echo "theme-manager: starship preset name is required" >&2
        return 1
      fi
      if ! command -v starship >/dev/null 2>&1; then
        echo "theme-manager: starship not found in PATH" >&2
        return 1
      fi
      if [[ -z "${QUIET_MODE:-}" ]]; then
        echo "theme-manager: applying starship preset ${STARSHIP_PRESET}"
      fi
      if ! starship preset "${STARSHIP_PRESET}" > "${config_path}"; then
        echo "theme-manager: failed to apply starship preset ${STARSHIP_PRESET}" >&2
        return 1
      fi
      ;;
    named)
      if [[ -z "${STARSHIP_NAME:-}" ]]; then
        echo "theme-manager: starship theme name is required" >&2
        return 1
      fi
      local theme_path="${themes_dir}/${STARSHIP_NAME}"
      if [[ "${theme_path}" != *.toml ]]; then
        theme_path="${theme_path}.toml"
      fi
      if [[ ! -f "${theme_path}" ]]; then
        echo "theme-manager: starship theme not found: ${theme_path}" >&2
        return 1
      fi
      if [[ -z "${QUIET_MODE:-}" ]]; then
        echo "theme-manager: applying starship theme ${theme_path}"
      fi
      if ! cp -p -f "${theme_path}" "${config_path}"; then
        echo "theme-manager: failed to copy starship theme to ${config_path}" >&2
        return 1
      fi
      ;;
    theme)
      local theme_path="${STARSHIP_THEME_PATH:-}"
      if [[ -z "${theme_path}" ]]; then
        local current_dir
        current_dir="$(current_theme_dir 2>/dev/null || true)"
        theme_path="${current_dir}/starship.yaml"
      fi
      if [[ -z "${theme_path}" || ! -f "${theme_path}" ]]; then
        echo "theme-manager: starship theme file not found: ${theme_path}" >&2
        return 1
      fi
      if [[ -z "${QUIET_MODE:-}" ]]; then
        echo "theme-manager: applying starship theme ${theme_path}"
      fi
      if ! cp -p -f "${theme_path}" "${config_path}"; then
        echo "theme-manager: failed to copy starship theme to ${config_path}" >&2
        return 1
      fi
      ;;
    *)
      return 0
      ;;
  esac
}

reload_components() {
  if skip_apps; then
    return 0
  fi

  run_filtered omarchy-restart-terminal "terminal"
  if command -v pgrep >/dev/null 2>&1; then
    if pgrep -x waybar >/dev/null 2>&1; then
      run_filtered omarchy-restart-waybar "waybar"
    fi
  else
    run_filtered omarchy-restart-waybar "waybar"
  fi
  run_filtered omarchy-restart-swayosd "swayosd"
  run_filtered hyprctl "hyprctl" reload
  run_filtered makoctl "makoctl" reload
  if command -v pkill >/dev/null 2>&1; then
    pkill -SIGUSR2 btop >/dev/null 2>&1 || true
  fi
}

apply_theme_setters() {
  if skip_apps; then
    return 0
  fi

  run_filtered omarchy-theme-set-gnome "gnome"
  run_filtered omarchy-theme-set-browser "browser"
  run_filtered omarchy-theme-set-vscode "vscode"
  run_filtered omarchy-theme-set-cursor "cursor"
  run_filtered omarchy-theme-set-obsidian "obsidian"
}

cmd_list() {
  local themes_dir
  themes_dir="$(theme_root_dir)"

  if [[ ! -d "${themes_dir}" ]]; then
    echo "theme-manager: themes directory not found: ${themes_dir}" >&2
    return 1
  fi

  local entries
  entries="$(sorted_theme_entries || true)"
  if [[ -z "${entries}" ]]; then
    return 0
  fi

  while IFS= read -r name; do
    [[ -z "${name}" ]] && continue
    title_case_theme "${name}"
  done <<< "${entries}"
}

cmd_set() {
  local theme_name="${1:-}"
  if [[ -z "${theme_name}" ]]; then
    echo "theme-manager: missing theme name" >&2
    return 2
  fi

  local normalized_name
  normalized_name="$(normalize_theme_name "${theme_name}")"

  local theme_path=""
  if ! theme_path="$(resolve_theme_path "${normalized_name}")"; then
    if [[ "${normalized_name}" != "${theme_name}" ]]; then
      echo "theme-manager: theme not found: ${normalized_name} (from '${theme_name}')" >&2
    else
      echo "theme-manager: theme not found: ${normalized_name}" >&2
    fi
    return 1
  fi
  if [[ -L "${theme_path}" && ! -e "${theme_path}" ]]; then
    echo "theme-manager: theme symlink is broken: ${theme_path}" >&2
    return 1
  fi

  local current_link
  current_link="$(current_theme_link)"
  mkdir -p "$(dirname "${current_link}")"
  ln -sfn "${theme_path}" "${current_link}"

  if skip_apps; then
    :
  else
    run_filtered omarchy-theme-bg-next "background"
  fi
  reload_components
  apply_theme_setters

  if ! skip_hook; then
    local hook_path="${HOME}/.config/omarchy/hooks/theme-set"
    if [[ -x "${hook_path}" ]]; then
      if [[ -n "${QUIET_MODE:-}" ]]; then
        "${hook_path}" "${normalized_name}" >/dev/null 2>&1 || true
      else
        "${hook_path}" "${normalized_name}"
      fi
    fi
  fi

  apply_waybar_theme
  apply_starship
}

cmd_waybar() {
  local mode="${1:-}"
  if [[ -z "${mode}" ]]; then
    echo "theme-manager: missing waybar mode (auto|none|<name>)" >&2
    return 2
  fi

  case "${mode}" in
    none)
      WAYBAR_MODE=""
      WAYBAR_NAME=""
      ;;
    auto)
      WAYBAR_MODE="auto"
      WAYBAR_NAME=""
      ;;
    *)
      WAYBAR_MODE="named"
      WAYBAR_NAME="${mode}"
      ;;
  esac

  apply_waybar_theme
}

cmd_starship() {
  local mode="${1:-}"
  if [[ -z "${mode}" ]]; then
    echo "theme-manager: missing starship mode (preset:<name>|named:<name>|theme|none)" >&2
    return 2
  fi

  STARSHIP_MODE=""
  STARSHIP_PRESET=""
  STARSHIP_NAME=""
  STARSHIP_THEME_PATH=""

  case "${mode}" in
    none)
      STARSHIP_MODE="none"
      ;;
    theme)
      STARSHIP_MODE="theme"
      ;;
    preset:*)
      STARSHIP_MODE="preset"
      STARSHIP_PRESET="${mode#preset:}"
      ;;
    named:*)
      STARSHIP_MODE="named"
      STARSHIP_NAME="${mode#named:}"
      ;;
    *)
      local themes_dir
      themes_dir="$(starship_themes_dir)"
      if [[ -f "${themes_dir}/${mode}.toml" ]]; then
        STARSHIP_MODE="named"
        STARSHIP_NAME="${mode}"
      else
        STARSHIP_MODE="preset"
        STARSHIP_PRESET="${mode}"
      fi
      ;;
  esac

  apply_starship
}

cmd_next() {
  local entries
  entries="$(sorted_theme_entries || true)"
  if [[ -z "${entries}" ]]; then
    echo "theme-manager: no themes available" >&2
    return 1
  fi

  local current_dir
  current_dir="$(current_theme_dir 2>/dev/null || true)"
  local current_name=""
  if [[ -n "${current_dir}" ]]; then
    current_name="$(basename "${current_dir}")"
  fi

  local themes=()
  local name
  while IFS= read -r name; do
    [[ -z "${name}" ]] && continue
    themes+=("${name}")
  done <<< "${entries}"

  if [[ ${#themes[@]} -eq 0 ]]; then
    echo "theme-manager: no themes available" >&2
    return 1
  fi

  local next_theme="${themes[0]}"
  local i
  for ((i = 0; i < ${#themes[@]}; i++)); do
    if [[ "${themes[$i]}" == "${current_name}" ]]; then
      local next_index=$(( (i + 1) % ${#themes[@]} ))
      next_theme="${themes[$next_index]}"
      break
    fi
  done

  cmd_set "${next_theme}"
}

cmd_current() {
  local current_link
  current_link="$(current_theme_link)"
  if [[ ! -L "${current_link}" ]]; then
    echo "theme-manager: current theme not set: ${current_link}" >&2
    return 1
  fi

  local target
  target="$(resolve_link_target "${current_link}")"
  title_case_theme "$(basename "${target}")"
}

cmd_bg_next() {
  if ! command -v omarchy-theme-bg-next >/dev/null 2>&1; then
    echo "theme-manager: omarchy-theme-bg-next not found in PATH" >&2
    return 1
  fi

  omarchy-theme-bg-next
}

parse_waybar_args() {
  WAYBAR_MODE=""
  WAYBAR_NAME=""
  PARSED_ARGS=()

  while [[ $# -gt 0 ]]; do
    case "$1" in
      -q|--quiet)
        QUIET_MODE=1
        shift
        ;;
      -w|--waybar)
        if [[ -n "${2:-}" && "${2}" != -* ]]; then
          WAYBAR_MODE="named"
          WAYBAR_NAME="$2"
          shift 2
        else
          WAYBAR_MODE="auto"
          shift
        fi
        ;;
      --waybar=*)
        WAYBAR_MODE="named"
        WAYBAR_NAME="${1#--waybar=}"
        if [[ -z "${WAYBAR_NAME}" ]]; then
          echo "theme-manager: --waybar requires a name when used with =" >&2
          return 2
        fi
        shift
        ;;
      *)
        PARSED_ARGS+=("$1")
        shift
        ;;
    esac
  done
}

parse_quiet_args() {
  local args=()
  while [[ $# -gt 0 ]]; do
    case "$1" in
      -q|--quiet)
        QUIET_MODE=1
        shift
        ;;
      *)
        args+=("$1")
        shift
        ;;
    esac
  done
  printf '%s\n' "${args[@]}"
}

list_waybar_themes() {
  local themes_dir
  themes_dir="$(waybar_themes_dir)"
  if [[ ! -d "${themes_dir}" ]]; then
    return 0
  fi

  local dir
  for dir in "${themes_dir}"/*; do
    [[ -d "${dir}" ]] || continue
    if [[ -f "${dir}/config.jsonc" && -f "${dir}/style.css" ]]; then
      basename "${dir}"
    fi
  done
}

add_unique_option() {
  local option="$1"
  local item
  for item in "${WAYBAR_OPTIONS[@]:-}"; do
    if [[ "${item}" == "${option}" ]]; then
      return 0
    fi
  done
  WAYBAR_OPTIONS+=("${option}")
}

add_unique_starship_option() {
  local option="$1"
  local item
  for item in "${STARSHIP_OPTIONS[@]:-}"; do
    if [[ "${item}" == "${option}" ]]; then
      return 0
    fi
  done
  STARSHIP_OPTIONS+=("${option}")
}

cmd_browse() {
  if ! command -v fzf >/dev/null 2>&1; then
    echo "theme-manager: fzf is required for browse" >&2
    return 1
  fi

  local themes
  themes="$(sorted_theme_entries || true)"
  if [[ -z "${themes}" ]]; then
    echo "theme-manager: no themes available" >&2
    return 1
  fi

  local preview_supported=0
  if command -v chafa >/dev/null 2>&1; then
    preview_supported=1
  elif [[ -n "${KITTY_WINDOW_ID:-}" ]] && command -v kitty >/dev/null 2>&1; then
    preview_supported=1
  fi

  local preview_cmd=""
  if [[ ${preview_supported} -eq 1 ]]; then
  preview_cmd="$(cat <<'PREVIEW_CMD'
preview={1}
preview="${preview#\'}"
preview="${preview%\'}"
preview_dir="${preview%/}"
preview="${preview_dir}/preview.png"
if [ ! -f "$preview" ]; then
  preview="${preview_dir}/theme.png"
fi
if [ ! -f "$preview" ]; then
  preview="${preview_dir}/waybar-theme/preview.png"
fi
if [ ! -f "$preview" ]; then
  preview="$(find "${preview_dir}/backgrounds" -maxdepth 1 -type f \( -name "*.png" -o -name "*.jpg" -o -name "*.jpeg" -o -name "*.webp" \) 2>/dev/null | sort | head -n 1)"
fi
if [ -f "$preview" ]; then
  if command -v chafa >/dev/null 2>&1; then
    chafa --format=symbols --size="${FZF_PREVIEW_COLUMNS}x${FZF_PREVIEW_LINES}" "$preview"
  elif [ -n "$KITTY_WINDOW_ID" ] && command -v kitty >/dev/null 2>&1; then
    kitty +kitten icat --clear --transfer-mode=stream --stdin=no --place "${FZF_PREVIEW_COLUMNS}x${FZF_PREVIEW_LINES}@0x0" --scale-up "$preview"
    i=0
    while [ "$i" -lt "${FZF_PREVIEW_LINES}" ]; do
      printf "\n"
      i=$((i + 1))
    done
  else
    file "$preview"
  fi
else
  if [ -n "$KITTY_WINDOW_ID" ] && command -v kitty >/dev/null 2>&1; then
    kitty +kitten icat --clear --transfer-mode=stream --stdin=no
  fi
  echo "No preview.png or backgrounds image found."
fi
PREVIEW_CMD
)"
  fi

  local theme_choice
  if [[ ${preview_supported} -eq 1 ]]; then
    local preview_bind='q:abort'
    if [[ -n "${KITTY_WINDOW_ID:-}" ]] && command -v kitty >/dev/null 2>&1; then
      preview_bind='q:abort+execute-silent(kitty +kitten icat --clear --stdin=no >/dev/null 2>&1),esc:abort+execute-silent(kitty +kitten icat --clear --stdin=no >/dev/null 2>&1),enter:accept+execute-silent(kitty +kitten icat --clear --stdin=no >/dev/null 2>&1)'
    else
      preview_bind='q:abort,esc:abort'
    fi
    theme_choice="$(printf '%s\n' "__no_theme_change__" "${themes}" | while IFS= read -r name; do
      if [[ "${name}" == "__no_theme_change__" ]]; then
        printf '%s\t%s\n' "__no_theme_change__" "No theme change"
        continue
      fi
      theme_path="$(resolve_theme_path "${name}" || true)"
      [[ -z "${theme_path}" ]] && continue
      printf '%s\t%s\n' "${theme_path}" "$(title_case_theme "${name}")"
    done | fzf --prompt='Select theme: ' --cycle --reverse --height=100% --border --border-label=" Theme Manager+ v${VERSION} " --border-label-pos=2 --padding=1 --preview "${preview_cmd}" --preview-window=right,75% --preview-border=rounded --with-nth=2 --delimiter=$'\t' --bind "${preview_bind}")" || return 0
  else
    theme_choice="$(printf '%s\n' "__no_theme_change__" "${themes}" | while IFS= read -r name; do
      if [[ "${name}" == "__no_theme_change__" ]]; then
        printf '%s\t%s\n' "__no_theme_change__" "No theme change"
        continue
      fi
      theme_path="$(resolve_theme_path "${name}" || true)"
      [[ -z "${theme_path}" ]] && continue
      printf '%s\t%s\n' "${theme_path}" "$(title_case_theme "${name}")"
    done | fzf --prompt='Select theme: ' --cycle --reverse --height=100% --border --border-label=" Theme Manager+ v${VERSION} " --border-label-pos=2 --padding=1 --with-nth=2 --delimiter=$'\t' --bind 'q:abort')" || return 0
  fi

  local theme_path="${theme_choice%%$'\t'*}"
  if [[ -z "${theme_path}" ]]; then
    return 0
  fi

  local theme_id=""
  local no_theme_change=0
  if [[ "${theme_path}" == "__no_theme_change__" ]]; then
    no_theme_change=1
    theme_path="$(resolve_current_theme_path)"
  else
    theme_id="$(basename "${theme_path}")"
    if [[ ! -d "${theme_path}" && ! -L "${theme_path}" ]]; then
      echo "theme-manager: selected theme missing: ${theme_path}" >&2
      return 1
    fi
  fi

  WAYBAR_OPTIONS=()
  add_unique_option "Omarchy default"

  if [[ -f "${theme_path}/waybar-theme/config.jsonc" && -f "${theme_path}/waybar-theme/style.css" ]]; then
    add_unique_option "Use theme waybar"
  fi

  local waybar_theme
  while IFS= read -r waybar_theme; do
    [[ -z "${waybar_theme}" ]] && continue
    add_unique_option "${waybar_theme}"
  done <<< "$(list_waybar_themes)"

  local waybar_preview_cmd=""
  if [[ ${preview_supported} -eq 1 ]]; then
    waybar_preview_cmd="$(cat <<'PREVIEW_CMD'
preview={3}
preview="${preview#\'}"
preview="${preview%\'}"
if [ -z "$preview" ] || [ "$preview" = "-" ]; then
  if [ -n "$KITTY_WINDOW_ID" ] && command -v kitty >/dev/null 2>&1; then
    kitty +kitten icat --clear --transfer-mode=stream --stdin=no
  fi
  echo "No preview.png"
else
  if [ -f "$preview" ]; then
    if command -v chafa >/dev/null 2>&1; then
      chafa --format=symbols --size="${FZF_PREVIEW_COLUMNS}x${FZF_PREVIEW_LINES}" "$preview"
    elif [ -n "$KITTY_WINDOW_ID" ] && command -v kitty >/dev/null 2>&1; then
      kitty +kitten icat --clear --transfer-mode=stream --stdin=no --place "${FZF_PREVIEW_COLUMNS}x${FZF_PREVIEW_LINES}@0x0" --scale-up "$preview"
      i=0
      while [ "$i" -lt "${FZF_PREVIEW_LINES}" ]; do
        printf "\n"
        i=$((i + 1))
      done
    else
      file "$preview"
    fi
  else
    if [ -n "$KITTY_WINDOW_ID" ] && command -v kitty >/dev/null 2>&1; then
      kitty +kitten icat --clear --transfer-mode=stream --stdin=no
    fi
    echo "No preview.png"
  fi
fi
PREVIEW_CMD
)"
  fi

  local waybar_choice
  if [[ ${preview_supported} -eq 1 ]]; then
    waybar_choice="$(
      printf '%s\n' "${WAYBAR_OPTIONS[@]}" | while IFS= read -r option; do
        case "${option}" in
          "Omarchy default")
            printf '%s\t%s\t-\n' "default" "Omarchy default"
            ;;
          "Use theme waybar")
            local preview_file
            preview_file="$(find -L "${theme_path}/waybar-theme" -maxdepth 1 -type f \( -iname "*.png" -o -iname "*.PNG" \) 2>/dev/null | sort | head -n 1)"
            if [[ -n "${preview_file}" ]]; then
              printf '%s\t%s\t%s\n' "theme" "Use theme waybar" "${preview_file}"
            else
              printf '%s\t%s\t-\n' "theme" "Use theme waybar"
            fi
            ;;
          *)
            local preview_path
            preview_path="$(find -L "$(waybar_themes_dir)/${option}" -maxdepth 1 -type f \( -iname "*.png" -o -iname "*.PNG" \) 2>/dev/null | sort | head -n 1)"
            if [[ -n "${preview_path}" ]]; then
              printf '%s\t%s\t%s\n' "named" "${option}" "${preview_path}"
            else
              printf '%s\t%s\t-\n' "named" "${option}"
            fi
            ;;
        esac
      done | fzf --prompt='Select Waybar: ' --cycle --reverse --height=100% --border --border-label=" Theme Manager+ v${VERSION} " --border-label-pos=2 --padding=1 --preview "${waybar_preview_cmd}" --preview-window=right,75% --preview-border=rounded --with-nth=2 --delimiter=$'\t' --bind 'q:abort'
    )" || return 0
  else
    waybar_choice="$(printf '%s\n' "${WAYBAR_OPTIONS[@]}" | fzf --prompt='Select Waybar: ' --cycle --reverse --height=100% --border --border-label=" Theme Manager+ v${VERSION} " --border-label-pos=2 --padding=1 --bind 'q:abort')" || return 0
  fi

  local waybar_kind="${waybar_choice%%$'\t'*}"
  local waybar_label="${waybar_choice#*$'\t'}"
  waybar_label="${waybar_label%%$'\t'*}"

  case "${waybar_kind:-${waybar_label}}" in
    "default"|"Omarchy default")
      WAYBAR_MODE=""
      WAYBAR_NAME=""
      ;;
    "theme"|"Use theme waybar")
      WAYBAR_MODE="auto"
      WAYBAR_NAME=""
      ;;
    *)
      WAYBAR_MODE="named"
      WAYBAR_NAME="${waybar_label:-${waybar_choice}}"
      ;;
  esac

  STARSHIP_OPTIONS=()
  add_unique_starship_option "Omarchy default"

  if [[ -f "${theme_path}/starship.yaml" ]]; then
    add_unique_starship_option "Use theme starship"
  fi

  local preset
  while IFS= read -r preset; do
    [[ -z "${preset}" ]] && continue
    add_unique_starship_option "Preset: ${preset}"
  done <<< "$(list_starship_presets)"

  local theme
  while IFS= read -r theme; do
    [[ -z "${theme}" ]] && continue
    add_unique_starship_option "Theme: ${theme}"
  done <<< "$(list_starship_themes)"

  if [[ ${#STARSHIP_OPTIONS[@]} -gt 1 ]]; then
    local starship_preview_cmd=""
    if command -v starship >/dev/null 2>&1; then
      local themes_dir_val
      themes_dir_val="$(starship_themes_dir)"
      
      # Create a temporary preview script
      local preview_script
      preview_script="$(mktemp)"
      cat > "${preview_script}" <<'PREVIEW_SCRIPT'
#!/usr/bin/env bash
choice="$1"
theme_path="$2"
themes_dir="$3"

if [ "$choice" = "Omarchy default" ]; then
  echo "No Starship config change"
  echo ""
  echo "The current Omarchy theme prompt will be used."
  exit 0
fi

if [ "$choice" = "Use theme starship" ]; then
  config_path="${theme_path}/starship.yaml"
  if [ ! -f "$config_path" ]; then
    echo "Theme-specific Starship config not found"
    exit 0
  fi
elif [[ "$choice" == "Preset: "* ]]; then
  preset_name="${choice#Preset: }"
  tmp_config="$(mktemp)"
  if ! starship preset "$preset_name" > "$tmp_config" 2>/dev/null; then
    echo "Failed to load preset: $preset_name"
    rm -f "$tmp_config"
    exit 0
  fi
  config_path="$tmp_config"
elif [[ "$choice" == "Theme: "* ]]; then
  theme_name="${choice#Theme: }"
  config_path="${themes_dir}/${theme_name}.toml"
  if [ ! -f "$config_path" ]; then
    echo "Theme config not found: $config_path"
    exit 0
  fi
else
  echo "Unknown selection"
  exit 0
fi

if [ -f "$config_path" ]; then
  echo "=== Starship Prompt Preview ==="
  echo ""
  preview_dir="$(mktemp -d)"
  cd "$preview_dir" || exit 0
  git init -q 2>/dev/null
  echo "mock" > README.md
  git add . 2>/dev/null
  
  if command -v perl >/dev/null 2>&1; then
    STARSHIP_CONFIG="$config_path" starship prompt --path "$preview_dir" --terminal-width "${FZF_PREVIEW_COLUMNS:-80}" --jobs 0 2>/dev/null | perl -pe 's/\\\[//g; s/\\\]//g' || echo "Failed to render prompt"
  else
    STARSHIP_CONFIG="$config_path" starship prompt --path "$preview_dir" --terminal-width "${FZF_PREVIEW_COLUMNS:-80}" --jobs 0 2>/dev/null | sed 's/\\[//g; s/\\]//g' || echo "Failed to render prompt"
  fi
  echo ""
  
  if command -v perl >/dev/null 2>&1; then
    right_prompt="$(STARSHIP_CONFIG="$config_path" starship prompt --right --path "$preview_dir" --terminal-width "${FZF_PREVIEW_COLUMNS:-80}" 2>/dev/null | perl -pe 's/\\\[//g; s/\\\]//g')"
  else
    right_prompt="$(STARSHIP_CONFIG="$config_path" starship prompt --right --path "$preview_dir" --terminal-width "${FZF_PREVIEW_COLUMNS:-80}" 2>/dev/null | sed 's/\\[//g; s/\\]//g')"
  fi
  if [ -n "$right_prompt" ]; then
    echo "Right prompt: $right_prompt"
    echo ""
  fi
  
  cd /tmp || exit 0
  rm -rf "$preview_dir"
  [ -n "${tmp_config:-}" ] && rm -f "$tmp_config"
  
  echo "---"
  echo "Config: $choice"
fi
PREVIEW_SCRIPT
      chmod +x "${preview_script}"
      starship_preview_cmd="${preview_script} {} \"${theme_path}\" \"${themes_dir_val}\""
    fi

    local starship_choice
    if [[ -n "${starship_preview_cmd}" ]]; then
      starship_choice="$(printf '%s\n' "${STARSHIP_OPTIONS[@]}" | fzf --prompt='Select Starship: ' --cycle --reverse --height=100% --border --border-label=" Theme Manager+ v${VERSION} " --border-label-pos=2 --padding=1 --preview "${starship_preview_cmd}" --preview-window=right:60%:wrap --bind 'q:abort')" || { rm -f "${preview_script}"; return 0; }
      rm -f "${preview_script}"
    else
      starship_choice="$(printf '%s\n' "${STARSHIP_OPTIONS[@]}" | fzf --prompt='Select Starship: ' --cycle --reverse --height=100% --border --border-label=" Theme Manager+ v${VERSION} " --border-label-pos=2 --padding=1 --bind 'q:abort')" || return 0
    fi
    case "${starship_choice}" in
      "Omarchy default")
        STARSHIP_MODE="none"
        STARSHIP_PRESET=""
        STARSHIP_NAME=""
        STARSHIP_THEME_PATH=""
        ;;
      "Use theme starship")
        STARSHIP_MODE="theme"
        STARSHIP_PRESET=""
        STARSHIP_NAME=""
        STARSHIP_THEME_PATH="${theme_path}/starship.yaml"
        ;;
      "Preset: "*)
        STARSHIP_MODE="preset"
        STARSHIP_PRESET="${starship_choice#Preset: }"
        STARSHIP_NAME=""
        STARSHIP_THEME_PATH=""
        ;;
      "Theme: "*)
        STARSHIP_MODE="named"
        STARSHIP_NAME="${starship_choice#Theme: }"
        STARSHIP_PRESET=""
        STARSHIP_THEME_PATH=""
        ;;
      *)
        STARSHIP_MODE="none"
        STARSHIP_PRESET=""
        STARSHIP_NAME=""
        STARSHIP_THEME_PATH=""
        ;;
    esac
  fi

  apply_default_starship

  if [[ ${no_theme_change} -eq 1 ]]; then
    apply_waybar_theme
    apply_starship
    return 0
  fi

  if ! cmd_set "${theme_id}"; then
    echo "theme-manager: browse failed applying theme: ${theme_id}" >&2
    return 1
  fi
}

cmd_install() {
  local git_url="${1:-}"
  if [[ -z "${git_url}" ]]; then
    echo "theme-manager: missing git URL" >&2
    return 2
  fi

  local repo_name
  repo_name="$(basename "${git_url}")"
  repo_name="${repo_name%.git}"
  repo_name="${repo_name#omarchy-}"
  repo_name="${repo_name%-theme}"

  local theme_name
  theme_name="$(normalize_theme_name "${repo_name}")"

  local themes_dir
  themes_dir="$(theme_root_dir)"
  mkdir -p "${themes_dir}"

  local theme_path="${themes_dir}/${theme_name}"
  if [[ -e "${theme_path}" ]]; then
    echo "theme-manager: theme already exists: ${theme_name}" >&2
    return 1
  fi

  if ! command -v git >/dev/null 2>&1; then
    echo "theme-manager: git is required to install themes" >&2
    return 1
  fi

  git clone "${git_url}" "${theme_path}"
  cmd_set "${theme_name}"
}

cmd_update() {
  local themes_dir
  themes_dir="$(theme_root_dir)"
  if [[ ! -d "${themes_dir}" ]]; then
    echo "theme-manager: themes directory not found: ${themes_dir}" >&2
    return 1
  fi

  if ! command -v git >/dev/null 2>&1; then
    echo "theme-manager: git is required to update themes" >&2
    return 1
  fi

  local entry
  local updated=0
  for entry in "${themes_dir}"/*; do
    if [[ -d "${entry}" && ! -L "${entry}" && -d "${entry}/.git" ]]; then
      git -C "${entry}" pull
      updated=1
    fi
  done

  if [[ ${updated} -eq 0 ]]; then
    echo "theme-manager: no git-based themes found" >&2
  fi
}

cmd_remove() {
  local theme_name="${1:-}"
  local themes_dir
  themes_dir="$(theme_root_dir)"

  if [[ ! -d "${themes_dir}" ]]; then
    echo "theme-manager: themes directory not found: ${themes_dir}" >&2
    return 1
  fi

  if [[ -z "${theme_name}" ]]; then
    local extras=()
    local entry
    for entry in "${themes_dir}"/*; do
      if [[ -d "${entry}" && ! -L "${entry}" ]]; then
        extras+=("$(basename "${entry}")")
      fi
    done

    if [[ ${#extras[@]} -eq 0 ]]; then
      echo "theme-manager: no removable themes found" >&2
      return 1
    fi

    echo "Select a theme to remove:"
    select theme_name in "${extras[@]}"; do
      if [[ -n "${theme_name:-}" ]]; then
        break
      fi
    done
  else
    theme_name="$(normalize_theme_name "${theme_name}")"
  fi

  local theme_path="${themes_dir}/${theme_name}"
  if [[ ! -d "${theme_path}" && ! -L "${theme_path}" ]]; then
    echo "theme-manager: theme not found: ${theme_name}" >&2
    return 1
  fi

  local current_dir
  current_dir="$(current_theme_dir 2>/dev/null || true)"
  if [[ -n "${current_dir}" && "$(basename "${current_dir}")" == "${theme_name}" ]]; then
    local entries
    entries="$(sorted_theme_entries || true)"
    local count
    count="$(printf '%s\n' "${entries}" | sed '/^$/d' | wc -l | tr -d ' ')"
    if [[ "${count}" -le 1 ]]; then
      echo "theme-manager: cannot remove the only theme" >&2
      return 1
    fi
    cmd_next
  fi

  rm -rf "${theme_path}"
}

main() {
  load_config
  local command="${1:-}"
  case "${command}" in
    list)
      cmd_list
      ;;
    set)
      shift
      parse_waybar_args "$@"
      apply_default_waybar
      apply_default_starship
      local theme_name="${PARSED_ARGS[*]:-}"
      cmd_set "${theme_name}"
      ;;
    next)
      shift
      parse_waybar_args "$@"
      apply_default_waybar
      apply_default_starship
      if [[ ${#PARSED_ARGS[@]} -gt 0 ]]; then
        echo "theme-manager: unexpected arguments to next: ${PARSED_ARGS[*]}" >&2
        return 2
      fi
      cmd_next
      ;;
    browse)
      shift
      local remaining
      remaining="$(parse_quiet_args "$@")"
      if [[ -n "${remaining}" ]]; then
        echo "theme-manager: browse takes no arguments" >&2
        return 2
      fi
      cmd_browse
      ;;
    current)
      cmd_current
      ;;
    bg-next)
      cmd_bg_next
      ;;
    print-config)
      shift
      if [[ $# -gt 0 ]]; then
        echo "theme-manager: print-config takes no arguments" >&2
        return 2
      fi
      print_config
      ;;
    version)
      shift
      if [[ $# -gt 0 ]]; then
        echo "theme-manager: version takes no arguments" >&2
        return 2
      fi
      print_version
      ;;
    install)
      shift
      cmd_install "${1:-}"
      ;;
    update)
      cmd_update
      ;;
    remove)
      shift
      cmd_remove "${1:-}"
      ;;
    waybar)
      shift
      cmd_waybar "${1:-}"
      ;;
    starship)
      shift
      cmd_starship "${1:-}"
      ;;
    help|-h|--help)
      print_usage
      return 2
      ;;
    "")
      cmd_browse
      ;;
    *)
      echo "theme-manager: unknown command: ${command}" >&2
      print_usage
      return 2
      ;;
  esac
}
