#!/usr/bin/env bash
set -euo pipefail

if [ "${BASH_SOURCE+set}" = "set" ] && [ -n "${BASH_SOURCE[0]}" ]; then
  SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
else
  SCRIPT_DIR="$(pwd)"
fi
TARGET_DIR="${HOME}/.local/bin"
TARGET_BIN="${TARGET_DIR}/theme-manager"
RUST_DIR="${SCRIPT_DIR}/rust"
REPO="${THEME_MANAGER_REPO:-OldJobobo/theme-manager-plus}"
VERSION="${THEME_MANAGER_VERSION:-}"
ASSET_SUFFIX="linux-x86_64"

fetch_latest_version() {
  if ! command -v curl >/dev/null 2>&1; then
    return 1
  fi
  local tag
  tag="$(
    curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
      | sed -n 's/.*"tag_name":[[:space:]]*"v\{0,1\}\([^"]*\)".*/\1/p' \
      | head -n1
  )"
  if [ -n "${tag}" ]; then
    VERSION="${tag}"
    return 0
  fi
  return 1
}

download_release() {
  if [ -z "${VERSION}" ]; then
    fetch_latest_version || return 1
  fi
  if ! command -v curl >/dev/null 2>&1; then
    return 1
  fi

  local asset url tmp
  asset="theme-manager-plus-v${VERSION}-${ASSET_SUFFIX}"
  url="https://github.com/${REPO}/releases/download/v${VERSION}/${asset}"
  tmp="$(mktemp)"
  if curl -fL "${url}" -o "${tmp}"; then
    install -m 0755 "${tmp}" "${TARGET_BIN}"
    rm -f "${tmp}"
    printf 'Installed %s (v%s) to %s\n' "${asset}" "${VERSION}" "${TARGET_BIN}"
    return 0
  fi
  rm -f "${tmp}"
  return 1
}

build_from_source() {
  if ! command -v cargo >/dev/null 2>&1; then
    echo "theme-manager: cargo is required to build the Rust CLI" >&2
    return 1
  fi
  if [ ! -d "${RUST_DIR}" ]; then
    echo "theme-manager: rust source not found; clone the repo to build from source" >&2
    return 1
  fi

  (
    cd "${RUST_DIR}"
    cargo build --release
  )

  local source_bin
  source_bin="${RUST_DIR}/target/release/theme-manager"
  install -m 0755 "${source_bin}" "${TARGET_BIN}"
  printf 'Installed %s\n' "${TARGET_BIN}"
}

mkdir -p "${TARGET_DIR}"
PATH_UPDATE_SKIPPED=0
PATH_UPDATE_SKIPPED_FILES=""

mark_path_update_skipped() {
  local file="$1"
  PATH_UPDATE_SKIPPED=1
  if [ -z "${PATH_UPDATE_SKIPPED_FILES}" ]; then
    PATH_UPDATE_SKIPPED_FILES="${file}"
  else
    PATH_UPDATE_SKIPPED_FILES="${PATH_UPDATE_SKIPPED_FILES}, ${file}"
  fi
}

ensure_path_entry() {
  local file="$1"
  local path_line='export PATH="$HOME/.local/bin:$PATH"'
  if [ -f "${file}" ] && [ -r "${file}" ] && grep -q 'HOME/\.local/bin' "${file}"; then
    return 0
  fi
  if [ ! -f "${file}" ]; then
    if ! touch "${file}" 2>/dev/null; then
      echo "theme-manager: warning: cannot create ${file}; skipping PATH update" >&2
      mark_path_update_skipped "${file}"
      return 0
    fi
  fi
  if [ ! -w "${file}" ]; then
    echo "theme-manager: warning: cannot write ${file}; skipping PATH update" >&2
    mark_path_update_skipped "${file}"
    return 0
  fi
  if ! printf '\n%s\n' "${path_line}" >> "${file}" 2>/dev/null; then
    echo "theme-manager: warning: failed to update ${file}; skipping PATH update" >&2
    mark_path_update_skipped "${file}"
    return 0
  fi
}

os="$(uname -s)"
arch="$(uname -m)"
if [ "${os}" != "Linux" ] || [ "${arch}" != "x86_64" ]; then
  echo "theme-manager: no prebuilt binary for ${os}/${arch}; building from source" >&2
  build_from_source
  exit 0
fi

if ! download_release; then
  echo "theme-manager: release download failed, attempting local build" >&2
  build_from_source
fi

mkdir -p "${HOME}/.config/starship-themes"
ensure_path_entry "${HOME}/.profile"
ensure_path_entry "${HOME}/.bashrc"
ensure_path_entry "${HOME}/.zshrc"
if [ "${PATH_UPDATE_SKIPPED}" -eq 1 ]; then
  echo "theme-manager: warning: could not update PATH in: ${PATH_UPDATE_SKIPPED_FILES}" >&2
  echo 'theme-manager: add ~/.local/bin to PATH manually: export PATH="$HOME/.local/bin:$PATH"' >&2
else
  echo "theme-manager: ensured ~/.local/bin is on PATH in ~/.profile, ~/.bashrc, and ~/.zshrc"
fi
