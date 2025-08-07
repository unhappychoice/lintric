# Lintric Specification and Technologies

## 1. Specification

This section outlines the core specifications and design principles of Lintric.

### 1.1. Purpose
Lintric is a tool designed to calculate code metrics based on line-level dependencies within source code, inspired by the concepts presented in "行単位依存関係に基づくコードメトリクスの定義（案）" (https://zenn.dev/unhappychoice/articles/0f2437226fe802). It aims to provide insights into code complexity and maintainability by analyzing how different lines of code depend on each other.

### 1.2. Input
- Source code files (initially Rust, with potential for expansion to other languages).

### 1.3. Output
- Total Dependencies: Number of lines each line depends on.
- Dependency Distance Cost: Cost based on the distance (line numbers) between dependent lines.
- Dependency Tree Complexity:
    - Depth: Maximum depth of the dependency tree.
    - Transitive Dependency Size: Number of transitive dependencies.
- Overall Complexity Score: A combined metric derived from the above.
- Output formats: Console, JSON.

### 1.4. Core Logic
- Line-by-line source code analysis.
- Identification of dependencies (variables, function calls, imports).
- Graph representation of dependencies.
- Metric calculation based on the dependency graph.

### 1.5. Architecture
Lintric will initially be developed as a monolithic application. However, the design will allow for future separation of the core logic (e.g., code analysis, metric calculation) into a distinct `core` package and the command-line interface into a `cli` package. This modular approach aims to facilitate reusability and independent development of components in the long term.

## 2. Technologies Used

This section lists the primary technologies and libraries utilized in the development of Lintric.

### 2.1. Programming Language
- **Rust**: Chosen for its performance, memory safety, and strong type system, which are crucial for efficient code analysis.

### 2.2. Key Libraries/Frameworks
- **`cargo`**: Rust's package manager and build system.
- **`tree-sitter`**: A parser generator tool and incremental parsing library. It will be used for robust and efficient Abstract Syntax Tree (AST) parsing of source code, enabling accurate dependency identification.
- **`clap`**: A powerful command-line argument parser for Rust, used for building the CLI interface.
- **`petgraph`**: A graph data structure library for Rust, used for representing and manipulating the dependency graph.

### 2.3. Development Tools
- **Git**: Version control system.
- **GitHub**: Code hosting and collaboration platform.

---

*This document is a living specification and will be updated as the project evolves.*