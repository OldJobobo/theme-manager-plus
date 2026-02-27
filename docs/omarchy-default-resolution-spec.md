# Omarchy Default Resolution Spec (Phase 2)

## Status
Proposed canonical contract for unified resolver implementation.

## Objective
Define one shared, version-aware policy for resolving Omarchy default assets used by Theme Manager+ module tabs and CLI flows.

## Non-Negotiables
- Read-only interaction with Omarchy base in `~/.local/share/omarchy`.
- No writes outside Theme Manager+ managed destinations.
- Consistent behavior between TUI and CLI for default discovery and use.

## Root Detection Precedence
Resolver root precedence must be:
1. `OMARCHY_PATH` (non-empty)
2. `ResolvedConfig.omarchy_bin_dir` parent
3. `$HOME/.local/share/omarchy`

Reference baseline: [omarchy.rs](/home/oldjobobo/Projects/rust/theme-manager-plus/rust/src/omarchy.rs:28)

## Resolver Contract
All module resolvers return this conceptual shape:

- `None` if no valid default candidate exists.
- `Some(ResolvedOmarchyDefault)` when found:
  - `path`: resolved filesystem path
  - `kind`: source category
  - `module`: `waybar | walker | hyprlock | starship`
  - `validation`: checks applied (for diagnostics)

Recommended source kind enum:
- `OmarchyDefaultNamed`
- `OmarchyDefaultBase`
- `OmarchyThemeStoreDefault`
- `OmarchyConfigFallback`
- `OmarchyUserConfigFallback`

## Validation Rules
- Waybar default candidate is valid only if both `config.jsonc` and `style.css` exist.
- Walker default candidate is valid only if `style.css` exists.
- Hyprlock default candidate is valid only if `hyprlock.conf` exists.
- Starship default candidate is valid only if target `.toml` file exists.

## Canonical Candidate Precedence by Module

### Waybar
1. `<root>/default/waybar/themes/omarchy-default`
2. `<root>/default/waybar`

### Walker
1. `<root>/default/walker/themes/omarchy-default`
2. `<root>/default/walker`

### Hyprlock
1. `<root>/default/hyprlock/themes/omarchy-default`
2. `<root>/default/hyprlock`
3. `<root>/themes/omarchy-default`
4. `<root>/config/hypr`
5. `$HOME/.config/omarchy/default/hyprlock/themes/omarchy-default`
6. `$HOME/.config/omarchy/default/hyprlock`
7. `$HOME/.config/omarchy/themes/omarchy-default`
8. `$HOME/.config/omarchy/config/hypr`

### Starship
1. `<root>/default/starship/themes/omarchy-default.toml`
2. `<root>/default/starship.toml`
3. `<root>/default/starship/starship.toml`

## Link/Create Behavior
- `ensure_omarchy_default_theme_link`-style flows must:
1. Resolve default with shared resolver.
2. If destination link/file does not exist, create symlink.
3. If destination exists but is a broken symlink or wrong target, repair it.
4. If destination exists as regular file/dir, preserve and warn; do not destructively replace without explicit apply path.

## TUI Visibility Rules
- TUI module tabs must derive `omarchy-default` visibility from shared resolver output only.
- No module-specific visibility exceptions.
- If resolver returns `None`, do not show `omarchy-default` in the tab list.

## CLI Parity Rules
- `set`, `next`, `preset load`, and module-only commands must all use the same shared resolver and link-integrity checks.
- Given identical filesystem/config state, resolved default target must be identical across all command paths.

## Logging/Warn Policy
- Quiet mode: suppress informational logs.
- Non-quiet mode:
  - log link creation/repair with source and destination.
  - warn once when destination exists but cannot be safely repaired automatically.

## Compatibility Policy
- Prefer newest known Omarchy default layout first.
- Keep fallback support for older observed layouts where safe and file-validated.
- Do not assume every Omarchy install has every module default.

## Sign-off Checklist
- [x] Root precedence defined.
- [x] Resolver return contract defined.
- [x] Module-specific candidate order defined.
- [x] Validation requirements defined.
- [x] TUI/CLI parity rules defined.
- [x] Link integrity behavior defined.
- [x] Compatibility policy defined.
