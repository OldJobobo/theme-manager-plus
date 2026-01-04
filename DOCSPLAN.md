# Documentation Plan

## 1) Overview
- Purpose: alternative manager that triggers Omarchy’s theme switch flow (menu parity).
- Non-goals: not replacing Omarchy’s theming system.
- High-level behavior: set theme symlink, run Omarchy tools, apply optional Waybar themes.
- Writing style: plain English, simple terms, minimal jargon; short sentences; explain “why” when helpful.
- Examples: avoid real theme names; use placeholders like `<ThemeName>`.

## 2) Quick Start
- Prereqs: Omarchy installed, PATH includes Omarchy scripts, kitty/chafa optional for previews.
- Basic commands:
  - `./bin/theme-manager list`
  - `./bin/theme-manager set <theme>`
  - `./bin/theme-manager browse`
- Example usage flows:
  - `./bin/theme-manager set <ThemeName> -w`
  - `./bin/theme-manager set <ThemeName> -w default`
  - `./bin/theme-manager set <ThemeName> -w -q`

## 3) Command Reference
- `set <theme> [-w/--waybar [name]] [-q/--quiet]`
- `next`, `current`, `bg-next`
- `browse`
- `install`, `update`, `remove`
- Exit codes and common failure messages.

## 4) Waybar Integration
- Theme folder conventions:
  - per-theme: `waybar-theme/config.jsonc` + `style.css`
  - shared: `~/.config/waybar/themes/<name>/`
- Behavior of `-w`:
  - auto (use theme’s `waybar-theme` if present)
  - named (use `~/.config/waybar/themes/<name>`)
- Copy behavior (no backups).
- Restart behavior (`omarchy-restart-waybar`, plus optional `tmplus-restart-waybar` helper).
- Preview selection: `waybar-theme/preview.png` (case-insensitive), `theme.png` fallback, then first image in `backgrounds/`.

## 5) Omarchy Compatibility
- Omarchy tools invoked:
  - `omarchy-theme-bg-next`
  - `omarchy-restart-terminal`, `omarchy-restart-waybar`, `omarchy-restart-swayosd`
  - `extras/omarchy/tmplus-restart-waybar` for Waybar restarts with `-c/-s` flags
  - `omarchy-theme-set-gnome`, `omarchy-theme-set-browser`, `omarchy-theme-set-vscode`, `omarchy-theme-set-cursor`, `omarchy-theme-set-obsidian`
  - `omarchy-hook theme-set`
- Parity gaps (if any) and rationale.

## 6) Configuration & Environment
- Flags: `-w`, `-q`.
- Env vars:
  - `THEME_MANAGER_SKIP_APPS`
  - `THEME_MANAGER_SKIP_HOOK`
- Directory layout: `~/.config/omarchy`, `~/.config/waybar`.
- Config format: TOML (`config.toml.example`).

## 7) Troubleshooting
- Common warnings and what they mean.
- Diagnosing failed theme apply.
- Quiet vs verbose behavior.

## 8) Testing
- Rust tests: `cargo test` in `rust/`.
- Legacy Bats tests (Bash CLI): `./tests/run.sh`.
- Adding fixtures and new tests.

## 9) Development Guide
- Project structure (`bin/`, `src/`, `tests/`).
- Coding style.
- Contribution workflow.
- How to add new add-ons (future Waybar or other extensions).

## 10) FAQ
- Why not call `omarchy-theme-set` directly?
- Why copy Waybar configs?
- Can I use custom theme paths?
