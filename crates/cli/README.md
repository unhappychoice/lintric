# Lintric CLI

The `lintric-cli` crate provides a command-line interface for the Lintric code metrics tool.

## Installation

To install Lintric CLI, you need to have Rust and Cargo installed. If you don't have them, you can install them from [rustup.rs](https://rustup.rs/).

Once Rust and Cargo are installed, you can build and run Lintric from the source:

```bash
git clone https://github.com/unhappychoice/lintric.git
cd lintric/cli
cargo build --release
```

This will create an executable in `target/release/lintric`.

## Usage

Lintric CLI can analyze single files or entire directories, and provides various output formats.

### Basic Usage

To analyze a Rust source code file, run the following command:

```bash
target/release/lintric <path_to_your_file.rs>
```

For example:

```bash
target/release/lintric src/main.rs
```

To analyze a directory:

```bash
target/release/lintric src/
```

### Output Formats

#### JSON Output

To output the results in JSON format, use the `--json` flag:

```bash
target/release/lintric --json <path_to_your_file.rs>
```

#### Verbose Output

To show verbose output with line-by-line metrics, use the `--verbose` flag:

```bash
target/release/lintric --verbose <path_to_your_file.rs>
```

#### HTML Report

To generate an HTML report, use the `--html` flag. The report will be generated in `.lintric/output/html/index.html`.

```bash
target/release/lintric --html <path_to_your_file_or_directory>
```