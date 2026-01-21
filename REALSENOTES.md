# Release Notes

## Unreleased

## 0.2.4
- Theme picker now shows both Omarchy default and user themes.

## 0.2.3
- Install script works when run via `curl | bash`.
- Waybar symlinks now target the selected theme directory (avoids broken links after Omarchy switches).

## 0.2.2
Waybar apply now symlinks by default; set `WAYBAR_APPLY_MODE="copy"` to copy instead.

## 0.2.1
Running `theme-manager` with no arguments now opens the browse picker.

## 0.2.0
Theme Manager Plus now ships a full Rust CLI and a richer TUI experience, including presets and improved previews.

Highlights
- Rust CLI with TOML config support and parity for theme switching, Waybar, and Starship actions.
- New browse TUI with tabbed navigation, review/apply flow, fuzzy search, image previews, and prompt previews.
- Presets: save/load/list/remove named bundles for theme, Waybar, and Starship selections.
- Review screen shortcut (Ctrl+S) to save presets, plus a single-line status bar with active selections.
- Release install now uses prebuilt binaries with a source-build fallback, plus an uninstall script.

Details
- Waybar exec restart built into the Rust binary, with automatic fallback to symlink mode when needed.
- Starship supports presets, named themes, and theme-provided configs (`starship.yaml`).
- Awww-powered background transitions with additional transition options (used only when `awww` and its daemon are available; not auto-started).

## 0.1.9
Pressing Enter to advance tabs now resets the next list to the top selection.

## 0.1.8
Browse navigation now moves one row per update while holding keys, preventing multi-row jumps.

## 0.1.7
Theme switching now matches Omarchy’s current theme format when using the Rust CLI.

Highlights
- Waybar apply now uses symlinks by default and backs up existing Waybar paths under the themes directory.
- Rust CLI materializes `~/.config/omarchy/current/theme` and runs `omarchy-theme-set-templates` for `colors.toml` output generation.
- Browse TUI no longer buffers huge key repeat bursts when holding navigation keys.
