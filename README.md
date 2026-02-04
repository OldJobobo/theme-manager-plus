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
  - A Starship preset or user theme
- Supports **presets** (theme + Waybar + Starship bundles)

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
- `theme-manager browse` — interactive selector (theme + Waybar + Starship)
- `theme-manager waybar <mode>` — apply Waybar only
- `theme-manager starship <mode>` — apply Starship only
- `theme-manager preset save|load|list|remove`
- `theme-manager version`

---

## Available Themes

Below is a list of custom themes available for Omarchy. Install any theme using:

```sh
theme-manager install https://github.com/OldJobobo/<theme-name>
```

### Theme List

- **[Batman](https://github.com/OldJobobo/omarchy-batman-theme)** - Dark theme inspired by the Dark Knight
- **[Caroline Skyline](https://github.com/OldJobobo/omarchy-caroline-skyline-theme)** - Urban skyline-inspired colorscheme
- **[City 783](https://github.com/OldJobobo/omarchy-city-783-theme)** - Cyberpunk city aesthetic
- **[Deckard](https://github.com/OldJobobo/omarchy-deckard-theme)** - Blade Runner-inspired theme
- **[Dracula](https://github.com/OldJobobo/omarchy-dracula-theme)** - Classic Dracula color scheme
- **[Eldritch](https://github.com/OldJobobo/omarchy-eldritch-theme)** - Lovecraftian horror-inspired dark theme
- **[Event Horizon](https://github.com/OldJobobo/omarchy-event-horizon-theme)** - Dark space-themed colorscheme
- **[Flat Dracula](https://github.com/OldJobobo/omarchy-flat-dracula-theme)** - Flattened variant of Dracula
- **[Florida Man](https://github.com/OldJobobo/omarchy-florida-man-theme)** - Vibrant Florida-inspired colors
- **[Grimdark Solarized](https://github.com/OldJobobo/omarchy-grimdark-solarized-theme)** - Dark take on Solarized
- **[Hex](https://github.com/OldJobobo/omarchy-hex-theme)** - Hexagonal geometric theme
- **[Hinterlands](https://github.com/OldJobobo/omarchy-hinterlands-theme)** - Nature-inspired wilderness theme
- **[Miasma](https://github.com/OldJobobo/omarchy-miasma-theme)** - Foggy, atmospheric colorscheme
- **[Monolith](https://github.com/OldJobobo/omarchy-monolith-theme)** - Minimalist monochrome theme
- **[Phosphor OS](https://github.com/OldJobobo/omarchy-phosphor-os-theme)** - Retro phosphor terminal aesthetic
- **[The Loop](https://github.com/OldJobobo/omarchy-the-loop-theme)** - Continuous loop-inspired design
- **[Waffle Cat](https://github.com/OldJobobo/omarchy-waffle-cat-theme)** - Warm forward color scheme
- **[X-1632](https://github.com/OldJobobo/omarchy-x-1632-theme)** - Futuristic experimental theme

### Quick Install Examples

```sh
# Install Event Horizon theme
theme-manager install https://github.com/OldJobobo/omarchy-event-horizon-theme

# Install Phosphor OS theme
theme-manager install https://github.com/OldJobobo/omarchy-phosphor-os-theme

# Install and apply with Waybar
theme-manager install https://github.com/OldJobobo/omarchy-dracula-theme
theme-manager set Dracula -w
```

---

## Command Reference (Short)

### `set <theme> [-w|--waybar [name]] [-q|--quiet]`

Switch themes.

- `-w` (no name): use the theme’s `waybar-theme/` if present
- `-w <name>`: use `~/.config/waybar/themes/<name>/`
- `-q`: suppress external command output

---

### `browse`

Full-screen selector with previews.

- Tabs: **Theme**, **Waybar**, **Starship**, **Presets**, **Review**
- Apply with **Ctrl+Enter** by default
- Includes a **“No theme change”** option
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

Presets store a **theme + Waybar + Starship** bundle.

Save example:
```sh
theme-manager preset save "Daily Driver" \
  --theme noir \
  --waybar auto \
  --starship preset:bracketed-segmented
```

Load example:
```sh
theme-manager preset load "Daily Driver" -w
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

---

## Starship Integration

Supported sources:
- Starship presets
- User themes: `~/.config/starship-themes/*.toml`
- Theme-specific: `starship.yaml`

Behavior:
- Active config is written to `~/.config/starship.toml`
- Presets appear automatically in browse mode
- Example themes live in `extras/starship-themes/`

---

## Omarchy Compatibility

Theme Manager Plus **calls Omarchy’s own scripts** to stay compatible.

Scripts invoked include:
- `omarchy-theme-bg-next`
- `omarchy-restart-terminal`
- `omarchy-restart-waybar`
- `omarchy-restart-swayosd`
- `omarchy-theme-set-*`
- `omarchy-hook theme-set`

### Order of operations (simplified)

1. Materialize theme and write `theme.name`
2. Apply Waybar / Starship (if selected)
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
