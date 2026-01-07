# Changelog

All notable changes to this project are documented in this file.

## Unreleased
- Added tabbed browse workflow with review/apply step, mouse focus, and improved key handling.
- Added fuzzy search in the browse picker lists (Theme/Waybar/Starship) with `/` to search.
- Added preset save/load/list/remove commands with TOML storage and a new Presets tab in the TUI.
- Added Review-screen preset save shortcut (Ctrl+S) and a single-line status bar in the TUI.
- Added new awww transition presets and expanded awww transition configuration options.
- Improved preview rendering and clearing behavior in the TUI.
- Added awww-backed wallpaper transitions with theme background cycling, daemon auto-start, and debug logging support.
- Added built-in Waybar exec restart logic in the Rust binary (no external helper required).
- Falling back to copy mode when Waybar styles import `../omarchy/current/theme/waybar.css` (exec mode).
- Deferred app restarts until after theme, Waybar, and Starship changes are applied.
- Reworked browse TUI layout with syntax-highlighted config previews and dedicated image/prompt preview panes.
- Improved Starship preview rendering and placement in the browse picker.
- Updated config defaults/docs for new awww behavior and slower transition timing.

## 0.1.5
- Improved README flow and Starship documentation details.
- Added bundled Starship theme examples under `extras/starship-themes/`.
- Added `extras/omarchy/tmplus-restart-waybar` helper for restarting Waybar with custom config/style paths.
- Added support for theme-specific `starship.yaml` files in the Starship picker.
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
