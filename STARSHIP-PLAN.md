# Starship Support Plan

## Goals
- Apply Starship config on theme switches.
- Support builtin Starship presets and user-installed Starship configs.
- Keep behavior opt-in via config defaults or browse selection.

## Configuration
Add config keys and defaults:
- `STARSHIP_CONFIG` (default: `~/.config/starship.toml`)
- `STARSHIP_THEMES_DIR` (default: `~/.config/starship-themes`)
- `DEFAULT_STARSHIP_MODE` (`preset`, `named`, or empty)
- `DEFAULT_STARSHIP_PRESET` (used when mode is `preset`)
- `DEFAULT_STARSHIP_NAME` (used when mode is `named`)

## Implementation
- Ensure `STARSHIP_THEMES_DIR` exists during install and when applying Starship.
- Apply modes:
  - `preset`: run `starship preset <name>` and write to `STARSHIP_CONFIG`.
  - `named`: copy `STARSHIP_THEMES_DIR/<name>.toml` to `STARSHIP_CONFIG`.
  - empty: no Starship change.
- Hook into the theme switch flow after config load and before/after Omarchy hooks.
- Add Starship selection in `browse`:
  - `Omarchy default` (no change)
  - `Preset: <name>` from `starship preset --list` (if available)
  - `Theme: <name>` from `STARSHIP_THEMES_DIR/*.toml`

## Documentation
- Update `README.md` with Starship behavior and usage examples.
- Update `CONFIGPLAN.md` and `config.example` with new keys.
- Note Starship support in changelog entries.

## Tests
- Bats test: apply preset to `~/.config/starship.toml` using a mocked `starship` command.
- Bats test: apply named theme from `~/.config/starship-themes/*.toml`.
- Bats test: missing preset/theme produces a warning (optional).
