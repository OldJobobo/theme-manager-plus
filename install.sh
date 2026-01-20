#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
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
