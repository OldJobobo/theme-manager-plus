# Theme Manager Plus

## Overview
Theme Manager Plus is a command-line tool that switches Omarchy themes the same way the Omarchy menu does. It is not a replacement for Omarchy’s theming system. Think of it as a direct, scriptable version of “Menu → Style → Theme → <name>”.

What it does:
- Sets the current theme link so Omarchy apps know what theme is active.
- Runs Omarchy’s own theme scripts so apps update the same way they do from the menu.
- Reloads the usual components (Waybar, terminals, notifications, etc.).
- Can also apply a Waybar theme when you ask it to.

## Quick Start
Requirements:
- Omarchy installed on this machine.
- Omarchy scripts available in PATH (or set `OMARCHY_BIN_DIR` in config).
- `fzf` is optional (only needed for `browse`).

Common commands:
- `./bin/theme-manager list` — show available themes.
- `./bin/theme-manager set <ThemeName>` — switch to a theme.
- `./bin/theme-manager set <ThemeName> -w` — switch and apply the theme’s Waybar theme.
- `./bin/theme-manager browse` — pick a theme and Waybar option in a full‑screen selector.

## Command Reference (Short)
- `set <theme> [-w/--waybar [name]] [-q/--quiet]`  
  Switch themes. `-w` applies Waybar (auto or named). `-q` suppresses external command output.
- `browse`  
  Full‑screen selector with theme previews; then a Waybar picker.
- `next`, `current`, `bg-next`  
  Cycle themes, show current theme, or cycle background.
- `install <git-url>`, `update`, `remove [theme]`  
  Install/update/remove git-based themes.
- `print-config`  
  Show resolved configuration values.

## Waybar Integration
Two ways to apply Waybar:
- Per-theme: `waybar-theme/config.jsonc` and `style.css` inside the theme folder.
- Shared: `~/.config/waybar/themes/<name>/` with the same two files.

Behavior:
- `-w` with no name uses the theme’s `waybar-theme/` if present.
- `-w <name>` uses the shared Waybar theme.
- Files are copied into `~/.config/waybar/` (no backups).
- Waybar is restarted after apply.

## Omarchy Compatibility
This tool calls Omarchy’s scripts to stay compatible. It runs:
- `omarchy-theme-bg-next`
- `omarchy-restart-terminal`, `omarchy-restart-waybar`, `omarchy-restart-swayosd`
- `omarchy-theme-set-gnome`, `omarchy-theme-set-browser`, `omarchy-theme-set-vscode`, `omarchy-theme-set-cursor`, `omarchy-theme-set-obsidian`
- `omarchy-hook theme-set`

## Configuration
You can set defaults in either file:
- `~/.config/theme-manager/config`
- `./.theme-manager.conf` (local file wins)

Keys (all optional):
- `THEME_ROOT_DIR`, `CURRENT_THEME_LINK`
- `OMARCHY_BIN_DIR`
- `WAYBAR_DIR`, `WAYBAR_THEMES_DIR`
- `DEFAULT_WAYBAR_MODE` (`auto` or `named`)
- `DEFAULT_WAYBAR_NAME`
- `QUIET_MODE_DEFAULT` (`1` to enable quiet mode by default)

Use `./bin/theme-manager print-config` to see resolved values.

## Troubleshooting (Common)
- “theme not found” → check spelling or `THEME_ROOT_DIR`.
- “omarchy-* not found” → ensure Omarchy scripts are in PATH or set `OMARCHY_BIN_DIR`.
- No Waybar change → confirm `waybar-theme/` files or `~/.config/waybar/themes/<name>/`.

## Testing
Run tests with:
```
./tests/run.sh
```
Requires `bats` in PATH.

## Development Notes
- Core logic: `src/theme-manager.sh`
- CLI entry: `bin/theme-manager`
- Tests: `tests/`

## FAQ
**Why not replace Omarchy’s theming?**  
Because Omarchy already owns the theme system; this tool just drives it.

**Why copy Waybar files instead of symlinks?**  
Copying is more reliable with Waybar restarts and avoids symlink edge cases.

**Can I use custom theme paths?**  
Yes, set `THEME_ROOT_DIR` in your config.
