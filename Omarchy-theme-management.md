# Omarchy theme management (Omarchy v3.3.x)

This document summarizes how Omarchy handles themes (loading, switching, installing, updating) and where theme data is wired into app configs. It reflects Omarchy v3.3.x installed under `~/.local/share/omarchy`.

## Sources inspected
- Theme management scripts: `~/.local/share/omarchy/bin/omarchy-theme-*`
- Hook runner: `~/.local/share/omarchy/bin/omarchy-hook`
- Installer theme setup: `~/.local/share/omarchy/install/config/theme.sh`
- Template generator: `~/.local/share/omarchy/bin/omarchy-theme-set-templates`
- Configs that import theme files:
  - `~/.local/share/omarchy/config/alacritty/alacritty.toml`
  - `~/.local/share/omarchy/config/kitty/kitty.conf`
  - `~/.local/share/omarchy/config/ghostty/config`
  - `~/.local/share/omarchy/config/hypr/hyprland.conf`
  - `~/.local/share/omarchy/config/hypr/hyprlock.conf`
  - `~/.local/share/omarchy/config/hyprland-preview-share-picker/config.yaml`
  - `~/.local/share/omarchy/config/waybar/style.css`
  - `~/.local/share/omarchy/config/swayosd/style.css`
- Theme menu integration: `~/.local/share/omarchy/default/elephant/omarchy_themes.lua`
- Theme library: `~/.local/share/omarchy/themes/*`

## Core paths (installed system)
Omarchy installs into `~/.local/share/omarchy` and exposes `OMARCHY_PATH` via `~/.local/share/omarchy/config/uwsm/env`.

**Theme store and current pointers**
- Default themes live at: `~/.local/share/omarchy/themes/` (aka `$OMARCHY_PATH/themes`)
- User theme directory: `~/.config/omarchy/themes/`
- Current theme directory: `~/.config/omarchy/current/theme` (materialized copy, not a symlink)
- Current theme name file: `~/.config/omarchy/current/theme.name`
- Staging directory: `~/.config/omarchy/current/next-theme`
- Current background symlink: `~/.config/omarchy/current/background`

**Theme-aware app config paths**
- Alacritty: `~/.config/alacritty/alacritty.toml` imports `~/.config/omarchy/current/theme/alacritty.toml`
- Kitty: `~/.config/kitty/kitty.conf` includes `~/.config/omarchy/current/theme/kitty.conf`
- Ghostty: `~/.config/ghostty/config` uses `config-file = ?"~/.config/omarchy/current/theme/ghostty.conf"`
- Hyprland: `~/.config/hypr/hyprland.conf` sources `~/.config/omarchy/current/theme/hyprland.conf`
- Hyprlock: `~/.config/hypr/hyprlock.conf` sources `~/.config/omarchy/current/theme/hyprlock.conf`
- Hyprland Preview Share Picker: uses `~/.config/omarchy/current/theme/hyprland-preview-share-picker.css`
- Waybar: `~/.config/waybar/style.css` imports `~/.config/omarchy/current/theme/waybar.css`
- SwayOSD: `~/.config/swayosd/style.css` imports `~/.config/omarchy/current/theme/swayosd.css`
- btop: `~/.config/btop/btop.conf` uses `color_theme = "current"` and `~/.config/btop/themes/current.theme` is a symlink to `~/.config/omarchy/current/theme/btop.theme`
- mako: `~/.config/mako/config` is a symlink to `~/.config/omarchy/current/theme/mako.ini`

## Theme directory format
Each theme is a directory under `~/.config/omarchy/themes/<theme-name>` or `$OMARCHY_PATH/themes/<theme-name>`.

Common files present in default themes:
- `alacritty.toml`
- `btop.theme`
- `chromium.theme` (single line `r,g,b` string for Chromium theme color)
- `ghostty.conf`
- `hyprland.conf`
- `hyprlock.conf`
- `icons.theme` (GNOME icon theme name, e.g., `Yaru-magenta`)
- `kitty.conf`
- `mako.ini`
- `swayosd.css`
- `vscode.json` (theme name + extension)
- `walker.css`
- `waybar.css`
- `backgrounds/` (image files)
- `preview.png` or `preview.jpg` (theme picker UI)

Optional files:
- `light.mode` (presence marks the theme as light; affects GNOME and Chromium color scheme)
- `obsidian.css` (if present, copied into Obsidian vaults)
- `colors.toml` (drives template-based config generation)

**Template generation (colors.toml)**
- Templates live in: `$OMARCHY_PATH/default/themed/*.tpl`
- User overrides in: `~/.config/omarchy/themed/*.tpl`
- Generated files are written into `~/.config/omarchy/current/next-theme/`
- Output files are skipped if the theme already provided that file.

Example `vscode.json`:
```json
{
  "name": "Tokyo Night",
  "extension": "enkia.tokyo-night"
}
```

## Theme selection and switching flow
### `omarchy-theme-set <theme-name>`
Path: `~/.local/share/omarchy/bin/omarchy-theme-set`

Behavior:
1. Normalizes the theme name: strips HTML tags, lowercases, converts spaces to `-`.
2. Resolves theme path from:
   - `~/.config/omarchy/themes/<theme>`
   - `$OMARCHY_PATH/themes/<theme>`
3. Creates a clean staging dir: `~/.config/omarchy/current/next-theme`.
4. Copies the theme into `next-theme`.
5. Generates template-based configs via `omarchy-theme-set-templates` (if `colors.toml` exists).
6. Swaps `next-theme` into `~/.config/omarchy/current/theme`.
7. Writes `~/.config/omarchy/current/theme.name`.
8. Rotates background via `omarchy-theme-bg-next`.
9. Restarts/reloads affected components:
   - Waybar (if running)
   - swayosd
   - terminal
   - `omarchy-restart-hyprctl`
   - `omarchy-restart-btop`
   - `omarchy-restart-opencode`
   - `omarchy-restart-mako`
10. Applies app-specific theme changes:
   - GNOME (`omarchy-theme-set-gnome`)
   - Browser (`omarchy-theme-set-browser`)
   - VS Code/VSCodium/Cursor (`omarchy-theme-set-vscode`)
   - Obsidian (`omarchy-theme-set-obsidian`)
11. Fires hook: `omarchy-hook theme-set <theme-name>`

### Background cycling: `omarchy-theme-bg-next`
- Reads the current theme name from `~/.config/omarchy/current/theme.name`.
- Looks in:
  - `~/.config/omarchy/current/theme/backgrounds/`
  - `~/.config/omarchy/backgrounds/<theme>/` (user overrides)
- Updates symlink `~/.config/omarchy/current/background` to the next background.
- Relaunches `swaybg` via `uwsm-app` (or uses a black background if none found).

### Theme list/current
- `omarchy-theme-list`: enumerates dirs/symlinks in `~/.config/omarchy/themes` plus `$OMARCHY_PATH/themes`, sorts, title-cases, and replaces `-` with space.
- `omarchy-theme-current`: reads `~/.config/omarchy/current/theme.name` (prints "Unknown" if missing).
- There is no `omarchy-theme-next` script in current Omarchy.

## Installing, updating, and removing themes
### Install: `omarchy-theme-install [git-url]`
- Prompts via `gum` if no URL is provided.
- Clones a git repo into `~/.config/omarchy/themes/<theme-name>`.
- The theme name is derived from the repo name with:
  - `omarchy-` prefix removed
  - `-theme` suffix removed
- If a theme folder already exists, it is removed before cloning.
- Automatically calls `omarchy-theme-set <theme>`.

### Update: `omarchy-theme-update`
- Iterates over `~/.config/omarchy/themes/*/`.
- Runs `git pull` **only** if:
  - directory is **not** a symlink, and
  - contains `.git`.

### Remove: `omarchy-theme-remove [theme]`
- If no name is given, it offers a selection of **extra themes** only (directories that are not symlinks) via `gum choose`.
- Deletes the theme directory.

## App-specific theme setters
### GNOME
`omarchy-theme-set-gnome`:
- If `light.mode` exists, sets:
  - `org.gnome.desktop.interface color-scheme = prefer-light`
  - `gtk-theme = Adwaita`
- Otherwise, uses `prefer-dark` and `Adwaita-dark`.
- Icon theme from `icons.theme` (fallback: `Yaru-blue`).

### Chromium/Brave/Helium
`omarchy-theme-set-browser`:
- Reads `chromium.theme` (RGB) and converts to hex.
- Falls back to a neutral default if the theme file is missing.
- Chromium:
  - `chromium --set-theme-color` with RGB string
  - `chromium --set-color-scheme` light/dark based on `light.mode`
  - Clears `/etc/chromium/policies/managed/color.json`
- Brave:
  - Writes `/etc/brave/policies/managed/color.json` with `BrowserThemeColor`
  - `brave --refresh-platform-policy`
- Installer pre-creates `/etc/{chromium,brave}/policies/managed` and makes them writable.

### VS Code, VSCodium, Cursor
`omarchy-theme-set-vscode`:
- Reads `~/.config/omarchy/current/theme/vscode.json` for `{ name, extension }`.
- Ensures the extension is installed.
- Updates `workbench.colorTheme` in the editor’s `settings.json` (JSONC-safe, uses `sed`).
- If the theme file is missing, it removes `workbench.colorTheme`.
- Skippable via flags:
  - VS Code: `~/.local/state/omarchy/toggles/skip-vscode-theme-changes`
  - VSCodium: `~/.local/state/omarchy/toggles/skip-codium-theme-changes`
  - Cursor: `~/.local/state/omarchy/toggles/skip-cursor-theme-changes`

### Obsidian
`omarchy-theme-set-obsidian`:
- Reads vault paths from `~/.config/obsidian/obsidian.json`.
- If `~/.config/omarchy/current/theme/obsidian.css` exists, it copies it into each vault’s
  `.obsidian/themes/Omarchy/` and ensures `manifest.json` exists.
- No auto-generated CSS pipeline in current Omarchy.

## Hooks
- `omarchy-hook <name> [args...]` executes `~/.config/omarchy/hooks/<name>` if present.
- Theme switching calls: `omarchy-hook theme-set <theme-name>`
- Sample hook: `~/.config/omarchy/hooks/theme-set.sample`

## Initial theme setup during install
`install/config/theme.sh`:
- Ensures `~/.config/omarchy/themes/` exists.
- Calls `omarchy-theme-set "Tokyo Night"`.
- Creates app-specific theme symlinks (btop, mako).
- Ensures browser policy directories exist and are writable.

## What an alternative theme manager must replicate
Minimum viable behavior to stay compatible with Omarchy’s ecosystem:
1. **Set the current theme directory**: `~/.config/omarchy/current/theme` should point at a valid theme tree
   (symlink or materialized directory).
2. **Write the current theme name**: `~/.config/omarchy/current/theme.name`.
3. **Set/update background**: update `~/.config/omarchy/current/background` and restart `swaybg`.
3. **Reload/restart** these components:
   - Waybar, swayosd, hyprland (`hyprctl reload`), mako (`makoctl reload`), btop (`SIGUSR2`), terminals.
4. **Apply app-specific themes**: GNOME, browser, VS Code/VSCodium/Cursor, Obsidian.
5. **Run hook**: `omarchy-hook theme-set <theme-name>`.

If you want feature parity with Omarchy’s built-in tooling, also implement:
- Theme list/current behavior using `~/.config/omarchy/themes/` and `$OMARCHY_PATH/themes/`.
- Git-based install/update/remove semantics (symlinks = default themes, real dirs = user themes).
- Light theme detection by presence of `light.mode`.
- Template generation via `colors.toml` and `.tpl` files into the current theme directory.
