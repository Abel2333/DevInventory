# Repository Guidelines

## Project Structure & Module Organization
- Rust binary crate; entry point in `src/main.rs`.
- `Cargo.toml` defines metadata and dependencies. Add new modules under `src/` with snake_case filenames and expose via `mod` declarations.
- Place integration tests in `tests/` and fixtures under `tests/fixtures/` if added.

## Build, Test, and Development Commands
- `cargo build` — compile the crate in debug mode.
- `cargo run` — build then run the binary (useful for quick checks).
- `cargo test` — execute unit and integration tests.
- `cargo fmt` — format code with rustfmt; run before committing.
- `cargo clippy` — lint for common pitfalls; treat warnings as errors when possible (`cargo clippy -- -D warnings`).

## Coding Style & Naming Conventions
- Follow Rust 2024 edition defaults (4-space indent, trailing commas where appropriate).
- Filenames and modules: snake_case (e.g., `dev_inventory.rs`).
- Types and traits: UpperCamelCase (`InventoryItem`).
- Functions, variables, and fields: snake_case (`load_config`).
- Prefer `?` for error propagation and small functions for clarity; document public functions with `///` doc comments.

## Testing Guidelines
- Unit tests live beside source in `src/` inside `#[cfg(test)]` modules; integration tests belong in `tests/` using descriptive filenames.
- Name tests with intent (`handles_empty_input`, `formats_inventory_row`).
- Aim for coverage of edge cases and happy paths; add regression tests for any bugfix.
- Keep tests deterministic—avoid network or time dependencies without mocking.

## Commit & Pull Request Guidelines
- Existing history is minimal; adopt Conventional Commits (`feat:`, `fix:`, `chore:`) for clarity.
- Commits should be small and focused; include `cargo fmt` and `cargo clippy` clean state before committing.
- Pull requests: include summary of changes, steps to reproduce/fix, and reference related issues. Add screenshots or sample output when behavior changes.

## Security & Configuration Tips
- Do not commit secrets or API keys; prefer environment variables or local config files ignored by git.
- Review new dependencies for license compatibility and minimal footprint; document why each is needed in the PR.
