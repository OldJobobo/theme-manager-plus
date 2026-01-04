# RustUI Phase 2 Architecture Notes

This document captures the Rust crate scaffolding, module layout, and the planned TOML config schema and migration approach.

## Crate Layout

Location: `rust/`

Modules:
- `cli`: clap-based CLI definitions matching current commands and flags.
- `config`: TOML loading, precedence, and path resolution.
- `paths`: helpers for theme roots and current links.
- `theme_ops`: list/set/next/current/bg-next logic.
- `omarchy`: reloads, setters, hook execution, skip flags.
- `waybar`: apply modes, theme detection, preview resolution.
- `starship`: preset/theme selection and config writing.
- `git_ops`: install/update/remove theme behavior.
- `tui`: browse flow using ratatui + crossterm.
- `preview`: image selection rules and terminal rendering backends.

Note: only `cli` and `config` are scaffolded in Phase 2; other modules are defined in this plan for Phase 3-5.

## CLI Surface (Rust)

Commands:
- `list`
- `set <theme> [-w/--waybar [name]] [-q/--quiet]`
- `next [-w/--waybar [name]] [-q/--quiet]`
- `browse [-q/--quiet]`
- `current`
- `bg-next`
- `print-config`
- `version`
- `install <git-url>`
- `update`
- `remove [theme]`

## TOML Config Schema (Proposed)

Location:
- User: `~/.config/theme-manager/config.toml`
- Local override: `./.theme-manager.toml`

Precedence:
CLI flags > env vars > local TOML > user TOML > defaults

Example:
```toml
[paths]
theme_root_dir = "~/.config/omarchy/themes"
current_theme_link = "~/.config/omarchy/current/theme"
omarchy_bin_dir = "~/.local/share/omarchy/bin"
waybar_dir = "~/.config/waybar"
waybar_themes_dir = "~/.config/waybar/themes"
starship_config = "~/.config/starship.toml"
starship_themes_dir = "~/.config/starship-themes"

[waybar]
apply_mode = "exec" # copy|exec
restart_cmd = "tmplus-restart-waybar"
default_mode = "auto" # auto|named
default_name = ""

[starship]
default_mode = "" # preset|named|theme|"" (empty = none)
default_preset = ""
default_name = ""

[behavior]
quiet_default = false
```

## Migration Notes (Draft)

- On first run, if legacy config exists (`~/.config/theme-manager/config` or `./.theme-manager.conf`), the Rust CLI will:
  - Load legacy config values.
  - Write a TOML file to `~/.config/theme-manager/config.toml` (or `./.theme-manager.toml` for local).
  - Emit a one-time warning noting migration.
- Legacy config parsing will remain supported for one release after migration.

## Preview Backends (Planned)

Primary:
- Kitty `icat` (if `KITTY_WINDOW_ID` set).

Secondary (if available):
- Alacritty graphics protocol (if stable crate exists).
- Ghostty graphics protocol (if stable crate exists).
- Warp (if API exists; otherwise fall back).

Fallback:
- `chafa` if installed.
- `file` output if no preview backend available.
