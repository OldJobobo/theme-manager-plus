# Theme Manager Plus

## Overview

**Theme Manager Plus** is a TUI and CLI tool that switches Omarchy themes *exactly the same way the Omarchy menu does*, with additional flexibility and automation.

It is **not a replacement** for Omarchy’s theming system.  
Think of it as a **direct, expanded interface** for driving Omarchy’s existing theme flow—script-compatible, hook-compatible, and future-proof.

### What it does

- Materializes `~/.config/omarchy/current/theme` and writes `theme.name` so Omarchy apps know which theme is active
- Runs Omarchy’s own theme scripts to keep behavior identical to the menu
- Reloads common components (Waybar, terminals, notifications, etc.)
- Optionally applies:
  - A Waybar theme
  - A Walker theme
  - A Hyprlock theme
  - A Starship preset or user theme
- Supports **presets** (theme + Waybar + Walker + Hyprlock + Starship bundles)

---

## Quick Start

```sh
curl -fsSL https://raw.githubusercontent.com/OldJobobo/theme-manager-plus/master/install.sh | bash
theme-manager
```

If the command is not found, open a new terminal or run:

```sh
source ~/.profile
```

---

## Requirements

- Omarchy installed on this machine
- Omarchy scripts available in `PATH`
  - or configured via `OMARCHY_BIN_DIR`
- Optional:
  - `starship` (for Starship presets or themes)
  - `kitty` or `chafa` (for browse previews)
  - `awww` (for wallpaper transitions; daemon is **not** auto-started)

---

## Installation

**Install latest (Linux x86_64):**
```sh
curl -fsSL https://raw.githubusercontent.com/OldJobobo/theme-manager-plus/master/install.sh | bash
```

**Install a specific version:**
```sh
THEME_MANAGER_VERSION=0.2.6 \
  curl -fsSL https://raw.githubusercontent.com/OldJobobo/theme-manager-plus/master/install.sh | bash
```

**Uninstall:**
```sh
curl -fsSL https://raw.githubusercontent.com/OldJobobo/theme-manager-plus/master/uninstall.sh | bash
```

**Uninstall and remove config:**
```sh
curl -fsSL https://raw.githubusercontent.com/OldJobobo/theme-manager-plus/master/uninstall.sh | bash -s -- --purge
```

---

## Common Commands

- `theme-manager` — open the full-screen browser (default)
- `theme-manager list` — list available themes
- `theme-manager set <Theme>` — switch to a theme
- `theme-manager set <Theme> -w` — switch theme and apply Waybar
- `theme-manager set <Theme> -k` — switch theme and apply bundled Walker theme
- `theme-manager set <Theme> --hyprlock` — switch theme and apply bundled Hyprlock theme
- `theme-manager browse` — interactive selector (theme + Waybar + Walker + Hyprlock + Starship)
- `theme-manager waybar <mode>` — apply Waybar only
- `theme-manager walker <mode>` — apply Walker only
- `theme-manager hyprlock <mode>` — apply Hyprlock only
- `theme-manager starship <mode>` — apply Starship only
- `theme-manager preset save|load|list|remove`
- `theme-manager version`

---

## Command Reference (Short)

### `set <theme> [-w|--waybar [name]] [-k|--walker [name]] [--hyprlock [name]] [-q|--quiet]`

Switch themes.

- `-w` (no name): use the theme’s `waybar-theme/` if present
- `-w <name>`: use `~/.config/waybar/themes/<name>/`
- `-k` (no name): use the theme’s `walker-theme/` if present
- `-k <name>`: use `~/.config/walker/themes/<name>/`
- `--hyprlock` (no name): use the theme’s `hyprlock-theme/` if present
- `--hyprlock <name>`: use `~/.config/hypr/themes/hyprlock/<name>/`
- `-q`: suppress external command output

---

### `browse`

Full-screen selector with previews.

- Tabs: **Theme**, **Waybar**, **Walker**, **Hyprlock**, **Starship**, **Presets**, **Review**
- Apply with **Ctrl+Enter** by default
- Includes a **“No theme change”** option
- Component tabs include **“No Waybar change”**, **“No Walker change”**, **“No Hyprlock change”**, and **“No Starship change”** (leave current config as-is)
- Supports search and preset saving

---

### `next` / `current` / `bg-next`

- `next`: cycle to the next theme
- `current`: print current theme name
- `bg-next`: cycle background via Omarchy

---

### `install <git-url>` / `update` / `remove [theme]`

- `install`: clone and activate a theme
- `update`: pull updates for git-based themes
- `remove`: delete a theme directory

---

### `preset save|load|list|remove`

Presets store a **theme + Waybar + Walker + Hyprlock + Starship** bundle.

Save example:
```sh
theme-manager preset save "Daily Driver" \
  --theme noir \
  --waybar auto \
  --walker auto \
  --hyprlock auto \
  --starship preset:bracketed-segmented
```

Load example:
```sh
theme-manager preset load "Daily Driver" -w
# or override Walker too:
theme-manager preset load "Daily Driver" -w -k omarchy-default
# or override Hyprlock:
theme-manager preset load "Daily Driver" --hyprlock omarchy-default
```

**Precedence:**  
CLI flags > preset values > config defaults

---

### `waybar <mode>`

Apply Waybar without changing the theme.

Modes:
- `auto`
- `none`
- `<name>` (shared Waybar theme)

---

### `starship <mode>`

Apply Starship without changing the theme.

Modes:
- `none`
- `theme`
- `preset:<name>`
- `named:<name>`
- `<name>` (named theme if it exists, otherwise preset)

---

### `walker <mode>`

Apply Walker without changing the theme.

Modes:
- `auto`
- `none`
- `<name>` (shared Walker theme)

---

### `hyprlock <mode>`

Apply Hyprlock without changing the theme.

Modes:
- `auto`
- `none`
- `<name>` (shared Hyprlock theme)

---

### `print-config`

Print resolved configuration values.

---

### `version`

Print CLI version.

---

## Browse Mode Details

### Previews

- `preview.png` (preferred)
- `theme.png`
- First image in `backgrounds/`

All checks are case-insensitive.

### Keybindings

- Apply: `Ctrl+Enter` (default)
- Save preset: `Ctrl+S`
- Clear search: `Ctrl+U`

### Ghostty users

Change apply key:
```toml
[tui]
apply_key = "ctrl+m"
```

Or unbind in Ghostty:
```ini
keybind = ctrl+enter=unbound
```

Restart Ghostty after changes.

---

## Waybar Integration

Two supported layouts:

**Per-theme**
```
theme/
└── waybar-theme/
    ├── config.jsonc
    └── style.css
```

**Shared**
```
~/.config/waybar/themes/<name>/
```

Behavior:
- Files are symlinked into `~/.config/waybar/` by default
- Set `WAYBAR_APPLY_MODE="copy"` to copy instead
- Waybar is restarted after apply
- If Omarchy default Waybar files are found, `omarchy-default` is auto-linked into `~/.config/waybar/themes/`

---

## Walker Integration

Supported sources:
- Theme-specific: `walker-theme/` (requires `style.css`, optional `layout.xml`)
- Shared themes: `~/.config/walker/themes/<name>/`

Behavior:
- Named Walker mode updates `~/.config/walker/config.toml` (`theme = "..."`)
- Auto mode builds `theme-manager-auto` under `~/.config/walker/themes/`
- Walker is restarted after apply
- If Omarchy default Walker files are found, `omarchy-default` is auto-linked into `~/.config/walker/themes/`

---

## Starship Integration

Supported sources:
- Starship presets
- User themes: `~/.config/starship-themes/*.toml`
- Theme-specific: `starship.toml`

Behavior:
- Active config is written to `~/.config/starship.toml`
- Presets appear automatically in browse mode
- Example themes live in `extras/starship-themes/`
- If Omarchy default Starship files are found, `omarchy-default.toml` is auto-linked into `~/.config/starship-themes/`

---

## Hyprlock Integration

Supported layouts:
- Theme-specific: `hyprlock-theme/hyprlock.conf`
- Shared: `~/.config/hypr/themes/hyprlock/<name>/hyprlock.conf`

Behavior:
- Applied to `~/.config/omarchy/current/theme/hyprlock.conf` by symlink by default (`copy` via config/env)
- Expects `~/.config/hypr/hyprlock.conf` to source `~/.config/omarchy/current/theme/hyprlock.conf`
- `No Hyprlock change` leaves current Hyprlock config untouched
- Host `~/.config/hypr/hyprlock.conf` handling is automatic:
  - Style-only Hyprlock themes keep/restore the Omarchy wrapper layout.
  - Full-layout Hyprlock themes use a minimal source-only host config to avoid duplicate widgets.
  - If host config is custom and does not source current theme, it is preserved and a warning is printed.
- If Omarchy default Hyprlock files are found, `omarchy-default` is auto-linked into `~/.config/hypr/themes/hyprlock/` and shown in TUI.

---

## Omarchy Compatibility

Theme Manager Plus **calls Omarchy’s own scripts** to stay compatible.

Scripts invoked include:
- `omarchy-theme-bg-next`
- `omarchy-restart-terminal`
- `omarchy-restart-waybar`
- `omarchy-restart-walker`
- `omarchy-restart-swayosd`
- `omarchy-theme-set-*`
- `omarchy-hook theme-set`

### Order of operations (simplified)

1. Materialize theme and write `theme.name`
2. Apply Waybar / Walker / Hyprlock / Starship (if selected)
3. Update background
4. Reload components
5. Run Omarchy app setters
6. Trigger Omarchy theme hook

Supports Omarchy templates via:
- `$OMARCHY_PATH/default/themed`
- `~/.config/omarchy/themed` (user overrides)

---

## Configuration

Configuration precedence:
1. CLI flags
2. Environment variables
3. `./.theme-manager.toml`
4. `~/.config/theme-manager/config.toml`
5. Defaults

Example (`awww` transitions):
```toml
[behavior]
awww_transition = true
awww_transition_type = "grow"
awww_transition_duration = 2.4
awww_transition_fps = 60
```

Presets are stored in:
```
~/.config/theme-manager/presets.toml
```

---

## Troubleshooting

- **Theme not found** → check spelling or `THEME_ROOT_DIR`
- **Omarchy scripts missing** → ensure they are in `PATH`
- **Waybar not changing** → verify `waybar-theme/` contents
- **Missing previews** → check `preview.png`, `theme.png`, or `backgrounds/`
- **GTK / browser warnings** → usually harmless; use `-q`

---

## Development Notes

- Rust CLI entry: `rust/src/main.rs`
- Legacy Bash CLI: `bin/theme-manager`
- Rust tests: `rust/tests/`
- Bats tests: `tests/`

Run tests:
```sh
cd rust
cargo test
```

---

## FAQ

**Why not replace Omarchy’s theming?**  
Because Omarchy owns the system; this tool just drives it.

**Why symlink Waybar files?**  
To preserve Omarchy’s expected paths and imports.

**Can I use custom theme paths?**  
Yes—configure `THEME_ROOT_DIR`.

**Does browse require fzf?**  
No. The Rust TUI replaces it entirely.
