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

## FAQ

**Why not replace Omarchy’s theming?**  
Because Omarchy owns the system; this tool just drives it.

**Does browse require fzf?**  
No. The Rust TUI replaces it entirely.
