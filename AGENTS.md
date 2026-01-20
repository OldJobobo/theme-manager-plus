# Repository Guidelines

## Project Structure & Module Organization
- `Omarchy-theme-management.md`: reference notes on Omarchy theme compatibility and required behavior.
- `README.md`: user-focused guide (plain-English overview, commands, config, troubleshooting).
- `DOCSPLAN.md`: documentation roadmap and writing style rules.
- `CONFIGPLAN.md`: config system scope, keys, and testing checklist.
- `src/theme-manager.sh`: defines `VERSION` and the `version` command.
- `bin/theme-manager`: CLI entry point.
- `extras/omarchy/theme_manager_plus.lua`: Walker/Elephant menu (kept for reference).
- `install-omarchy-menu.sh`: installs the TUI app launcher via `omarchy-tui-install`.
- `tests/`: Bats tests and the test runner script.

## Project Intent & Scope
This project does not replace Omarchy theming. It provides an alternative manager that triggers the same theme switching behavior as Omarchy’s menu flow (Menu > Style > Theme > <name>). Today the focus is on matching Omarchy’s built-in theme change behavior via the manager script. Future work may add optional theme add-ons (e.g., Waybar themes), but those should layer on top of the Omarchy-compatible flow rather than diverge from it.

## Build, Test, and Development Commands
- `./bin/theme-manager help`: show CLI usage and available commands.
- `./bin/theme-manager list`: list available themes.
- `./bin/theme-manager set <theme>`: switch to a theme (use `-w` for Waybar, `-q` for quiet).
- `./bin/theme-manager next`: cycle to the next theme.
- `./bin/theme-manager browse`: interactive theme + Waybar picker (requires `fzf`).
- `./bin/theme-manager current`: print the current theme.
- `./bin/theme-manager bg-next`: cycle the background within the current theme.
- `./bin/theme-manager print-config`: show resolved config values.
- `./bin/theme-manager install <git-url>`: clone and activate a theme.
- `./bin/theme-manager update`: pull updates for git-based themes.
- `./bin/theme-manager remove [theme]`: remove a theme (prompts if omitted).
- `./install-omarchy-menu.sh`: create a Theme Manager+ launcher in Omarchy’s app list.
- `./tests/run.sh`: run the Bats test suite (requires `bats` in PATH).

## Coding Style & Naming Conventions
Use `bash` with `set -euo pipefail` and keep functions small and composable. Indent with 2 spaces. Use `snake_case` for variables/functions and lowercase hyphenated filenames (e.g., `theme-set.sh`). Prefer explicit return codes for invalid input. If you add linting/formatting (e.g., `shellcheck`, `shfmt`), document the exact commands and versions here.

## Testing Guidelines
Tests live in `tests/` and use Bats. Name files by feature, e.g., `tests/theme_manager.bats`. Keep tests hermetic; if a test needs filesystem fixtures, create them under `tests/fixtures/` and clean up within the test. Run tests via `./tests/run.sh` or directly with `bats tests`.

## Commit & Pull Request Guidelines
No commit message convention is enforced. Use concise, present-tense messages (e.g., "add theme switcher") and include context in the body for behavior changes. For pull requests, include a brief summary, testing notes, and any relevant screenshots or terminal output.

## Architecture Notes
Follow the compatibility requirements outlined in `Omarchy-theme-management.md`. Maintain the current theme and background symlinks under `~/.config/omarchy/current/`, reload user-facing components, and trigger `omarchy-hook theme-set` after switching.

## Configuration
Defaults can be set via `./.theme-manager.conf` or `~/.config/theme-manager/config`. Local config overrides user config; CLI flags override both.
See `config.example` for a fully commented template.

## Versioning
Update the `VERSION` constant in `src/theme-manager.sh` when behavior changes. The CLI exposes it via `theme-manager version`.
Add entries under `## Unreleased` in `CHANGELOG.md` as you make changes, then move them into a new version heading when you bump `VERSION` in `src/theme-manager.sh`.
Add a matching entry to `REALSENOTES.md` for each release. Release notes are user-facing highlights; exclude tests and documentation-only changes.
