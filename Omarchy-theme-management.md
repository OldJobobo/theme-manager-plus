# Omarchy theme management (from Omarchy 3.2.3-2 ISO)

This document summarizes how Omarchy handles themes (loading, switching, installing, updating) and where the theme data is wired into app configs. It is based on the Omarchy repo embedded in `omarchy-3.2.3-2.iso` under `arch/x86_64/airootfs.sfs`.

## Sources inspected
- Theme management scripts: `root/omarchy/bin/omarchy-theme-*`
- Hook runner: `root/omarchy/bin/omarchy-hook`
- Installer theme setup: `root/omarchy/install/config/theme.sh`
- Configs that import theme files:
  - `root/omarchy/config/alacritty/alacritty.toml`
  - `root/omarchy/config/kitty/kitty.conf`
  - `root/omarchy/config/ghostty/config`
  - `root/omarchy/config/hypr/hyprland.conf`
  - `root/omarchy/config/hypr/hyprlock.conf`
  - `root/omarchy/config/hyprland-preview-share-picker/config.yaml`
  - `root/omarchy/config/waybar/style.css`
  - `root/omarchy/config/swayosd/style.css`
  - `root/omarchy/config/btop/btop.conf`
- Theme menu integration: `root/omarchy/default/elephant/omarchy_themes.lua`
- Theme library: `root/omarchy/themes/*`

## Core paths (installed system)
Omarchy installs into `~/.local/share/omarchy` and then symlinks/copies into `~/.config` for user-facing config.

**Theme store and current pointers**
- Default themes live at: `~/.local/share/omarchy/themes/`
- User theme directory: `~/.config/omarchy/themes/`
  - During install, each default theme is symlinked into this directory.
  - Additional themes (git clones) live here as real directories.
- Current theme symlink: `~/.config/omarchy/current/theme` → `~/.config/omarchy/themes/<theme>`
- Current background symlink: `~/.config/omarchy/current/background` → a file in `~/.config/omarchy/current/theme/backgrounds/`

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
Each theme is a directory under `~/.config/omarchy/themes/<theme-name>`.

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
- `neovim.lua`
- `swayosd.css`
- `vscode.json` (theme name + extension)
- `walker.css`
- `waybar.css`
- `backgrounds/` (image files)
- `preview.png` (for theme picker UI)

Optional files:
- `light.mode` (presence marks the theme as light; affects GNOME and Chromium color scheme)
- `obsidian.css` (if present, used verbatim for Obsidian instead of auto-generated CSS)

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
2. Validates that `~/.config/omarchy/themes/<theme>` exists.
3. Updates the current theme symlink: `~/.config/omarchy/current/theme`.
4. Rotates background via `omarchy-theme-bg-next`.
5. Restarts/reloads affected components:
   - Waybar (if running)
   - swayosd
   - terminal (touch Alacritty config, signal Kitty/Ghostty)
   - `hyprctl reload`
   - `pkill -SIGUSR2 btop`
   - `makoctl reload`
6. Applies app-specific theme changes:
   - GNOME (`omarchy-theme-set-gnome`)
   - Browser (`omarchy-theme-set-browser`)
   - VS Code (`omarchy-theme-set-vscode`)
   - Cursor (`omarchy-theme-set-cursor`)
   - Obsidian (`omarchy-theme-set-obsidian`)
7. Fires hook: `omarchy-hook theme-set <theme-name>`

### Background cycling: `omarchy-theme-bg-next`
- Looks in `~/.config/omarchy/current/theme/backgrounds/` for images.
- Updates symlink `~/.config/omarchy/current/background` to the next background.
- Relaunches `swaybg` (or uses a black background if none found).

### Theme list/current/next
- `omarchy-theme-list`: enumerates dirs/symlinks in `~/.config/omarchy/themes`, sorts, title-cases, and replaces `-` with space.
- `omarchy-theme-current`: reads the target of `~/.config/omarchy/current/theme`, title-cases it.
- `omarchy-theme-next`: cycles to the next theme in sorted order and calls `omarchy-theme-set`.

## Installing, updating, and removing themes
### Install: `omarchy-theme-install [git-url]`
- Clones a git repo into `~/.config/omarchy/themes/<theme-name>`.
- The theme name is derived from the repo name with:
  - `omarchy-` prefix removed
  - `-theme` suffix removed
- Automatically calls `omarchy-theme-set <theme>`.

### Update: `omarchy-theme-update`
- Iterates over `~/.config/omarchy/themes/*/`.
- Runs `git pull` **only** if:
  - directory is **not** a symlink, and
  - contains `.git`.

### Remove: `omarchy-theme-remove [theme]`
- If no name is given, it offers a selection of **extra themes** only (directories that are not symlinks).
- If the removed theme is current, it switches to the next theme first.
- Deletes the theme directory.

## App-specific theme setters
### GNOME
`omarchy-theme-set-gnome`:
- If `light.mode` exists, sets:
  - `org.gnome.desktop.interface color-scheme = prefer-light`
  - `gtk-theme = Adwaita`
- Otherwise, uses `prefer-dark` and `Adwaita-dark`.
- Icon theme from `icons.theme` (fallback: `Yaru-blue`).

### Chromium/Brave
`omarchy-theme-set-browser`:
- Reads `chromium.theme` (RGB) and converts to hex.
- Chromium:
  - `chromium --set-theme-color` with RGB string
  - `chromium --set-color-scheme` light/dark based on `light.mode`
  - Clears `/etc/chromium/policies/managed/color.json`
- Brave:
  - Writes `/etc/brave/policies/managed/color.json` with `BrowserThemeColor`
  - `brave --refresh-platform-policy`
- Installer pre-creates `/etc/{chromium,brave}/policies/managed` and makes them writable.

### VS Code and Cursor
`omarchy-theme-set-vscode [editor_cmd] [settings_path] [skip_flag] [editor_name]`:
- Reads `~/.config/omarchy/current/theme/vscode.json` for `{ name, extension }`.
- Ensures the extension is installed.
- Updates `workbench.colorTheme` in the editor’s `settings.json` (JSONC-safe, uses `sed`).
- If the theme file is missing, it removes `workbench.colorTheme`.
- Skippable via flags:
  - VS Code: `~/.local/state/omarchy/toggles/skip-vscode-theme-changes`
  - Cursor: `~/.local/state/omarchy/toggles/skip-cursor-theme-changes`

`omarchy-theme-set-cursor` simply calls `omarchy-theme-set-vscode` with Cursor’s settings path.

### Obsidian
`omarchy-theme-set-obsidian`:
- Maintains a vault registry at `~/.local/state/omarchy/obsidian-vaults`.
- Reads vault paths from `~/.config/obsidian/obsidian.json`.
- For each vault:
  - Ensures `.obsidian/themes/Omarchy/{manifest.json, theme.css}` exists.
  - If `obsidian.css` exists in the current theme, it is copied.
  - Otherwise, generates CSS by extracting and deriving colors from:
    - `alacritty.toml` (primary, normal, bright, dim, selection colors)
    - `waybar.css` (`@define-color` entries)
    - `hyprland.conf` (active/inactive border colors)
    - `btop.theme` (`theme[div_line]` for borders)
  - Also uses fonts from Alacritty and/or fontconfig.
- Supports `--reset` to remove Omarchy theme files and registry.

## Hooks
- `omarchy-hook <name> [args...]` executes `~/.config/omarchy/hooks/<name>` if present.
- Theme switching calls: `omarchy-hook theme-set <theme-name>`
- Sample hook: `~/.config/omarchy/hooks/theme-set.sample`

## Initial theme setup during install
`install/config/theme.sh`:
- Symlinks default themes into `~/.config/omarchy/themes/`.
- Sets initial theme to `tokyo-night` and a specific background image.
- Creates app-specific theme symlinks (btop, mako).
- Ensures browser policy directories exist and are writable.

## What an alternative theme manager must replicate
Minimum viable behavior to stay compatible with Omarchy’s ecosystem:
1. **Set the current theme symlink**: `~/.config/omarchy/current/theme` → `~/.config/omarchy/themes/<theme>`.
2. **Set/update background**: update `~/.config/omarchy/current/background` and restart `swaybg`.
3. **Reload/restart** these components:
   - Waybar, swayosd, hyprland (`hyprctl reload`), mako (`makoctl reload`), btop (`SIGUSR2`), terminals.
4. **Apply app-specific themes**: GNOME, browser, VS Code, Cursor, Obsidian.
5. **Run hook**: `omarchy-hook theme-set <theme-name>`.

If you want feature parity with Omarchy’s built-in tooling, also implement:
- Theme list/current/next behavior using `~/.config/omarchy/themes/`.
- Git-based install/update/remove semantics (symlinks = default themes, real dirs = user themes).
- Light theme detection by presence of `light.mode`.
