# PR Split Plan

Use this as a tracked cleanup plan for the current mixed worktree.

## Slice 1: Walker Hardening + Parity
- `rust/src/walker.rs`
- `rust/src/cli.rs`
- `rust/src/lib.rs`
- `rust/src/presets.rs`
- `rust/tests/cli_walker.rs`
- `rust/tests/cli_presets.rs`

## Slice 2: Hyprlock Module + Compatibility
- `rust/src/hyprlock.rs`
- `rust/src/theme_ops.rs`
- `rust/src/omarchy.rs`
- `rust/src/tui.rs`
- `rust/tests/cli_hyprlock.rs`
- `rust/tests/cli_theme_ops.rs`

## Slice 3: Documentation + Release Metadata
- `README.md`
- `CHANGELOG.md`
- `RELEASE_NOTES.md`
- `config.toml.example`
- `HYPRLOCKPLAN.md`

## Slice 4: Version Source Unification
- `VERSION`
- `rust/build.rs`
- `rust/Cargo.toml`
- `src/theme-manager.sh`

## Validation Per Slice
1. `cargo test --test cli_walker`
2. `cargo test --test cli_hyprlock`
3. `cargo test --test cli_presets`
4. `cargo test`
