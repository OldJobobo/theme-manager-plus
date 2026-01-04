#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_DIR="${HOME}/.local/bin"
TARGET_LINK="${TARGET_DIR}/theme-manager"
RUST_DIR="${SCRIPT_DIR}/rust"

if ! command -v cargo >/dev/null 2>&1; then
  echo "theme-manager: cargo is required to build the Rust CLI" >&2
  exit 1
fi

mkdir -p "${TARGET_DIR}"

(
  cd "${RUST_DIR}"
  cargo build --release
)

SOURCE_BIN="${RUST_DIR}/target/release/theme-manager"
ln -sfn "${SOURCE_BIN}" "${TARGET_LINK}"

mkdir -p "${HOME}/.config/starship-themes"

printf 'Linked %s -> %s\n' "${TARGET_LINK}" "${SOURCE_BIN}"
