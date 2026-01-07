# Release Notes

## 0.2.0
Theme Manager Plus now ships a full Rust CLI and a richer TUI experience, including presets and improved previews.

Highlights
- Rust CLI with TOML config support and parity for theme switching, Waybar, and Starship actions.
- New browse TUI with tabbed navigation, review/apply flow, fuzzy search, image previews, and prompt previews.
- Presets: save/load/list/remove named bundles for theme, Waybar, and Starship selections.
- Review screen shortcut (Ctrl+S) to save presets, plus a single-line status bar with active selections.

Details
- Waybar exec restart built into the Rust binary, with automatic fallback to copy mode when needed.
- Starship supports presets, named themes, and theme-provided configs (`starship.yaml`).
- Awww-powered background transitions with additional transition options (used only when `awww` and its daemon are available; not auto-started).
