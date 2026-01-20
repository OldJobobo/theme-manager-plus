#!/usr/bin/env bash
set -euo pipefail

TARGET_BIN="${HOME}/.local/bin/theme-manager"
PURGE="${1:-}"

if [ -L "${TARGET_BIN}" ] || [ -f "${TARGET_BIN}" ]; then
  rm -f "${TARGET_BIN}"
  printf 'Removed %s\n' "${TARGET_BIN}"
else
  printf 'No installed binary found at %s\n' "${TARGET_BIN}"
fi

if [ "${PURGE}" = "--purge" ]; then
  rm -rf "${HOME}/.config/theme-manager"
  rm -f "${HOME}/.theme-manager.conf"
  printf 'Removed config files\n'
fi
