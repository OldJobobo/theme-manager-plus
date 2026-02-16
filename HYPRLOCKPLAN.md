# Hyprlock Module Implementation Plan

This plan adds Hyprlock as a first-class optional component with parity to Waybar/Walker/Starship.

## Phase 1: Core Config and Mode Plumbing
- [x] Add Hyprlock config schema and resolved paths/modes in `rust/src/config.rs`.
- [x] Add `HyprlockMode` and defaults mapping in `rust/src/theme_ops.rs`.
- [x] Extend `CommandContext` with hyprlock mode/name.
- [x] Wire hyprlock into context assembly in `rust/src/lib.rs`.

### Exit Criteria
- [x] `theme-manager` can parse hyprlock config defaults without runtime errors.
- [x] Theme set flow receives hyprlock mode/name through `CommandContext`.

## Phase 2: Hyprlock Apply Module
- [x] Add `rust/src/hyprlock.rs`.
- [x] Implement `prepare_hyprlock(ctx, theme_dir)` for `none|auto|named`.
- [x] Support `copy|symlink` apply modes.
- [x] Add `ensure_omarchy_default_theme_link` for `omarchy-default`.
- [x] Validate required `hyprlock.conf` presence.

### Exit Criteria
- [x] Named mode applies shared theme config from `~/.config/hyprlock/themes/<name>/`.
- [x] Auto mode applies theme-bundled `hyprlock-theme/`.
- [x] `none` leaves current hyprlock config untouched.
- [x] Omarchy default symlink is created when source exists.

## Phase 3: CLI and Preset Parity
- [x] Add `hyprlock` standalone subcommand.
- [x] Add `--hyprlock` overrides to `set`, `next`, and `preset load`.
- [x] Add `preset save --hyprlock <mode|name>`.
- [x] Extend preset schema and load/save mapping for hyprlock.

### Exit Criteria
- [x] `theme-manager set THEME --hyprlock`
- [x] `theme-manager set THEME --hyprlock NAME`
- [x] `theme-manager next --hyprlock ...`
- [x] `theme-manager preset save --hyprlock ...`
- [x] `theme-manager preset load ... --hyprlock ...`

## Phase 4: TUI Integration
- [x] Add Hyprlock tab in browse flow.
- [x] Add list items:
  - [x] `No Hyprlock change`
  - [x] `Use theme hyprlock` (if present)
  - [x] named shared themes
- [x] Add hyprlock preview rendering.
- [x] Include hyprlock in review summary and preset save/load mapping.

### Exit Criteria
- [x] TUI can select/apply hyprlock independently.
- [x] `No Hyprlock change` is explicit no-op.

## Phase 5: Tests and Docs
- [x] Add `rust/tests/cli_hyprlock.rs`.
- [x] Add/update preset/theme-op tests for hyprlock override behavior.
- [x] Ensure `cargo test` passes.
- [x] Update `README.md`, `CHANGELOG.md`, `RELEASE_NOTES.md`, and `config.toml.example`.

### Exit Criteria
- [x] Hyprlock test coverage exists for named/auto/none/default-link paths.
- [x] Docs reflect new command surface and behavior.
- [x] All tests pass or unrelated failures are explicitly called out.

## Execution Notes
- Keep Omarchy-compatible theme flow unchanged.
- Keep no-op semantics explicit (`No Hyprlock change` leaves config as-is).
- Avoid GUI side effects in tests by using existing test stubs and env isolation.
