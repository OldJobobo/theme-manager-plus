# Presets Plan

## 1) Purpose
Presets let users save and reapply theme-related selections as a named bundle. A preset captures:
- Theme name
- Waybar selection (none/auto/named)
- Starship selection (none/preset/named/theme)

Presets do not change behavior settings like quiet mode, awww transitions, or skip flags.

## 2) Storage
Primary file (user scope):
- `~/.config/theme-manager/presets.toml`

Format:
- Simple TOML keyed by preset name.
- Each preset is a table with explicit fields.
- Preset names are user-facing and stored as-is (no normalization beyond trimming).

Example:
```toml
[preset."Daily Driver"]
theme = "dracula"
waybar.mode = "auto"
starship.mode = "preset"
starship.preset = "bracketed-segmented"

[preset."Minimal"]
theme = "noir"
waybar.mode = "none"
starship.mode = "none"

[preset."Workspace"]
theme = "paper"
waybar.mode = "named"
waybar.name = "work"
starship.mode = "named"
starship.name = "mono"

[preset."Theme Starship"]
theme = "dawn"
starship.mode = "theme"
```

Notes:
- `waybar.mode` values: `none`, `auto`, `named`.
- `starship.mode` values: `none`, `preset`, `named`, `theme`.
- `starship.mode = "theme"` uses `starship.yaml` inside the selected theme.
- Missing optional fields should be treated as empty/none.

## 3) Precedence and Overrides
Order of precedence for a `set` using presets:
1) CLI flags
2) Preset values
3) Config defaults

Examples:
- `theme-manager preset load "Daily Driver" -w` forces Waybar auto even if preset says `named`.
- `theme-manager preset load "Minimal" --waybar=work` uses named Waybar despite preset `none`.
- Starship CLI flags override preset Starship values if a Starship flag is added later.

## 4) CLI Surface
New command group:
- `preset save <PresetName> [--theme <ThemeName>] [--waybar <mode|name>] [--starship <mode|name>]`
- `preset load <PresetName> [set flags...]`
- `preset list`
- `preset remove <PresetName>`

Details:
- `preset save` uses the current theme and defaults unless explicit options override.
- `preset load` applies the preset by routing through the existing `set` flow.
- `preset list` prints names in sorted order.
- `preset remove` deletes the preset entry; error if missing.

## 5) TUI Integration
Add a new Presets tab with:
- List of preset names on the left.
- A summary panel on the right showing theme/waybar/starship selections.
- Enter selects preset and moves to Review.
- Ctrl+Enter on Review applies the selected preset values.

Behavior:
- Preset selection populates Theme/Waybar/Starship tabs with the preset values.
- Tabs remain editable after loading a preset (user can override before apply).

## 6) Validation Rules
- `theme` must resolve to an existing theme dir or symlink.
- `waybar.mode = "named"` requires `waybar.name`.
- `starship.mode = "preset"` requires `starship.preset`.
- `starship.mode = "named"` requires `starship.name`.
- `starship.mode = "theme"` does not require extra fields but fails if the theme lacks `starship.yaml`.

Invalid presets should fail on load with a clear error.

## 7) Error and Exit Codes
Follow current conventions:
- Unknown/missing preset: exit 1 with message.
- Invalid preset fields: exit 1 with message.
- Usage errors (missing args): exit 2.

## 8) Tests
Add Rust integration tests covering:
- Save/load/list/remove happy paths.
- Load with missing theme or bad starship config.
- CLI flags overriding preset values.
- TUI preset selection mapping into Review (can be unit-tested via selection logic if possible).

## 9) Documentation Updates
Add to:
- `README.md`: presets section, examples, and precedence rules.
- `config.toml.example`: note that presets are stored in `presets.toml`.
- `CHANGELOG.md`: entry under "Unreleased".

## 10) Versioning
When presets ship:
- Bump `VERSION` in `src/theme-manager.sh`.
- Add a changelog entry describing the presets feature.
