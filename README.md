# Lintric

Lintric is a tool designed to calculate code metrics based on line-level dependencies within source code.

## Features

- **Total Dependencies**: Number of lines each line depends on.
- **Dependency Distance Cost**: Cost based on the distance (line numbers) between dependent lines.
- **Dependency Tree Complexity**:
    - **Depth**: Maximum depth of the dependency tree.
    - **Transitive Dependency Size**: Number of transitive dependencies.
- **Overall Complexity Score**: A combined metric derived from the above.

## Installation

To install Lintric, you need to have Rust and Cargo installed. If you don't have them, you can install them from [rustup.rs](https://rustup.rs/).

Once Rust and Cargo are installed, you can build and run Lintric from the source:

```bash
git clone https://github.com/unhappychoice/lintric.git
cd lintric
cargo build --release
```

This will create an executable in `target/release/lintric`.

## Usage

To analyze a Rust source code file, run the following command:

```bash
target/release/lintric <path_to_your_file.rs>
```

For example:

```bash
target/release/lintric src/main.rs
```

To output the results in JSON format, use the `--json` flag:

```bash
target/release/lintric --json <path_to_your_file.rs>
```

## Metrics Explained

### Total Dependencies
This metric represents the total number of lines that a given line of code directly depends on. A higher number indicates more direct dependencies, potentially suggesting higher coupling.

### Dependency Distance Cost
This metric calculates a cost based on the distance (line numbers) between dependent lines. Dependencies that span a greater number of lines incur a higher cost. This can indicate that related code is not co-located, potentially making the code harder to understand and maintain.

### Dependency Tree Complexity - Depth
This metric measures the maximum depth of the dependency tree originating from a given line. A deeper tree suggests a longer chain of dependencies, which might imply a more complex flow of control or data, and potentially more difficult debugging.

### Dependency Tree Complexity - Transitive Dependency Size
This metric counts the total number of lines that a given line transitively depends on (i.e., direct dependencies, their dependencies, and so on). A larger transitive dependency size indicates a broader impact of changes to that line, suggesting higher overall coupling and potential for ripple effects.

### Overall Complexity Score
This is a combined metric that aggregates the above individual metrics into a single score. It provides a holistic view of the line's complexity and maintainability. The exact weighting of individual metrics can be adjusted based on further analysis and project needs.
