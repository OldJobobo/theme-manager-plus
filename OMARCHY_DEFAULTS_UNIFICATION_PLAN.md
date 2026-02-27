# Omarchy Default Settings Unification Plan

## Goal
Unify how all Theme Manager+ module tabs and CLI flows load Omarchy default settings so behavior consistently resolves from Omarchy base files across supported Omarchy layouts/versions.

## Scope
- In scope: Waybar, Walker, Hyprlock, Starship default-source discovery and application paths used by TUI tabs and CLI (`set`, `next`, `browse`, `preset load`, module-only commands).
- In scope: shared resolver logic, tests, and docs.
- Out of scope: changing Omarchy files in `~/.local/share/omarchy/` (read-only source), UI redesign, non-default custom theme behavior.

## Tracking
- Status values: `Not Started`, `In Progress`, `Blocked`, `Done`
- Update each phase header with current status during execution.

---

## Phase 1 - Baseline Audit (Status: Done)
### Objective
Produce a concrete map of current default-resolution behavior per module and per flow.

### Actionable Items
- [x] Inventory all current default resolvers and callers in Rust:
  - [x] `rust/src/waybar.rs`
  - [x] `rust/src/walker.rs`
  - [x] `rust/src/hyprlock.rs`
  - [x] `rust/src/starship.rs`
  - [x] `rust/src/tui.rs`
  - [x] `rust/src/lib.rs`
  - [x] `rust/src/theme_ops.rs`
  - [x] `rust/src/omarchy.rs`
- [x] Create a matrix of current fallback order per module (actual behavior, not intended behavior).
- [x] Identify behavior mismatches between:
  - [x] TUI tab item availability
  - [x] CLI default application
  - [x] Module-only commands
- [x] Record test coverage currently present and missing by module/layout.

### Deliverables
- [x] Audit matrix document added to repo (`docs/omarchy-default-audit.md` or equivalent).
- [x] List of concrete divergence bugs/risks with file references.

### Exit Criteria
- [x] We can answer, for every module, "what exact file path is picked first, second, third" for each Omarchy layout.

---

## Phase 2 - Canonical Resolution Spec (Status: Done)
### Objective
Define one canonical resolution contract that is version-aware and shared.

### Actionable Items
- [x] Define supported Omarchy layout candidates per module (ordered precedence).
- [x] Define root detection precedence:
  - [x] `OMARCHY_PATH`
  - [x] configured `omarchy_bin_dir`
  - [x] `~/.local/share/omarchy`
- [x] Define resolver return contract (e.g., `Option<ResolvedDefault>` with path + source-kind metadata).
- [x] Define missing-file behavior:
  - [x] silent skip vs warning
  - [x] when to show/no-show in TUI lists
- [x] Define compatibility policy for older/newer Omarchy versions.

### Deliverables
- [x] Canonical spec doc committed (`docs/omarchy-default-resolution-spec.md`).
- [x] Sign-off checklist embedded in doc.

### Exit Criteria
- [x] Every module can consume the same contract without special-case path probing.

---

## Phase 3 - Shared Resolver Implementation (Status: Done)
### Objective
Implement one internal resolver module and remove duplicated probing logic.

### Actionable Items
- [x] Add new module (example): `rust/src/omarchy_defaults.rs`.
- [x] Implement shared functions:
  - [x] `resolve_waybar_default(...)`
  - [x] `resolve_walker_default(...)`
  - [x] `resolve_hyprlock_default(...)`
  - [x] `resolve_starship_default(...)`
- [x] Centralize candidate-path generation and existence checks.
- [x] Reuse/centralize Omarchy root detection.
- [x] Add structured debug logging hooks (quiet-aware).
- [x] Keep behavior non-destructive and reversible.

### Deliverables
- [x] New resolver module with unit-testable helpers.
- [x] No module-level duplicated fallback probing remains.

### Exit Criteria
- [x] One source of truth exists for default resolution and compiles cleanly.

---

## Phase 4 - Integration Across Tabs and Commands (Status: Done)
### Objective
Wire all tabs and CLI command paths to the shared resolver.

### Actionable Items
- [x] Update module handlers to consume shared resolver:
  - [x] `waybar::ensure_omarchy_default_theme_link`
  - [x] `walker::ensure_omarchy_default_theme_link`
  - [x] `hyprlock::ensure_omarchy_default_theme_link`
  - [x] `starship::ensure_omarchy_default_theme_link`
- [x] Update TUI builders to rely on shared resolution outcomes:
  - [x] `build_waybar_items`
  - [x] `build_walker_items`
  - [x] `build_hyprlock_items`
  - [x] `build_starship_items`
- [x] Validate parity between:
  - [x] `browse` apply path
  - [x] `set`/`next`
  - [x] `preset load`
  - [x] module-only subcommands
- [x] Preserve Omarchy-safe behavior (read-only in `~/.local/share/omarchy/`).

### Deliverables
- [x] Unified behavior in runtime code paths.
- [x] No TUI/CLI mismatch for `omarchy-default` availability and target selection.

### Exit Criteria
- [x] Same input environment produces same resolved default path across all flows.

---

## Phase 5 - Test Matrix Expansion (Status: In Progress)
### Objective
Add regression tests that prove cross-version compatibility and unified behavior.

### Actionable Items
- [ ] Add/expand table-driven tests for each module default resolver:
  - [x] layout variant A: `default/<module>` style
  - [x] layout variant B: `default/<module>/themes/omarchy-default`
  - [x] layout variant C: theme-root fallback where applicable
  - [x] missing paths behavior
- [ ] Add integration tests validating TUI list and CLI apply parity.
- [x] Add precedence tests where multiple candidates exist.
- [x] Ensure tests remain hermetic under `rust/tests/support`.
- [x] Run full test suite and capture results.

### Deliverables
- [ ] New tests in `rust/tests/` with clear naming by module/behavior.
- [ ] Updated coverage notes in tracking doc.

### Exit Criteria
- [ ] Failing test reproduces any future regression in resolver precedence/parity.

---

## Phase 6 - Documentation and Rollout (Status: In Progress)
### Objective
Document final behavior and prepare release notes.

### Actionable Items
- [x] Update `README.md` with canonical default-resolution behavior.
- [x] Update `AGENTS.md` contributor guidance for Omarchy default loading.
- [x] Add changelog entry under `## Unreleased` in `CHANGELOG.md`.
- [ ] If version bump is included, update `VERSION` and `RELEASE_NOTES.md`.
- [x] Include troubleshooting notes for users with non-standard Omarchy layouts.

### Deliverables
- [ ] User-facing docs aligned with implemented behavior.
- [ ] Release notes reflect functional impact.

### Exit Criteria
- [ ] A maintainer can verify behavior from docs alone.

---

## Cross-Phase Risk Controls
- [ ] Never modify files under `~/.local/share/omarchy/`.
- [ ] Prefer smallest reversible changes first.
- [ ] Keep all path probing explicit and test-backed.
- [ ] Preserve existing behavior unless mismatch/bug is intentional and documented.

## Definition of Done
- [ ] Shared resolver is the only source for Omarchy default discovery.
- [ ] TUI tabs and CLI flows are behaviorally consistent.
- [ ] Multi-layout compatibility is covered by tests.
- [ ] Docs/changelog updated and accurate.
