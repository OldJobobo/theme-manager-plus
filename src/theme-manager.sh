#!/usr/bin/env bash
set -euo pipefail

print_usage() {
  cat <<'USAGE'
Usage: theme-manager <command> [args]

Commands:
  list                  List available themes
  set <theme>           Switch to a theme (options: -w/--waybar [name])
  next                  Switch to the next theme in order
  browse                Interactive theme + waybar selection (fzf required)
  current               Print the current theme
  bg-next               Switch to the next background in the current theme
  install <git-url>     Clone and activate a theme from git
  update                Pull updates for git-based themes
  remove [theme]        Remove a theme (prompts if omitted)
  help                  Show this help
USAGE
}

theme_root_dir() {
  echo "${HOME}/.config/omarchy/themes"
}

current_theme_link() {
  echo "${HOME}/.config/omarchy/current/theme"
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
  local themes_dir
  themes_dir="$(theme_root_dir)"

  if [[ ! -d "${themes_dir}" ]]; then
    return 1
  fi

  local entry
  for entry in "${themes_dir}"/*; do
    if [[ -d "${entry}" || -L "${entry}" ]]; then
      basename "${entry}"
    fi
  done
}

sorted_theme_entries() {
  list_theme_entries | sort
}

run_if_available() {
  local command_name="${1:-}"
  shift || true
  if command -v "${command_name}" >/dev/null 2>&1; then
    "${command_name}" "$@" || true
  fi
}

command_exists() {
  command -v "$1" >/dev/null 2>&1
}

warn_missing_command() {
  local command_name="$1"
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
    waybar_dir="${HOME}/.config/waybar/themes/${WAYBAR_NAME}"
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

  local waybar_config_dir="${HOME}/.config/waybar"
  mkdir -p "${waybar_config_dir}"
  echo "theme-manager: applying waybar config from ${config_path}"
  echo "theme-manager: applying waybar style from ${style_path}"
  if ! cp -p -f "${config_path}" "${waybar_config_dir}/config.jsonc"; then
    echo "theme-manager: failed to copy waybar config to ${waybar_config_dir}/config.jsonc" >&2
    return 1
  fi
  if ! cp -p -f "${style_path}" "${waybar_config_dir}/style.css"; then
    echo "theme-manager: failed to copy waybar style to ${waybar_config_dir}/style.css" >&2
    return 1
  fi
  run_or_warn omarchy-restart-waybar
}

reload_components() {
  if skip_apps; then
    return 0
  fi

  run_or_warn omarchy-restart-terminal
  if command -v pgrep >/dev/null 2>&1; then
    if pgrep -x waybar >/dev/null 2>&1; then
      run_or_warn omarchy-restart-waybar
    fi
  else
    run_or_warn omarchy-restart-waybar
  fi
  run_or_warn omarchy-restart-swayosd
  run_or_warn hyprctl reload
  run_or_warn makoctl reload
  if command -v pkill >/dev/null 2>&1; then
    pkill -SIGUSR2 btop >/dev/null 2>&1 || true
  fi
}

apply_theme_setters() {
  if skip_apps; then
    return 0
  fi

  run_or_warn omarchy-theme-set-gnome
  run_or_warn omarchy-theme-set-browser
  run_or_warn omarchy-theme-set-vscode
  run_or_warn omarchy-theme-set-cursor
  run_or_warn omarchy-theme-set-obsidian
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

  local themes_dir
  themes_dir="$(theme_root_dir)"

  local theme_path="${themes_dir}/${normalized_name}"
  if [[ -L "${theme_path}" && ! -e "${theme_path}" ]]; then
    echo "theme-manager: theme symlink is broken: ${theme_path}" >&2
    return 1
  fi
  if [[ ! -d "${theme_path}" && ! -L "${theme_path}" ]]; then
    if [[ "${normalized_name}" != "${theme_name}" ]]; then
      echo "theme-manager: theme not found: ${normalized_name} (from '${theme_name}')" >&2
    else
      echo "theme-manager: theme not found: ${normalized_name}" >&2
    fi
    return 1
  fi

  local current_link
  current_link="$(current_theme_link)"
  mkdir -p "$(dirname "${current_link}")"
  ln -sfn "${theme_path}" "${current_link}"

  if skip_apps; then
    :
  else
    run_or_warn omarchy-theme-bg-next
  fi
  reload_components
  apply_theme_setters

  if ! skip_hook; then
    local hook_path="${HOME}/.config/omarchy/hooks/theme-set"
    if [[ -x "${hook_path}" ]]; then
      "${hook_path}" "${normalized_name}"
    fi
  fi

  apply_waybar_theme
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
        shift
        ;;
      *)
        PARSED_ARGS+=("$1")
        shift
        ;;
    esac
  done
}

list_waybar_themes() {
  local themes_dir="${HOME}/.config/waybar/themes"
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

  local theme_choice
  theme_choice="$(printf '%s\n' "${themes}" | fzf --prompt='Select theme: ' --cycle --reverse --height=50% --bind 'q:abort')" || return 0

  local theme_dir="${HOME}/.config/omarchy/themes/${theme_choice}"
  if [[ ! -d "${theme_dir}" && ! -L "${theme_dir}" ]]; then
    echo "theme-manager: selected theme missing: ${theme_dir}" >&2
    return 1
  fi

  WAYBAR_OPTIONS=()
  add_unique_option "Omarchy default"

  if [[ -f "${theme_dir}/waybar-theme/config.jsonc" && -f "${theme_dir}/waybar-theme/style.css" ]]; then
    add_unique_option "Use theme waybar"
  fi

  local waybar_theme
  while IFS= read -r waybar_theme; do
    [[ -z "${waybar_theme}" ]] && continue
    add_unique_option "${waybar_theme}"
  done <<< "$(list_waybar_themes)"

  local waybar_choice
  waybar_choice="$(printf '%s\n' "${WAYBAR_OPTIONS[@]}" | fzf --prompt='Select Waybar: ' --cycle --reverse --height=50% --bind 'q:abort')" || return 0

  case "${waybar_choice}" in
    "Omarchy default")
      WAYBAR_MODE=""
      WAYBAR_NAME=""
      ;;
    "Use theme waybar")
      WAYBAR_MODE="auto"
      WAYBAR_NAME=""
      ;;
    *)
      WAYBAR_MODE="named"
      WAYBAR_NAME="${waybar_choice}"
      ;;
  esac

  if ! cmd_set "${theme_choice}"; then
    echo "theme-manager: browse failed applying theme: ${theme_choice}" >&2
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
  local command="${1:-}"
  case "${command}" in
    list)
      cmd_list
      ;;
    set)
      shift
      parse_waybar_args "$@"
      local theme_name="${PARSED_ARGS[*]:-}"
      cmd_set "${theme_name}"
      ;;
    next)
      shift
      parse_waybar_args "$@"
      if [[ ${#PARSED_ARGS[@]} -gt 0 ]]; then
        echo "theme-manager: unexpected arguments to next: ${PARSED_ARGS[*]}" >&2
        return 2
      fi
      cmd_next
      ;;
    browse)
      shift
      if [[ $# -gt 0 ]]; then
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
    help|-h|--help|"")
      print_usage
      return 2
      ;;
    *)
      echo "theme-manager: unknown command: ${command}" >&2
      print_usage
      return 2
      ;;
  esac
}
