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
  print-config          Show resolved configuration values
  install <git-url>     Clone and activate a theme from git
  update                Pull updates for git-based themes
  remove [theme]        Remove a theme (prompts if omitted)
  help                  Show this help
USAGE
}

theme_root_dir() {
  echo "${THEME_ROOT_DIR:-${HOME}/.config/omarchy/themes}"
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
  local allow_keys=("THEME_ROOT_DIR" "CURRENT_THEME_LINK" "OMARCHY_BIN_DIR" "WAYBAR_DIR" "WAYBAR_THEMES_DIR" "DEFAULT_WAYBAR_MODE" "DEFAULT_WAYBAR_NAME" "QUIET_MODE_DEFAULT")

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
  for key in THEME_ROOT_DIR CURRENT_THEME_LINK OMARCHY_BIN_DIR WAYBAR_DIR WAYBAR_THEMES_DIR DEFAULT_WAYBAR_MODE DEFAULT_WAYBAR_NAME QUIET_MODE_DEFAULT QUIET_MODE; do
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

print_config() {
  cat <<EOF
THEME_ROOT_DIR=$(theme_root_dir)
CURRENT_THEME_LINK=$(current_theme_link)
OMARCHY_BIN_DIR=${OMARCHY_BIN_DIR:-}
WAYBAR_DIR=$(waybar_dir)
WAYBAR_THEMES_DIR=$(waybar_themes_dir)
DEFAULT_WAYBAR_MODE=${DEFAULT_WAYBAR_MODE:-}
DEFAULT_WAYBAR_NAME=${DEFAULT_WAYBAR_NAME:-}
QUIET_MODE_DEFAULT=${QUIET_MODE_DEFAULT:-}
QUIET_MODE=${QUIET_MODE:-}
EOF
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

  local waybar_config_dir
  waybar_config_dir="$(waybar_dir)"
  mkdir -p "${waybar_config_dir}"
  if [[ -z "${QUIET_MODE:-}" ]]; then
    echo "theme-manager: applying waybar config from ${config_path}"
    echo "theme-manager: applying waybar style from ${style_path}"
  fi
  if ! cp -p -f "${config_path}" "${waybar_config_dir}/config.jsonc"; then
    echo "theme-manager: failed to copy waybar config to ${waybar_config_dir}/config.jsonc" >&2
    return 1
  fi
  if ! cp -p -f "${style_path}" "${waybar_config_dir}/style.css"; then
    echo "theme-manager: failed to copy waybar style to ${waybar_config_dir}/style.css" >&2
    return 1
  fi
  run_filtered omarchy-restart-waybar "waybar"
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
  preview="${preview_dir}/waybar-theme/preview.png"
fi
if [ ! -f "$preview" ]; then
  preview="$(find "${preview_dir}/backgrounds" -maxdepth 1 -type f -name "*.png" -o -name "*.jpg" -o -name "*.jpeg" -o -name "*.webp" 2>/dev/null | sort | head -n 1)"
fi
if [ -f "$preview" ]; then
  if command -v chafa >/dev/null 2>&1; then
    chafa --format=symbols --size="${FZF_PREVIEW_COLUMNS}x${FZF_PREVIEW_LINES}" "$preview"
  elif [ -n "$KITTY_WINDOW_ID" ] && command -v kitty >/dev/null 2>&1; then
    kitty +kitten icat --clear --transfer-mode=stream --stdin=no --place "${FZF_PREVIEW_COLUMNS}x${FZF_PREVIEW_LINES}@0x0" --scale-up "$preview"
  else
    file "$preview"
  fi
else
  echo "No preview.png or backgrounds image found."
fi
PREVIEW_CMD
)"
  fi

  local themes_dir
  themes_dir="$(theme_root_dir)"

  local theme_choice
  if [[ ${preview_supported} -eq 1 ]]; then
    local preview_bind='q:abort'
    if [[ -n "${KITTY_WINDOW_ID:-}" ]] && command -v kitty >/dev/null 2>&1; then
      preview_bind='q:abort+execute-silent(kitty +kitten icat --clear >/dev/null 2>&1),esc:abort+execute-silent(kitty +kitten icat --clear >/dev/null 2>&1),enter:accept+execute-silent(kitty +kitten icat --clear >/dev/null 2>&1)'
    else
      preview_bind='q:abort,esc:abort'
    fi
    theme_choice="$(printf '%s\n' "${themes}" | while IFS= read -r name; do printf '%s\t%s\n' "${themes_dir}/${name}" "$(title_case_theme "${name}")"; done | fzf --prompt='Select theme: ' --cycle --reverse --height=100% --preview "${preview_cmd}" --preview-window=right,60% --with-nth=2 --delimiter=$'\t' --bind "${preview_bind}")" || return 0
  else
    theme_choice="$(printf '%s\n' "${themes}" | while IFS= read -r name; do printf '%s\t%s\n' "${themes_dir}/${name}" "$(title_case_theme "${name}")"; done | fzf --prompt='Select theme: ' --cycle --reverse --height=100% --with-nth=2 --delimiter=$'\t' --bind 'q:abort')" || return 0
  fi

  local theme_path="${theme_choice%%$'\t'*}"
  if [[ -z "${theme_path}" ]]; then
    return 0
  fi

  local theme_id
  theme_id="$(basename "${theme_path}")"
  if [[ ! -d "${theme_path}" && ! -L "${theme_path}" ]]; then
    echo "theme-manager: selected theme missing: ${theme_path}" >&2
    return 1
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
  echo "No preview.png"
  exit 0
fi
if [ -f "$preview" ]; then
  if command -v chafa >/dev/null 2>&1; then
    chafa --format=symbols --size="${FZF_PREVIEW_COLUMNS}x${FZF_PREVIEW_LINES}" "$preview"
  elif [ -n "$KITTY_WINDOW_ID" ] && command -v kitty >/dev/null 2>&1; then
    kitty +kitten icat --clear --transfer-mode=stream --stdin=no --place "${FZF_PREVIEW_COLUMNS}x${FZF_PREVIEW_LINES}@0x0" --scale-up "$preview"
  else
    file "$preview"
  fi
else
  echo "No preview.png"
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
            if [[ -f "${theme_path}/waybar-theme/preview.png" ]]; then
              printf '%s\t%s\t%s\n' "theme" "Use theme waybar" "${theme_path}/waybar-theme/preview.png"
            else
              printf '%s\t%s\t-\n' "theme" "Use theme waybar"
            fi
            ;;
          *)
            local preview_path
            preview_path="$(waybar_themes_dir)/${option}/preview.png"
            if [[ -f "${preview_path}" ]]; then
              printf '%s\t%s\t%s\n' "named" "${option}" "${preview_path}"
            else
              printf '%s\t%s\t-\n' "named" "${option}"
            fi
            ;;
        esac
      done | fzf --prompt='Select Waybar: ' --cycle --reverse --height=100% --preview "${waybar_preview_cmd}" --preview-window=right,60% --with-nth=2 --delimiter=$'\t' --bind 'q:abort'
    )" || return 0
  else
    waybar_choice="$(printf '%s\n' "${WAYBAR_OPTIONS[@]}" | fzf --prompt='Select Waybar: ' --cycle --reverse --height=100% --bind 'q:abort')" || return 0
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
      local theme_name="${PARSED_ARGS[*]:-}"
      cmd_set "${theme_name}"
      ;;
    next)
      shift
      parse_waybar_args "$@"
      apply_default_waybar
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
