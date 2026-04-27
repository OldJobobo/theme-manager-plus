# Foot Sixel Support Plan

Status: In progress  
Owner: TBD  
Scope: Add Sixel image preview support for Foot terminal in browse TUI.

## Goal

Add image preview support in Foot by introducing a Sixel-capable preview backend that integrates with the existing TUI preview lifecycle without regressing Kitty or Chafa behavior.

## Tracking

- [x] 1. Backend model updates
  - [x] Add `Sixel` to `PreviewBackendKind` in `rust/src/tui.rs`.
  - [x] Keep backend flow ordered as: `Kitty` -> `Sixel(Foot)` -> `Chafa` -> `None`.

- [x] 2. Backend detection
  - [x] Detect Foot from env (`TERM` / `TERM_PROGRAM` contains `foot`).
  - [x] Require an available image renderer for Sixel path (`chafa`).
  - [x] Add tests for detection precedence and fallback behavior.

- [x] 3. Sixel rendering implementation
  - [x] Implement `PreviewBackend::render()` branch for `Sixel`.
  - [x] Render image in the preview pane region (respect TUI rect).
  - [x] Implement explicit rect clearing when preview should be removed.

- [ ] 4. Draw-loop compatibility hardening
  - [x] Verify existing invalidate logic works with Sixel (`Clear`, `Render`, `ClearAndRender`).
  - [ ] Refactor to post-draw image emission only if Foot behavior requires it.
  - [ ] Confirm no stale/distorted preview artifacts across tab/apply transitions.

- [x] 5. Text fallback behavior
  - [x] For Sixel backend with image path, return blank text preview so the pane is image-only.
  - [x] Preserve existing non-image fallback messaging (`No preview available.`).

- [x] 6. Tests
  - [x] Add unit tests for backend detection logic.
  - [x] Keep/verify existing preview action decision tests.
  - [x] Run `cargo test --manifest-path rust/Cargo.toml`.

- [x] 7. Documentation and release notes
  - [x] Update preview prerequisites/docs in `README.md`.
  - [x] Add `## Unreleased` entry in `CHANGELOG.md`.
  - [x] Add matching `## Unreleased` note in `RELEASE_NOTES.md`.

## Acceptance Criteria

- Foot terminal renders theme preview images in browse mode via Sixel.
- Preview clears correctly when switching tabs/items or when no image is available.
- Kitty preview behavior remains unchanged.
- Chafa text preview fallback remains unchanged when graphics backends are unavailable.
- Test suite passes.

## Validation Checklist

- [x] `cargo test --manifest-path rust/Cargo.toml`
- [ ] `cargo run --manifest-path rust/Cargo.toml -- browse`
- [ ] Manual test in Foot:
  - [ ] Image appears for themes with `preview.png` / `theme.png` / `backgrounds/*`.
  - [ ] Image clears when moving to items without preview.
  - [ ] No stale image after apply and tab changes.
