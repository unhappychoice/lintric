# Contributing to Lintric

We welcome contributions to Lintric! This document outlines the guidelines for contributing to the project.

## How to Contribute

### Bug Reports

If you find a bug, please open an issue on GitHub. Before opening a new issue, please check if a similar issue already exists.

### Feature Requests

If you have a feature request, please open an issue on GitHub. Describe the feature, why it's needed, and how it would benefit the project.

### Pull Requests

1.  **Fork the repository** and clone it to your local machine.
2.  **Create a new branch** for your changes: `git checkout -b feature/your-feature-name` or `git checkout -b bugfix/your-bug-fix`.
3.  **Make your changes**, adhering to the project's coding style and conventions.
4.  **Write tests** for your changes, if applicable.
5.  **Run tests and checks** to ensure everything passes.
6.  **Commit your changes** using a clear and concise commit message (see [Commit Message Guidelines](#commit-message-guidelines)).
7.  **Push your branch** to your forked repository.
8.  **Open a Pull Request** to the `main` branch of the original repository.

## Development Setup

### Prerequisites

*   Rust (install via [rustup.rs](https://rustup.rs/))
*   Cargo (comes with Rust installation)

## Project Structure

Lintric is organized into two main crates:

*   **`core/`**: Contains the core logic for calculating code metrics based on line-level dependencies. It uses `tree-sitter` for AST parsing and `petgraph` for dependency graph construction.
*   **`cli/`**: Provides a command-line interface for the Lintric tool, utilizing the `lintric-core` library.

## Testing and Quality

Before submitting a pull request, please ensure all tests pass and code quality checks are met.

*   **Build**:
    ```bash
    cargo build --release
    ```
*   **Run Tests**:
    ```bash
    cargo test --workspace
    ```
*   **Check Formatting**:
    ```bash
    cargo fmt -- --check
    ```
    To fix formatting issues, run `cargo fmt`.
*   **Run Clippy (Linter)**:
    ```bash
    cargo clippy --workspace -- -D warnings
    ```
    To fix clippy warnings, run `cargo clippy --workspace --fix`.
*   **Update Snapshots (for tests)**:
    ```bash
    INSTA_UPDATE=always cargo test --workspace
    ```
*   **Generate Code Coverage Report**:
    (Requires `cargo-tarpaulin` to be installed: `cargo install cargo-tarpaulin`)
    ```bash
    cargo tarpaulin --workspace --out Lcov
    ```
*   **Perform Security Audit**:
    (Requires `cargo-audit` to be installed: `cargo install cargo-audit`)
    ```bash
    cargo audit
    ```

## Commit Message Guidelines

We follow a conventional commit style for our commit messages.
