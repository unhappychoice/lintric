# Contributing to Lintric

We welcome contributions to Lintric! By following these guidelines, you can help us maintain a consistent and high-quality codebase.

## How to Contribute

### Bug Reports

If you find a bug, please open an [issue on GitHub](https://github.com/unhappychoice/lintric/issues). Before opening a new issue, please check if a similar issue already exists.

### Feature Requests

If you have a feature request, please open an [issue on GitHub](https://github.com/unhappychoice/lintric/issues). Describe the feature, why it's needed, and how it would benefit the project.

### Pull Requests

1.  **Fork the repository** and clone it to your local machine.
2.  **Create a new branch** for your changes: `git checkout -b feature/your-feature-name` or `git checkout -b bugfix/your-bug-fix`.
3.  **Make your changes**, adhering to the project's coding style and conventions.
4.  **Write tests** for your changes, if applicable.
5.  **Run tests and checks** to ensure everything passes. See [Testing and Quality](#testing-and-quality) for more details.
6.  **Commit your changes** using a clear and concise commit message. See [Commit Message Guidelines](#commit-message-guidelines) for more details.
7.  **Push your branch** to your forked repository.
8.  **Open a Pull Request** to the `main` branch of the original repository.

## Development Setup

### Prerequisites

*   Rust (install via [rustup.rs](https://rustup.rs/))
*   Cargo (comes with Rust installation)

### Project Structure

Lintric is organized into two main crates:

*   **`core/`**: Contains the core logic for calculating code metrics based on line-level dependencies. It uses `tree-sitter` for AST parsing and `petgraph` for dependency graph construction.
*   **`cli/`**: Provides a command-line interface for the Lintric tool, utilizing the `lintric-core` library.

## Testing and Quality

For detailed information on running tests, checking formatting, running linters, and generating reports, please refer to the [Commands documentation](docs/development/commands.md#testing-and-quality).

## Debugging

For detailed information on debugging methods, such as displaying AST and IR, please refer to the [Commands documentation](docs/development/commands.md#debugging-methods).

## Implementing New Language Features

If you are implementing dependency collection for a new language feature, please refer to the [Implementing New Language Features documentation](docs/development/implementing.md).

## Commit Message Guidelines

We follow a [conventional commit style](https://www.conventionalcommits.org/en/v1.0.0/) for our commit messages.
