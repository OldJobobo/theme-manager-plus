# Config File Plan

## 1) Scope & Goals
- [x] Allow users to set defaults for flags (`-q`, `-w`).
- [x] Allow overriding hard-coded paths (`~/.config/omarchy`, `~/.config/waybar`, etc.).
- [x] Keep behavior backward compatible when no config exists.

## 2) Config File Location & Precedence
- [x] Default path: `~/.config/theme-manager/config.toml`
- [x] Optional project/local override: `./.theme-manager.toml`
- [x] Precedence: CLI flags > env vars > local config > user config > defaults.

## 3) Config Format
- [x] TOML config with `[paths]`, `[waybar]`, `[starship]`, `[behavior]` sections.
- [x] Unknown keys are ignored by default TOML parsing.

## 4) Config Keys (Initial Set)
- [x] `[paths].theme_root_dir` (default: `~/.config/omarchy/themes`)
- [x] `[paths].current_theme_link` (default: `~/.config/omarchy/current/theme`)
- [x] `[paths].current_background_link` (default: `~/.config/omarchy/current/background`)
- [x] `[paths].omarchy_bin_dir` (optional; prepends PATH)
- [x] `[paths].waybar_dir` (default: `~/.config/waybar`)
- [x] `[paths].waybar_themes_dir` (default: `~/.config/waybar/themes`)
- [x] `[paths].starship_config` (default: `~/.config/starship.toml`)
- [x] `[paths].starship_themes_dir` (default: `~/.config/starship-themes`)
- [x] `[waybar].apply_mode` (`copy` or `exec`)
- [x] `[waybar].restart_cmd` (override restart command when `exec`)
- [x] `[waybar].default_mode` (`auto` or `named`)
- [x] `[waybar].default_name` (used when mode is `named`)
- [x] `[starship].default_mode` (`preset` or `named`)
- [x] `[starship].default_preset` (used when mode is `preset`)
- [x] `[starship].default_name` (used when mode is `named`)
- [x] `[behavior].quiet_default` (true/false)
- [x] `[behavior].awww_transition` (true/false)
- [x] `[behavior].awww_transition_type` (string)
- [x] `[behavior].awww_transition_duration` (seconds float)
- [x] `[behavior].awww_transition_angle` (degrees float)
- [x] `[behavior].awww_transition_fps` (integer)
- [x] `[behavior].awww_transition_pos` (string)
- [x] `[behavior].awww_transition_bezier` (string)
- [x] `[behavior].awww_transition_wave` (string)
- [x] `[behavior].awww_auto_start` (true/false)

## 5) Load Order Logic
- [x] Add `load_config()` early in `main`.
- [x] If config file is present, read it and export allowed keys.
- [x] Merge with env vars and CLI flags.

## 6) Security & Safety
- [x] Use strict TOML parsing and avoid `source` semantics.

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
