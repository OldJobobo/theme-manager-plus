# Config File Plan

## 1) Scope & Goals
- [x] Allow users to set defaults for flags (`-q`, `-w`).
- [x] Allow overriding hard-coded paths (`~/.config/omarchy`, `~/.config/waybar`, etc.).
- [x] Keep behavior backward compatible when no config exists.

## 2) Config File Location & Precedence
- [x] Default path: `~/.config/theme-manager/config`
- [x] Optional project/local override: `./.theme-manager.conf`
- [x] Precedence: CLI flags > env vars > local config > user config > defaults.

## 3) Config Format
- [x] Simple `KEY=VALUE` shell-style file (easy to parse).
- [x] Only allow known keys; ignore unknowns with a warning.

## 4) Config Keys (Initial Set)
- [x] `THEME_ROOT_DIR` (default: `~/.config/omarchy/themes`)
- [x] `CURRENT_THEME_LINK` (default: `~/.config/omarchy/current/theme`)
- [x] `OMARCHY_BIN_DIR` (optional; used to prepend PATH)
- [x] `WAYBAR_DIR` (default: `~/.config/waybar`)
- [x] `WAYBAR_THEMES_DIR` (default: `~/.config/waybar/themes`)
- [x] `WAYBAR_APPLY_MODE` (`copy` or `exec`)
- [x] `WAYBAR_RESTART_CMD` (optional; override restart command when `exec`)
- [x] `DEFAULT_WAYBAR_MODE` (`auto` or `named`)
- [x] `DEFAULT_WAYBAR_NAME` (used when mode is `named`)
- [x] `STARSHIP_CONFIG` (default: `~/.config/starship.toml`)
- [x] `STARSHIP_THEMES_DIR` (default: `~/.config/starship-themes`)
- [x] `DEFAULT_STARSHIP_MODE` (`preset` or `named`)
- [x] `DEFAULT_STARSHIP_PRESET` (used when mode is `preset`)
- [x] `DEFAULT_STARSHIP_NAME` (used when mode is `named`)
- [x] `QUIET_MODE_DEFAULT` (`1` or empty)

## 5) Load Order Logic
- [x] Add `load_config()` early in `main`.
- [x] If config file is present, read it and export allowed keys.
- [x] Merge with env vars and CLI flags.

## 6) Security & Safety
- [x] Do not `source` arbitrary config without validation.
- [x] Parse lines as `KEY=VALUE` and whitelist keys.
- [x] Trim quotes and whitespace.

## 7) CLI Behavior
- [x] Flags override config.
- [x] `print-config` shows resolved values for debugging.

## 8) Documentation
- [x] Add a config section in `README.md`:
  - Config location
  - Supported keys
  - Precedence rules
  - Example config reference

## 9) Tests
- [x] Defaults without config
- [x] Config overrides (paths and quiet mode)
- [x] CLI overrides config
- [x] Unknown keys ignored with warning
