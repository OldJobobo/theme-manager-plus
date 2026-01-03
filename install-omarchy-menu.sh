#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ICON_URL="https://cdn.jsdelivr.net/gh/homarr-labs/dashboard-icons/png/palette.png"

if ! command -v omarchy-tui-install >/dev/null 2>&1; then
  echo "omarchy-tui-install not found in PATH" >&2
  exit 1
fi

omarchy-tui-install "Theme Manager+" "theme-manager browse -q" float "${ICON_URL}"
