# Repository Guidelines

## Project Structure & Modules
- Root is a Cargo workspace. Code lives in two crates:
  - `core/` (library): metric engine. Key modules: `dependency_graph_builder.rs`, `metric_calculator.rs`, `models.rs`, and `src/collectors/*`.
  - `cli/` (binary): command-line app using `lintric-core`. Key files: `main.rs`, `display.rs`, `file_processor.rs`, `html_output.rs`, and `templates/`.
- Tests: `core/tests/**` (Rust, TypeScript fixtures, integration) and `cli/tests/**` (CLI + `insta` snapshots in `cli/tests/snapshots/`).
- CI: `.github/workflows/*` for fmt, clippy, tests, coverage, audit.
- Generated artifacts: HTML under `.lintric/output/html/index.html`; scratch space in `tmp/`.

## Build, Test, and Development
- Build workspace: `cargo build --workspace` (use `--release` for optimized binary).
- Run CLI locally: `cargo run --package lintric-cli -- <path> [--json|--html|--verbose]`.
- Test all crates: `cargo test --workspace`.
- Update `insta` snapshots: `INSTA_UPDATE=always cargo test --workspace`.
- Format check/fix: `cargo fmt -- --check` / `cargo fmt`.
- Lint: `cargo clippy --workspace -- -D warnings` (fix: `cargo clippy --workspace --fix`).
- Coverage (requires `cargo-tarpaulin`): `cargo tarpaulin --workspace --out Lcov`.
- Security audit (requires `cargo-audit`): `cargo audit`.

## Coding Style & Naming
- Rust 2021 edition; 4-space indentation; keep functions small and focused.
- Filenames and modules: `snake_case`; types and traits: `PascalCase`; functions/vars: `snake_case`.
- Prefer pure functions in `core`; isolate I/O and rendering in `cli`.
- Keep public APIs in `lintric-core` documented; avoid breaking changes without discussion.

## Testing Guidelines
- Use `insta` for CLI output and engine snapshots.
- Place CLI tests in `cli/tests/`; engine/integration tests in `core/tests/`.
- Name tests descriptively (e.g., `test_json_output`, `metric_calculator_test`).
- When changing output formats, update snapshots and review diffs.

## Commit & Pull Requests
- Commits: follow Conventional Commits (`feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`). Keep messages imperative and scoped.
- PRs: target `main`; include a clear description, linked issues (e.g., `Closes #123`), and before/after output or screenshot for HTML/report changes.
- CI must pass (fmt, clippy, tests). Add tests for new behavior and update docs where relevant.

## Notes & Tips
- Tree-sitter grammars: Rust/TS/TSX supported; add new languages under `core/src/collectors/` with tests and fixtures.
- HTML output writes to `.lintric/`; do not commit generated files.
