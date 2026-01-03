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
- `version`  
  Show the current CLI version.

## Command Reference (Detailed)
**`set <ThemeName>`**  
Switch to a theme. This updates the current theme link, runs Omarchy’s theme scripts, reloads components, and triggers the Omarchy hook.  
Use `-w` to apply Waybar:
- `-w` (no name): use the theme’s `waybar-theme/` folder if it exists.
- `-w <WaybarName>`: use `~/.config/waybar/themes/<WaybarName>/`.
Use `-q` to suppress most external output.

**`browse`**  
Pick a theme in a full‑screen list. If `preview.png` exists in the theme folder it will show on the right; otherwise it falls back to the first image in `backgrounds/`.  
Then choose how Waybar should be applied (default, theme, or named).

**`next` / `current` / `bg-next`**  
`next` switches to the next theme in sorted order.  
`current` prints the current theme name.  
`bg-next` cycles the background using Omarchy’s background tool.

**`install <git-url>` / `update` / `remove [theme]`**  
`install` clones a theme into your Omarchy themes folder and activates it.  
`update` pulls updates for git-based themes only.  
`remove` deletes a theme folder (prompts if no name is given).

**`print-config`**  
Prints the resolved config values after applying all overrides.

## Waybar Integration
Two ways to apply Waybar:
- Per-theme: `waybar-theme/config.jsonc` and `style.css` inside the theme folder.
- Shared: `~/.config/waybar/themes/<name>/` with the same two files.

Behavior:
- `-w` with no name uses the theme’s `waybar-theme/` if present.
- `-w <name>` uses the shared Waybar theme.
- Files are copied into `~/.config/waybar/` (no backups).
- Waybar is restarted after apply.

Notes:
- If a theme has `waybar-theme/preview.png`, the browse screen shows it.
- If there is no preview, the browser falls back to the first image in `backgrounds/`.

## Omarchy Compatibility
This tool calls Omarchy’s scripts to stay compatible. It runs:
- `omarchy-theme-bg-next`
- `omarchy-restart-terminal`, `omarchy-restart-waybar`, `omarchy-restart-swayosd`
- `omarchy-theme-set-gnome`, `omarchy-theme-set-browser`, `omarchy-theme-set-vscode`, `omarchy-theme-set-cursor`, `omarchy-theme-set-obsidian`
- `omarchy-hook theme-set`

Order of operations (simplified):
1) Update the current theme link.
2) Run `omarchy-theme-bg-next`.
3) Reload components (terminals, Waybar, notifications, Hyprland, mako).
4) Run Omarchy app-specific theme setters.
5) Trigger the Omarchy theme hook.
6) Apply Waybar theme if requested.

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

Environment flags (optional):
- `THEME_MANAGER_SKIP_APPS=1` skips app reloads and setters (fast, but not full parity).
- `THEME_MANAGER_SKIP_HOOK=1` skips `omarchy-hook theme-set`.

## Troubleshooting (Common)
- “theme not found” → check spelling or `THEME_ROOT_DIR`.
- “omarchy-* not found” → ensure Omarchy scripts are in PATH or set `OMARCHY_BIN_DIR`.
- No Waybar change → confirm `waybar-theme/` files or `~/.config/waybar/themes/<name>/`.
- Browse preview missing → check `preview.png` or `backgrounds/` in the theme folder.
- Odd warnings from browsers, GTK, or Wayland → usually safe to ignore; use `-q` to quiet them.

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

When adding new features:
- Keep behavior compatible with Omarchy’s flow.
- Prefer small, composable shell functions.
- Add a test when behavior changes.

## FAQ
**Why not replace Omarchy’s theming?**  
Because Omarchy already owns the theme system; this tool just drives it.

**Why copy Waybar files instead of symlinks?**  
Copying is more reliable with Waybar restarts and avoids symlink edge cases.

**Can I use custom theme paths?**  
Yes, set `THEME_ROOT_DIR` in your config.

**Does browse require fzf?**  
Yes. Without `fzf`, use `set` and other commands directly.
