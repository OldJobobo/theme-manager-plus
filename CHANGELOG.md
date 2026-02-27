# Changelog

All notable changes to this project are documented in this file.

## Unreleased

- Removed legacy Bash-only implementation artifacts from the repository:
  - deleted `src/theme-manager.sh` and `bin/theme-manager`
  - deleted Bash config samples (`config.example`, `./.theme-manager.conf`)
  - deleted legacy Bats tests under `tests/`
- Updated repository docs and contributor guidelines to reflect Rust-only CLI/TUI workflows.
- Unified Omarchy default component resolution for Waybar/Walker/Hyprlock/Starship through a shared resolver:
  - standardized cross-module fallback precedence and validation checks
  - repaired stale `omarchy-default` symlinks automatically when targets drift
  - added regression coverage for precedence and missing-default behavior

## 0.3.0

- Hardened Walker config updates to change only the `theme` key (without clobbering keys like `theme_name`).
- Fixed Walker auto-theme cleanup to consistently replace `theme-manager-auto` in `~/.config/walker/themes`.
- Added Walker override flags for parity with Waybar:
  - `set <theme> --walker [name]`
  - `next --walker [name]`
  - `preset save --walker <mode|name>`
  - `preset load --walker [name]`
- Refactored shared none/auto/named parsing and command context construction in the Rust CLI to reduce drift.
- Updated README command reference to document Walker command and browse tab coverage.
- Added `omarchy-restart-walker` to the apply/reload pipeline so Walker changes take effect immediately.
- Browse component tabs now use explicit no-op selections (`No Waybar change`, `No Walker change`, `No Starship change`) that always leave the current component config untouched.
- Added Omarchy default component linking:
  - Waybar `omarchy-default` directory symlink in `~/.config/waybar/themes/`
  - Walker `omarchy-default` directory symlink in `~/.config/walker/themes/`
  - Starship `omarchy-default.toml` symlink in `~/.config/starship-themes/`
- Added Hyprlock module parity with Waybar/Walker:
  - `set/next/preset load` support `--hyprlock [name]`
  - `preset save --hyprlock <mode|name>`
  - `hyprlock <mode>` standalone command
  - TUI Hyprlock tab with `No Hyprlock change`, auto theme, and named shared themes
  - Omarchy default Hyprlock theme auto-linking to `~/.config/hypr/themes/hyprlock/omarchy-default`
- Hyprlock apply now targets `~/.config/omarchy/current/theme/hyprlock.conf` (Omarchy source-chain compatible) instead of writing into `~/.config/hypr/`.
- Added warning when `~/.config/hypr/hyprlock.conf` does not source the current theme hyprlock path.
- Hyprlock host-config compatibility hardening:
  - Style-only Hyprlock themes now restore/use the Omarchy wrapper host config.
  - Full-layout Hyprlock themes switch host config to minimal source-only mode to prevent duplicate UI widgets.
  - Custom host `~/.config/hypr/hyprlock.conf` that does not source current theme is preserved with an explicit warning.
  - TUI only injects `omarchy-default` when a real Omarchy default source is discoverable.
- Unified version display onto a single source: repository `VERSION` file now drives Rust CLI/TUI version output and Bash CLI version output.

## 0.2.9
- Theme-specific Starship configs now use `starship.toml` instead of `starship.yaml`.

## 0.2.8
- Applying the current theme from the TUI now reloads the full theme stack (including swayosd and theme setters).
- Auto-detect Omarchy’s bin directory (default path or `$OMARCHY_PATH/bin`) to ensure helper commands resolve.
- Hardened swayosd restarts with PID checks and a fallback relaunch path.

## 0.2.7
- Added a "No theme change" option in the browse picker so Waybar/Starship can be applied without switching themes.
- Added `waybar` and `starship` commands to apply those components without changing themes.
- Improved error handling for broken theme symlinks.

## 0.2.6
- Added configurable TUI apply shortcut (`[tui] apply_key`) for terminals like Ghostty.

## 0.2.5
- Dropped the call to `omarchy-theme-set-cursor` (handled by `omarchy-theme-set-vscode`).

## 0.2.4
- Theme listing and selection now include both Omarchy default and user themes.

## 0.2.3
- Fixed `install.sh` to handle piped execution without `BASH_SOURCE` errors.
- Waybar symlinks now point at the selected theme directory instead of `current/theme`.

## 0.2.2
- Always symlink Waybar config/style unless `WAYBAR_APPLY_MODE="copy"` is set.

## 0.2.1
- Default to the browse picker when running `theme-manager` with no arguments.

## 0.2.0
- Added install/uninstall scripts for release binaries with a source-build fallback.

## 0.1.9
- Reset list selection to the top when advancing to the next tab with Enter.

## 0.1.8
- Limited held-key navigation to one list move per poll cycle to avoid multi-row jumps.

## 0.1.7
- Added tabbed browse workflow with review/apply step, mouse focus, and improved key handling.
- Added fuzzy search in the browse picker lists (Theme/Waybar/Starship) with an inline search field.
- Added preset save/load/list/remove commands with TOML storage and a new Presets tab in the TUI.
- Added Review-screen preset save shortcut (Ctrl+S) and a single-line status bar in the TUI.
- Added new awww transition presets and expanded awww transition configuration options.
- Improved preview rendering and clearing behavior in the TUI.
- Added awww-backed wallpaper transitions with theme background cycling, daemon auto-start, and debug logging support.
- Added built-in Waybar exec restart logic in the Rust binary (no external helper required).
- Added `waybar.restart_logs` config to control Waybar restart log output (default: silenced).
- Default Waybar apply mode is now symlink with backups for existing non-symlink Waybar paths.
- Falling back to symlink mode when Waybar styles import `../omarchy/current/theme/waybar.css` (exec mode).
- Deferred app restarts until after theme, Waybar, and Starship changes are applied.
- Reworked browse TUI layout with syntax-highlighted config previews and dedicated image/prompt preview panes.
- Improved Starship preview rendering and placement in the browse picker.
- Materialized current theme directories in the Rust binary and invoked `omarchy-theme-set-templates` to render `colors.toml` outputs.
- Documented the Rust binary’s template rendering and current theme materialization behavior.
- Updated config defaults/docs for new awww behavior and slower transition timing.
- Updated docs to reflect Omarchy’s current theme flow (materialized current theme directory, template generation, and theme.name usage).
- Fixed browse picker key repeat buffering by draining queued key events while held keys repeat.

## 0.1.5
- Improved README flow and Starship documentation details.
- Added bundled Starship theme examples under `extras/starship-themes/`.
- Added `extras/omarchy/tmplus-restart-waybar` helper for restarting Waybar with custom config/style paths.
- Added support for theme-specific `starship.toml` files in the Starship picker.
- Added configurable Waybar apply mode with optional exec restart support.
- Switched default Waybar apply mode to exec.
- Fixed kitty terminal image previews stuck on "Loading..." by adding newline padding and combining clear+display commands.
- Fixed waybar preview images not updating when navigating between themes by removing premature exit.
- Fixed waybar preview image detection for symlinked themes by adding `-L` flag to follow symlinks.
- Added case-insensitive PNG detection for waybar previews (`*.png` and `*.PNG`).
- Added `theme.png` fallback support for Omarchy theme previews (in addition to `preview.png`).
- Added live Starship prompt preview in browse picker showing actual rendered prompt appearance with colors.
- Clarified Waybar preview fallback behavior and config precedence in the docs.
- Switched documentation examples to placeholder theme names.

## 0.1.4
- Added validation for empty `--waybar=` usage and improved preview file selection logic.
- Expanded Bats coverage for error paths, Waybar selection, and update/removal edge cases.
- Documented changelog workflow in repository guidelines.
- Added Starship integration with preset and user theme support, plus browse selection.
- Added Starship configuration keys, install directory creation, and tests.
