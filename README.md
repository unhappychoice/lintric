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

For more detailed installation and usage instructions for the CLI, see [cli/README.md](cli/README.md).

## Core Library

The core logic for metric calculation is provided by the `lintric-core` library. For details on the metrics and their explanations, see [core/README.md](core/README.md).


## Contributing

For guidelines on how to contribute, please see our [CONTRIBUTING.md](CONTRIBUTING.md).
