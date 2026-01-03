#!/usr/bin/env bash
set -euo pipefail

if ! command -v bats >/dev/null 2>&1; then
  echo "bats is required to run tests. Install it and re-run: bats tests" >&2
  exit 1
fi

bats tests
