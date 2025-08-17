## Debugging Methods

*   **Display Abstract Syntax Tree (AST) of a file**:
    ```bash
    cargo run -p lintric-cli -- debug ast {fileName}
    ```
*   **Display Intermediate Representation (IR) of a file**:
    ```bash
    cargo run -p lintric-cli -- debug ir {fileName}
    ```
## Testing and Quality

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
    cargo clippy --workspace
    ```
    To fix clippy warnings, run `cargo clippy --workspace --fix`.
*   **Update Snapshots (for tests)**:
    ```bash
    cargo insta accept --workspace
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
