# RustUI Phase 1 Parity Map

This document captures the current CLI behavior and side effects to define the MVP parity target for the Rust rewrite on the `rustui` branch. It is derived from `src/theme-manager.sh`, `tests/theme_manager.bats`, and `Omarchy-theme-management.md`.

## Goals
- Match the existing CLI surface and exit codes for core commands.
- Preserve Omarchy compatibility sequencing (theme switch flow).
- Identify intentional simplifications for the Rust rewrite.
- Provide a checklist for porting tests to Rust-native suites.

## CLI Commands and Expected Behavior

### list
- Output: title-cased theme names, sorted by basename.
- Errors: missing theme root dir -> exit 1 with error message.

### set <theme> [flags]
- Normalizes theme name: strips HTML tags, lowercases, spaces to hyphens.
- Validates theme dir or symlink exists.
- Broken symlink -> exit 1 with "theme symlink is broken".
- Missing theme -> exit 1 with "theme not found".
- Side effects:
  - `~/.config/omarchy/current/theme` symlink updated to theme path.
  - `omarchy-theme-bg-next` called unless `THEME_MANAGER_SKIP_APPS=1`.
  - Reload components (Waybar, swayosd, terminals, hyprctl, mako, btop) unless skip.
  - Apply app-specific setters (GNOME/browser/vscode/cursor/obsidian) unless skip.
  - Run hook `~/.config/omarchy/hooks/theme-set` unless `THEME_MANAGER_SKIP_HOOK=1`.
  - Apply Waybar if requested or defaulted.
  - Apply Starship if requested or defaulted.

Flags:
- `-q/--quiet`: suppresses most external output.
- `-w/--waybar [name]`: sets Waybar mode to "auto" or "named".
- `--waybar=` with empty value -> exit 2 error.

### next
- Cycles to next theme in sorted list; wraps around.
- Requires at least one theme; else exit 1.
- Accepts `-q` and `-w` flags, no other args.

### current
- Prints title-cased current theme name.
- Errors if current theme symlink missing -> exit 1.

### bg-next
- Executes `omarchy-theme-bg-next`.
- Errors if command missing -> exit 1.

### browse
- Requires `fzf`, error if missing -> exit 1.
- Lists themes with previews; user selects theme, Waybar, Starship.
- Applies selected theme via `set`.
- Accepts `-q` only; any other args -> exit 2.

### print-config
- Prints resolved config values.
- Rejects extra args -> exit 2.

### version
- Prints CLI version.
- Rejects extra args -> exit 2.

### install <git-url>
- Derives theme name from repo (drop `omarchy-` prefix, `-theme` suffix).
- Clones repo into theme dir.
- Calls `set` on the installed theme.
- Errors: missing URL -> exit 2; git missing -> exit 1; theme exists -> exit 1.

### update
- Runs `git pull` for non-symlink themes with `.git`.
- Warns if no git-based themes found.
- Errors if theme root missing or git missing.

### remove [theme]
- Removes specified theme (normalize name) or prompts selection for removable themes.
- If removing current theme, switches to next theme first.
- Errors if only one theme exists.

## Config and Precedence (Current)
- Precedence: CLI flags > env vars > local config > user config > defaults.
- User config: `~/.config/theme-manager/config`
- Local config: `./.theme-manager.conf`
- Config format: `KEY=VALUE` (shell style) with allowed keys only.

## Environment Flags
- `THEME_MANAGER_SKIP_APPS=1` skips component reloads and setters.
- `THEME_MANAGER_SKIP_HOOK=1` skips theme-set hook.
- `QUIET_MODE=1` suppresses output for external commands.

## Waybar Behavior
- `WAYBAR_APPLY_MODE` default: `exec`.
- `exec` uses `tmplus-restart-waybar -c <config> -s <style>`.
- If exec helper missing, falls back to copy mode.
- `copy` mode copies config/style into `WAYBAR_DIR` and restarts Waybar.
- Theme preview search for Waybar uses case-insensitive PNG lookup.

## Starship Behavior
- Modes: preset, named theme, or theme-specific `starship.yaml`.
- Writes active config to `STARSHIP_CONFIG`.
- `browse` shows live prompt preview when `starship` exists.

## Browse Preview Rules (Theme)
- Preferred preview order:
  1) `preview.png` (case-insensitive) in theme root
  2) `theme.png` (case-insensitive) in theme root
  3) `waybar-theme/preview.png`
  4) First image in `backgrounds/` (png/jpg/jpeg/webp)
- Rendering uses `chafa` if available, else kitty `icat` when in kitty.

## Output Filtering (Current)
- `run_filtered` suppresses some known warnings and emits custom summaries.
- Rust rewrite will use best-practice messages with consistent exit codes.

## Known Simplifications Approved for Rust
- Simplify git update/remove semantics where safe (keep user experience clean).
- Allow improved wording while keeping exit codes consistent.
- Replace config format with TOML (migration to be defined).

## Test Porting Checklist
- Port each Bats test into Rust integration tests with temp fixtures:
  - usage/unknown command handling
  - list/current/next
  - set symlink update + broken symlink error
  - config precedence and unknown key warnings
  - waybar apply (copy vs exec)
  - starship preset/named errors and apply
  - install/update/remove paths
  - browse error when fzf missing

