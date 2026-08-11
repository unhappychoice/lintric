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
*   **Measure Dependency Detection Accuracy**:
    Reports precision and recall against hand-written expectations, so a change in what the
    analyzer detects is visible as a number rather than only as a changed snapshot. See
    [crates/accuracy/README.md](../../crates/accuracy/README.md) for the annotation format.
    ```bash
    cargo run -p lintric-accuracy              # print the report
    cargo run -p lintric-accuracy -- --check   # compare against the recorded baseline
    cargo run -p lintric-accuracy -- --update  # record the current numbers
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
