# Repository Guidelines

## Project Structure & Module Organization
- `Omarchy-theme-management.md`: reference notes on Omarchy theme compatibility and required behavior.
- `README.md`: user-focused guide (plain-English overview, commands, config, troubleshooting).
- `DOCSPLAN.md`: documentation roadmap and writing style rules.
- `CONFIGPLAN.md`: config system scope, keys, and testing checklist.
- `VERSION`: single source of truth for user-facing version display (Rust CLI/TUI).
- `rust/src/main.rs`: Rust CLI entry point and command dispatch.
- `extras/omarchy/theme_manager_plus.lua`: Walker/Elephant menu (kept for reference).
- `install-omarchy-menu.sh`: installs the TUI app launcher via `omarchy-tui-install`.
- `rust/tests/`: Rust integration tests and shared support fixtures.

## Project Intent & Scope
This project does not replace Omarchy theming. It provides an alternative manager that triggers the same theme switching behavior as Omarchy’s menu flow (Menu > Style > Theme > <name>). Today the focus is on matching Omarchy’s built-in theme change behavior via the manager script. Future work may add optional theme add-ons (e.g., Waybar themes), but those should layer on top of the Omarchy-compatible flow rather than diverge from it.

## Build, Test, and Development Commands
- `cargo run --manifest-path rust/Cargo.toml -- help`: show CLI usage and available commands.
- `cargo run --manifest-path rust/Cargo.toml -- list`: list available themes.
- `cargo run --manifest-path rust/Cargo.toml -- set <theme>`: switch to a theme (use `-w` for Waybar, `-q` for quiet).
- `cargo run --manifest-path rust/Cargo.toml -- next`: cycle to the next theme.
- `cargo run --manifest-path rust/Cargo.toml -- browse`: interactive selector.
- `cargo run --manifest-path rust/Cargo.toml -- current`: print the current theme.
- `cargo run --manifest-path rust/Cargo.toml -- bg-next`: cycle the background within the current theme.
- `cargo run --manifest-path rust/Cargo.toml -- print-config`: show resolved config values.
- `cargo run --manifest-path rust/Cargo.toml -- install <git-url>`: clone and activate a theme.
- `cargo run --manifest-path rust/Cargo.toml -- update`: pull updates for git-based themes.
- `cargo run --manifest-path rust/Cargo.toml -- remove [theme]`: remove a theme (prompts if omitted).
- `./install-omarchy-menu.sh`: create a Theme Manager+ launcher in Omarchy’s app list.
- `cargo test --manifest-path rust/Cargo.toml`: run the Rust test suite.

## Coding Style & Naming Conventions
Use idiomatic Rust in `rust/src`, keep modules focused, and prefer explicit, testable command helpers. Follow existing naming and error-handling patterns (`Result`, `anyhow`, and small composable functions). If you add linting/formatting commands, document exact invocations and versions here.

## Testing Guidelines
Tests live in `rust/tests/` and run with `cargo test`. Name files by feature, keep tests hermetic, and use shared support helpers in `rust/tests/support/`.

## Commit & Pull Request Guidelines
No commit message convention is enforced. Use concise, present-tense messages (e.g., "add theme switcher") and include context in the body for behavior changes. For pull requests, include a brief summary, testing notes, and any relevant screenshots or terminal output.

## Architecture Notes
Follow the compatibility requirements outlined in `Omarchy-theme-management.md`. Maintain the current theme and background symlinks under `~/.config/omarchy/current/`, reload user-facing components, and trigger `omarchy-hook theme-set` after switching.
Omarchy default component discovery for Waybar/Walker/Hyprlock/Starship is centralized in `rust/src/omarchy_defaults.rs`; keep module behavior aligned to this shared resolver instead of adding per-module fallback probing.

## Configuration
Defaults can be set via `~/.config/theme-manager/config.toml` or `./.theme-manager.toml`. Local config overrides user config; CLI flags override both.
See `config.toml.example` for a fully commented template.

## Versioning
Update the repository `VERSION` file when behavior changes. The CLI exposes it via `theme-manager version`.
Add entries under `## Unreleased` in `CHANGELOG.md` as you make changes, then move them into a new version heading when you bump `VERSION`.
Add a matching entry to `RELEASE_NOTES.md` for each release. Release notes are user-facing highlights; exclude tests and documentation-only changes.
