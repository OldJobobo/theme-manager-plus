# Release Notes

## Unreleased

- Walker theme updates now only rewrite the `theme` key in `~/.config/walker/config.toml`.
- Walker auto mode now reliably replaces `theme-manager-auto` under `~/.config/walker/themes`.
- New Walker override flags are available across core flows:
  - `theme-manager set <theme> --walker [name]`
  - `theme-manager next --walker [name]`
  - `theme-manager preset save --walker <mode|name>`
  - `theme-manager preset load <name> --walker [name]`
- Walker apply paths now call `omarchy-restart-walker` so updates are visible immediately.
- Browse component tabs now provide explicit no-op choices (`No Waybar change`, `No Walker change`, `No Starship change`) that preserve each component’s current config.
- Omarchy defaults are now always selectable as named shared themes when available:
  - Waybar: `~/.config/waybar/themes/omarchy-default`
  - Walker: `~/.config/walker/themes/omarchy-default`
  - Starship: `~/.config/starship-themes/omarchy-default.toml`
- New Hyprlock support is now available with parity to other add-ons:
  - `theme-manager set/next --hyprlock [name]`
  - `theme-manager preset save --hyprlock <mode|name>`
  - `theme-manager preset load ... --hyprlock [name]`
  - `theme-manager hyprlock <mode>`
  - TUI `Hyprlock` tab including `No Hyprlock change`
  - Omarchy default auto-link at `~/.config/hypr/themes/hyprlock/omarchy-default`
- Hyprlock apply writes to `~/.config/omarchy/current/theme/hyprlock.conf` so it follows Omarchy’s lock-screen source chain.
- The CLI now warns if `~/.config/hypr/hyprlock.conf` does not source the current theme hyprlock file.
- Hyprlock compatibility updates:
  - Style-only Hyprlock themes now use the Omarchy wrapper host layout.
  - Full-layout Hyprlock themes now use minimal source-only host mode to avoid duplicate password/background widgets.
  - Custom host `~/.config/hypr/hyprlock.conf` is preserved when it does not source current theme (warning shown).
  - TUI only shows/injects `omarchy-default` when Omarchy default Hyprlock source is actually discoverable.
- Version display is now unified through a single repository `VERSION` file so Bash and Rust/TUI stay in sync.

## 0.2.9
- Theme-specific Starship configs now look for `starship.toml` (instead of `starship.yaml`).

## 0.2.8
- Reapplying the current theme from the TUI now reloads the full theme stack (including swayosd).
- Omarchy helper commands now auto-resolve from default installs or `$OMARCHY_PATH/bin`.
- SwayOSD restarts are more reliable with PID checks and a fallback relaunch.

## 0.2.7
- Added a “No theme change” option in the browse picker to apply Waybar/Starship without switching themes.
- New `theme-manager waybar` and `theme-manager starship` commands apply just those components.
- Broken theme symlinks now surface a clear error instead of failing later.

## 0.2.6
- Added configurable TUI apply shortcut (`[tui] apply_key`) for terminals like Ghostty.

## 0.2.5
- Removed the `omarchy-theme-set-cursor` call (Cursor is handled by Omarchy’s VS Code setter).

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
- Starship supports presets, named themes, and theme-provided configs (`starship.toml`).
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
