# Omarchy Default Resolution Audit (Phase 1)

## Purpose
Capture **current implemented behavior** for Omarchy default loading across module tabs and CLI flows.

## Root Detection (Shared)
Current Omarchy root detection is centralized in `detect_omarchy_root`:
1. `OMARCHY_PATH`
2. `config.omarchy_bin_dir` parent
3. `$HOME/.local/share/omarchy`

Reference: [omarchy.rs](/home/oldjobobo/Projects/rust/theme-manager-plus/rust/src/omarchy.rs:28)

## Current Resolver Matrix (As Implemented)

| Module | Resolver Function | Candidate Order (first hit wins) | Validation |
|---|---|---|---|
| Waybar | `omarchy_default_waybar_theme_dir` | `default/waybar/themes/omarchy-default` -> `default/waybar` | Must contain both `config.jsonc` and `style.css` |
| Walker | `omarchy_default_walker_theme_dir` | `default/walker/themes/omarchy-default` | Directory exists only |
| Hyprlock | `omarchy_default_hyprlock_theme_dir` | `default/hyprlock/themes/omarchy-default` -> `default/hyprlock` -> `themes/omarchy-default` -> `config/hypr` -> `$HOME/.config/omarchy/default/hyprlock/themes/omarchy-default` -> `$HOME/.config/omarchy/default/hyprlock` -> `$HOME/.config/omarchy/themes/omarchy-default` -> `$HOME/.config/omarchy/config/hypr` | Must contain `hyprlock.conf` |
| Starship | `omarchy_default_starship_theme_file` | `default/starship/themes/omarchy-default.toml` -> `default/starship.toml` -> `default/starship/starship.toml` | File exists |

References:
- [waybar.rs](/home/oldjobobo/Projects/rust/theme-manager-plus/rust/src/waybar.rs:89)
- [walker.rs](/home/oldjobobo/Projects/rust/theme-manager-plus/rust/src/walker.rs:100)
- [hyprlock.rs](/home/oldjobobo/Projects/rust/theme-manager-plus/rust/src/hyprlock.rs:183)
- [starship.rs](/home/oldjobobo/Projects/rust/theme-manager-plus/rust/src/starship.rs:130)

## Where Defaults Are Consumed
- CLI default mode parsing uses `*_from_defaults` when flags are omitted.
  - Reference: [theme_ops.rs](/home/oldjobobo/Projects/rust/theme-manager-plus/rust/src/theme_ops.rs:63)
  - Reference: [lib.rs](/home/oldjobobo/Projects/rust/theme-manager-plus/rust/src/lib.rs:260)
- TUI tab builders call `ensure_omarchy_default_theme_link` for each module before listing options.
  - Reference: [tui.rs](/home/oldjobobo/Projects/rust/theme-manager-plus/rust/src/tui.rs:1151)

## Divergences / Risks Identified
1. Inconsistent fallback breadth across modules.
- Hyprlock supports many layouts (including config-space fallbacks), while Walker is strict (single path), and Waybar/Starship are mid-range.

2. Hyprlock tab behavior is special-cased; others are not.
- Hyprlock appends `omarchy-default` in TUI if `omarchy_default_theme_available()` is true even when the link is absent.
- Waybar/Walker/Starship rely solely on linked directory listing.
- Reference: [tui.rs](/home/oldjobobo/Projects/rust/theme-manager-plus/rust/src/tui.rs:1296)

3. Existing link is treated as valid without target verification.
- All `ensure_omarchy_default_theme_link` functions return early if link path exists; stale or wrong symlink is not corrected.

4. Walker resolver validates only directory existence.
- Resolver does not require `style.css`; invalid default dir can be linked and later rejected during apply.
- Reference: [walker.rs](/home/oldjobobo/Projects/rust/theme-manager-plus/rust/src/walker.rs:105)

5. Version compatibility handling is ad hoc per module.
- Candidate paths differ by module without a shared version policy or explicit source-kind metadata.

## Current Test Coverage Snapshot

| Module | Default-path tests present | Notable gaps |
|---|---|---|
| Waybar | Tests `.local/share/omarchy/default/waybar` link path | No precedence tests when multiple candidates exist |
| Walker | Tests `.local/share/omarchy/default/walker/themes/omarchy-default` | No alternate/fallback layout tests |
| Hyprlock | Tests multiple fallbacks (`default/hyprlock`, `themes/omarchy-default`, `config/hypr`, config-space paths) | No explicit precedence conflict test matrix |
| Starship | Tests `.local/share/omarchy/default/starship.toml` | No `default/starship/themes/omarchy-default.toml` precedence test |

References:
- [cli_waybar.rs](/home/oldjobobo/Projects/rust/theme-manager-plus/rust/tests/cli_waybar.rs:173)
- [cli_walker.rs](/home/oldjobobo/Projects/rust/theme-manager-plus/rust/tests/cli_walker.rs:309)
- [cli_hyprlock.rs](/home/oldjobobo/Projects/rust/theme-manager-plus/rust/tests/cli_hyprlock.rs:105)
- [cli_starship.rs](/home/oldjobobo/Projects/rust/theme-manager-plus/rust/tests/cli_starship.rs:129)

## Phase 1 Recommendation Summary
- Move all candidate probing into one shared resolver layer.
- Define one explicit precedence policy per module in a canonical spec.
- Standardize link integrity checks (exists + valid target semantics).
- Align TUI option visibility behavior across modules.
