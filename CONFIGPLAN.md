# Config File Plan

## 1) Scope & Goals
- Allow users to set defaults for flags (`-q`, `-w`).
- Allow overriding hard-coded paths (`~/.config/omarchy`, `~/.config/waybar`, etc.).
- Keep behavior backward compatible when no config exists.

## 2) Config File Location & Precedence
- Default path: `~/.config/theme-manager/config`
- Optional project/local override: `./.theme-manager.conf`
- Precedence: CLI flags > env vars > local config > user config > defaults.

## 3) Config Format
- Simple `KEY=VALUE` shell-style file (easy to parse).
- Only allow known keys; ignore unknowns with a warning.

## 4) Config Keys (Initial Set)
- `THEME_ROOT_DIR` (default: `~/.config/omarchy/themes`)
- `CURRENT_THEME_LINK` (default: `~/.config/omarchy/current/theme`)
- `OMARCHY_BIN_DIR` (optional; used to prepend PATH)
- `WAYBAR_DIR` (default: `~/.config/waybar`)
- `WAYBAR_THEMES_DIR` (default: `~/.config/waybar/themes`)
- `DEFAULT_WAYBAR_MODE` (`auto` or `named`)
- `DEFAULT_WAYBAR_NAME` (used when mode is `named`)
- `QUIET_MODE_DEFAULT` (`1` or empty)

## 5) Load Order Logic
- Add `load_config()` early in `main`.
- If config file is present, read it and export allowed keys.
- Merge with env vars and CLI flags.

## 6) Security & Safety
- Do not `source` arbitrary config without validation.
- Parse lines as `KEY=VALUE` and whitelist keys.
- Trim quotes and whitespace.

## 7) CLI Behavior
- Flags override config.
- Optional `--print-config` to show resolved values for debugging.

## 8) Documentation
- Add a new section in `README.md`:
  - Config location
  - Supported keys
  - Precedence rules
  - Example config

## 9) Tests
- Defaults without config
- Config overrides (paths and quiet mode)
- CLI overrides config
- Unknown keys ignored with warning
