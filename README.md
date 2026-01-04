# Theme Manager Plus

## Overview
Theme Manager Plus is a command-line tool that switches Omarchy themes the same way the Omarchy menu does. It is not a replacement for Omarchy’s theming system. Think of it as a direct, scriptable version of “Menu → Style → Theme → <name>”.

What it does:
- Sets the current theme link so Omarchy apps know what theme is active.
- Runs Omarchy’s own theme scripts so apps update the same way they do from the menu.
- Reloads the usual components (Waybar, terminals, notifications, etc.).
- Can also apply a Waybar theme when you ask it to.
- Includes a TUI app launcher entry in Omarchy (see “Omarchy App Launcher Integration”).

## Quick Start
Requirements:
- Omarchy installed on this machine.
- Omarchy scripts available in PATH (or set `OMARCHY_BIN_DIR` in config).
- `starship` is optional (only needed for preset selection or applying Starship presets).
- `kitty` or `chafa` is optional for browse previews (text fallback otherwise).

Common commands:
- `theme-manager list` — show available themes.
- `theme-manager set <ThemeName>` — switch to a theme.
- `theme-manager set <ThemeName> -w` — switch and apply the theme’s Waybar theme.
- `theme-manager browse` — pick a theme and Waybar option in a full‑screen selector.
- Starship presets or user themes can be applied via config defaults or `browse`.

## Command Reference (Short)
- `set <theme> [-w/--waybar [name]] [-q/--quiet]`  
  Switch themes. `-w` applies Waybar (auto or named). `-q` suppresses external command output.
- `browse`  
  Full‑screen selector with theme previews; then Waybar and Starship pickers.
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
Waybar:
- `-w` (no name): use the theme’s `waybar-theme/` folder if it exists.
- `-w <WaybarName>`: use `~/.config/waybar/themes/<WaybarName>/`.
Starship:
- Applies the configured preset or user theme when Starship defaults are set.
Quiet mode:
- `-q` suppresses most external output.

**`browse`**  
Pick a theme in a full‑screen list. If `preview.png` (case-insensitive) exists in the theme folder it will show on the right; otherwise it falls back to `theme.png` (case-insensitive) or the first image in `backgrounds/`.  
Then choose Waybar (default, theme, or named), then Starship (default, preset, or user theme).

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

**`version`**  
Prints the current CLI version.

## Integrations

### Waybar
Two ways to apply Waybar:
- Per-theme: `waybar-theme/config.jsonc` and `style.css` inside the theme folder.
- Shared: `~/.config/waybar/themes/<name>/` with the same two files.

Behavior:
- `-w` with no name uses the theme’s `waybar-theme/` if present.
- `-w <name>` uses the shared Waybar theme.
- Files are copied into `~/.config/waybar/` (no backups) when `WAYBAR_APPLY_MODE="copy"` or when exec mode falls back.
- Waybar is restarted after apply in both modes.

Notes:
- If a theme has `waybar-theme/preview.png` (or `preview.PNG`), the browse screen shows it.
- Theme previews also fall back to `theme.png` (case-insensitive) in the theme root.
- If there is no preview, the browser falls back to the first image in `backgrounds/`.
- Optional helper: `extras/omarchy/tmplus-restart-waybar` restarts Waybar with `-c/--config` and `-s/--style` support.
- `WAYBAR_APPLY_MODE="exec"` (default) uses `tmplus-restart-waybar` to restart Waybar with explicit config/style paths.
- If the helper is not found, Theme Manager+ falls back to copy mode with a warning.

### Starship
Theme Manager Plus can apply Starship prompt configs after a theme switch.

Two sources are supported:
- Starship presets: `starship preset <name>`
- User themes: `~/.config/starship-themes/<name>.toml`
- Theme-specific: `starship.yaml` inside the selected theme directory.

Behavior:
- Defaults are controlled via config (see below).
- `browse` lets you pick a Starship preset or user theme alongside the theme selection.
- User themes appear in the picker when they exist as `*.toml` in `~/.config/starship-themes/`.
- The active config is written to `~/.config/starship.toml`.
- `install.sh` ensures `~/.config/starship-themes/` exists for user themes.
- Preset names can be listed with `starship preset --list`.
- Example themes are available in `extras/starship-themes/` (copy into `~/.config/starship-themes/`).

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
- `~/.config/theme-manager/config.toml`
- `./.theme-manager.toml` (local file wins)

Migration note:
- The Rust CLI uses TOML. The legacy Bash CLI still supports `~/.config/theme-manager/config` and `./.theme-manager.conf`.

TOML sections (all optional):
- `[paths]` for theme, waybar, and starship locations
- `[waybar]` for apply mode and defaults
- `[starship]` for preset/named defaults
- `[behavior]` for quiet defaults

Precedence order: CLI flags > env vars > local config > user config > defaults.

Use `theme-manager print-config` to see resolved values.

Environment flags (optional):
- `THEME_MANAGER_SKIP_APPS=1` skips app reloads and setters (fast, but not full parity).
- `THEME_MANAGER_SKIP_HOOK=1` skips `omarchy-hook theme-set`.

## Omarchy App Launcher Integration
Theme Manager Plus integrates as a TUI app in Omarchy’s app launcher. This gives you
a standalone, floating terminal window that runs the same `browse` flow as the CLI.

Install the launcher:
```
./install-omarchy-menu.sh
```

This uses Omarchy’s TUI installer to create:
- `~/.local/share/applications/Theme Manager+.desktop`

The launcher opens a floating terminal and runs:
`theme-manager browse -q`.

Optional keybind:
Add a Hyprland bind to open the launcher directly:
```
bindd = SUPER SHIFT, R, Theme Manager+, exec, gtk-launch "Theme Manager+"
```
After saving, reload Hyprland:
```
hyprctl reload
```

## Troubleshooting (Common)
- “theme not found” → check spelling or `THEME_ROOT_DIR`.
- “omarchy-* not found” → ensure Omarchy scripts are in PATH or set `OMARCHY_BIN_DIR`.
- No Waybar change → confirm `waybar-theme/` files or `~/.config/waybar/themes/<name>/`.
- Browse preview missing → check `preview.png`, `theme.png`, or `backgrounds/` in the theme folder.
- Odd warnings from browsers, GTK, or Wayland → usually safe to ignore; use `-q` to quiet them.

## Testing
Run Rust tests with:
```
cd rust
cargo test
```
Legacy Bats tests still live under `tests/` for the Bash CLI.

## Development Notes
- Rust CLI entry: `rust/src/main.rs`
- Bash CLI (legacy): `bin/theme-manager`
- Rust tests: `rust/tests/`
- Legacy Bats tests: `tests/`

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
No. The Rust TUI replaces the `fzf` flow.
