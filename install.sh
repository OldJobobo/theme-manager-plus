#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_DIR="${HOME}/.local/bin"
TARGET_LINK="${TARGET_DIR}/theme-manager"
SOURCE_BIN="${SCRIPT_DIR}/bin/theme-manager"

mkdir -p "${TARGET_DIR}"
ln -sfn "${SOURCE_BIN}" "${TARGET_LINK}"

mkdir -p "${HOME}/.config/starship-themes"

printf 'Linked %s -> %s\n' "${TARGET_LINK}" "${SOURCE_BIN}"
