# Implementation Plan: Dual-Interface Architecture (CLI + TUI)

## 1. Executive Summary

This document outlines the technical migration strategy for evolving `devinventory` from a single CLI application to a dual-interface architecture supporting both CLI and TUI (Terminal User Interface) modes. The migration adopts Rust's idiomatic library-first pattern (`lib.rs` + `src/bin/`) to maximize code reuse while maintaining clean separation of concerns.

**Key Objectives:**

- Zero-downtime migration: Existing CLI functionality remains stable
- Shared core logic: Business logic (crypto, storage, app services) reused across interfaces
- Clean dependency management: TUI dependencies (ratatui, crossterm) isolated via feature flags
- Future-proof architecture: Clear path to workspace extraction if project scales

## 2. Target Architecture

### 2.1 Repository Structure

```TEXT
devinventory/
├── Cargo.toml              # Single crate with lib + bins (can evolve into workspace)
├── src/
│   ├── lib.rs              # Core library exports (the "brain")
│   ├── bin/
│   │   ├── cli.rs          # CLI entry point (migrated from main.rs)
│   │   └── tui.rs          # TUI entry point (new)
│   ├── app/                # Business logic (unchanged)
│   ├── config/             # Configuration management (shared)
│   ├── crypto/             # Cryptography services (shared)
│   ├── domain/             # Domain models (shared)
│   ├── keymgr/             # Key management (shared)
│   ├── storage/            # Persistence layer (shared)
│   ├── error.rs            # Error types (shared)
│   └── ui/                 # UI implementations
│       ├── mod.rs
│       ├── cli/            # CLI-specific presentation logic
│       └── tui/            # TUI-specific components (ratatui)
└── tests/                  # Integration tests
```

### 2.2 Crate Configuration

```TOML
[package]
name = "devinventory" # Consider renaming package from "DevInventory" for crate-name consistency
version = "0.1.0"
edition = "2024"
[lib]
name = "devinventory"
path = "src/lib.rs"
[[bin]]
name = "devinventory-cli"
path = "src/bin/cli.rs"
[[bin]]
name = "devinventory-tui"
path = "src/bin/tui.rs"
required-features = ["tui"]
[features]
tui = ["dep:ratatui", "dep:crossterm"]
[dependencies]
# Core dependencies (used by both)
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
# ... other shared deps
# TUI-specific dependencies (optional)
ratatui = { version = "0.24", optional = true }
crossterm = { version = "0.27", optional = true }
```

## 3. Implementation Phases

### Phase 1: Foundation Refactoring (Week 1)

**Goal:** Restructure project without changing business logic

**Tasks:**

1. **Create Library Interface**
   - Create `src/lib.rs` exporting all core modules (`app`, `config`, `crypto`, `domain`, `keymgr`, `storage`, `error`)
   - Ensure all public APIs are async-compatible where needed

2. **Migrate CLI Binary**
   - Move `src/main.rs` → `src/bin/cli.rs`
   - Update imports: `use crate::` → `use devinventory::`
   - Update `Cargo.toml` to add `[lib]` and `[[bin]]` definitions (and set package name to lowercase `devinventory` if renaming)
   - Verify: `cargo run --bin devinventory-cli` works identically to previous `cargo run`

3. **Feature Flag Setup**
   - Add `tui` feature flag with optional dependencies (no `cli` feature needed)
   - Ensure CLI builds without TUI deps: `cargo build --bin devinventory-cli` (lightweight)
   - Ensure TUI builds with deps: `cargo build --bin devinventory-tui --features tui`

**Deliverables:**

- All existing tests pass (`cargo test --lib`, `cargo test --bin devinventory-cli`)
- No functional changes to CLI behavior
- Clean separation between library and binary interface

**Branch:** `refactor/lib-structure` → Merge to `develop` upon completion

### Phase 2: TUI Infrastructure (Week 2)

**Goal:** Establish TUI event loop and terminal management

**Tasks:**

1. **Terminal Management Module**
   - Create `src/ui/tui/terminal.rs` (init, restore, panic hooks)
   - Implement `Tui` type alias and helper functions
   - Add Crossterm event polling abstraction

2. **Event System**
   - Create `src/ui/tui/events.rs`
   - Map Crossterm `KeyEvent` to application `Command` enum
   - Implement non-blocking event polling with tick rates (e.g., 60 FPS)

3. **State Management**
   - Create `TuiApp` wrapper in `src/ui/tui/app.rs`
   - Bridge between ratatui's immediate mode and our async services
   - Handle Tokio runtime integration within TUI context

4. Deliverables:
   - `cargo run --bin devinventory-tui` launches without errors
   - Empty TUI screen with proper terminal initialization/restoration
   - Clean exit on 'q' key press
   - No business logic yet (just infrastructure)

**Branch:** `feat/tui-infrastructure` (based on `develop` after Phase 1 merge)

### Phase 3: UI Components & Layout (Week 3-4)

**Goal:** Implement visual interface using ratatui widgets

**Tasks:**

1. Layout System
   - Define screen layouts (header, main content, status bar, sidebar)
   - Implement responsive sizing for different terminal dimensions

2. Component Library
   - `SecretList` component (table/list view)
   - `SecretDetail` component (form/view for single secret)
   - `StatusBar` component (error messages, operation feedback)
   - `InputModal` component (dialogs for user input)

3. Widget Implementations
   - Implement `Widget` trait for domain objects where appropriate
   - Create `ui::render()` function coordinating all components
   - Add styling system (colors, borders) consistent with CLI branding

Deliverables:

- Visual layout matches wireframes/design specs
- Navigation between views (List → Detail → Edit)
- Keyboard shortcuts displayed and functional
- Unit tests for widget rendering (Buffer comparison tests)

**Branch:** `feat/tui-components`

### Phase 4: Integration & Service Wiring (Week 5)

**Goal:** Connect TUI to existing business logic

**Tasks:**

1. **Service Integration**
   - Wire `SecretService` calls to TUI actions
   - Implement async operation handling (loading states, spinners)
   - Error handling: convert service errors to user-friendly TUI notifications

2. **State Synchronization**
   - Ensure TUI state updates reflect database changes
   - Handle concurrent modifications (refresh on focus)
   - Cache management for large secret lists

3. **User Workflows**
   - Add secret flow
   - Edit secret flow
   - Delete with confirmation
   - Search/filter functionality

**Deliverables:**

- Full CRUD operations working through TUI
- Password generation and crypto operations functional
- Error handling (popup dialogs for failures)
- Performance acceptable (< 100ms response for local operations)

**Branch:** `feat/tui-integration`

### Phase 5: Polish & Release Preparation (Week 6)

**Goal:** Production readiness

**Tasks:**

1. **Testing**
   - Integration tests for TUI workflows
   - Terminal compatibility testing (Linux, macOS, Windows)
   - Memory leak checks (long-running sessions)

2. **Documentation**
   - Update README with TUI installation instructions
   - Keybinding reference document
   - Migration guide for CLI power users

3. **Distribution**
   - Update CI/CD to build both binaries
   - Create release artifacts for `devinventory-cli` and `devinventory-tui`
   - Homebrew/Scoop formula updates (if applicable)

**Deliverables:**

- Release candidate builds
- Updated documentation
- Regression testing confirms CLI still works perfectly

**Branch:** **release/v0.2.0** (merging all TUI branches)

## 4. Branch strategy

```TEXT
master (stable)
  ↑
develop (integration)
  ↑
  ├── refactor/lib-structure  (Phase 1 - MERGE FIRST)
  │
  ├── feat/tui-infrastructure (Phase 2)
  │
  ├── feat/tui-components     (Phase 3)
  │
  └── feat/tui-integration    (Phase 4)
          ↓
    release/v0.2.0            (Phase 5)
          ↓
        master
```

**Rules:**

- `refactor/lib-structure` merges to `develop` immediately after Phase 1
- TUI feature branches branch from updated `develop`
- Weekly sync from `develop` into long-running TUI branches to prevent drift
- Feature flags allow merging incomplete TUI code to `develop` without affecting CLI users

## 5. Risk Mitigation

| Risk                              | Impact | Mitigation                                                                                                                                                      |
| :-------------------------------- | :----: | :-------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **CLI Regression**                |  High  | Comprehensive test suite before refactoring; keep `devinventory-cli` binary identical in behavior; A/B testing against old main.rs                              |
| **Async Runtime Conflicts**       | Medium | Tokio remains entry point for both binaries; TUI uses `block_on` or `spawn_blocking` for sync ratatui calls; clear ownership of runtime between UI and Services |
| **Terminal State Corruption**     | Medium | Robust panic hooks in `tui.rs` ensuring `LeaveAlternateScreen` and `disable_raw_mode` always execute; manual testing on panic scenarios                         |
| **Binary Size Bloat**             |  Low   | Feature flags ensure CLI users don't download ratatui/crossterm; `strip` symbols in release builds; consider `cargo-bloat` monitoring                           |
| **Developer Workflow Disruption** |  Low   | Clear migration guide; temporary support for old `cargo run` via default binary configuration; IDE settings update documentation                                |

## 6. Success Criteria

- [ ] `cargo build --release` produces two working binaries: `devinventory-cli` and `devinventory-tui`
- [ ] CLI binary size unchanged (no TUI deps included)
- [ ] 100% of existing CLI tests pass without modification
- [ ] TUI supports all core workflows: add, edit, delete, list, search secrets
- [ ] Clean terminal restoration on all exit paths (normal, error, panic)
- [ ] Documentation updated with TUI usage examples

## 6.1 TUI Style Sketch (Text Mockup)

Table-first layout with right-side preview. This is an ASCII sketch, not final UI.

```
DevInventory                     env: prod  db: ~/.devinv.db
─────────────────────────────────────────────────────────
 Name                 Type      Updated        │ Preview
> prod/aws/root       key       2026-01-31     │ Name: prod/aws/root
  prod/aws/readonly   key       2026-01-28     │ Type: key
  prod/github/token   token     2026-01-12     │ Updated: 2026-01-31
  staging/db/password secret    2025-12-30     │ Tags: aws, prod
─────────────────────────────────────────────────────────
 / search   Tab focus   Enter open   q quit
```

## 7. Appendix: Migration Checklist

**Pre-Migration:**

- [ ] All current work committed to `develop`
- [ ] CI/CD pipeline green
- [ ] Version bumped to 0.2.0-dev

**Post-Phase 1 (Foundation):**

- [ ] `cargo run --bin devinventory-cli -- --help` works
- [ ] `cargo test --lib` passes
- [ ] No `src/main.rs` exists (deleted, not just moved)

**Post-Phase 5 (Release):**

- [ ] GitHub releases contain both binaries
- [ ] `cargo install devinventory --bin devinventory-cli` works
- [ ] `cargo install devinventory --bin devinventory-tui --features tui` works

---

**Prepared by:** _Abel2333_

**Date:** 2026-01-29

**Target Release:** v0.2.0 (TUI Support)
