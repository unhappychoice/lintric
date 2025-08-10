# Lintric Core

The `lintric-core` crate provides the core logic for calculating code metrics based on line-level dependencies.

## Overview

Lintric Core processes source code files to build an Abstract Syntax Tree (AST) using `tree-sitter`, identifies dependencies between lines, constructs a directed graph representing these dependencies, and then calculates various code metrics based on this graph.

It currently supports:
- Rust (`.rs`)
- TypeScript (`.ts`)
- TSX (`.tsx`)

## Features

- **AST Parsing**: Utilizes `tree-sitter` to parse source code into an Abstract Syntax Tree.
- **Dependency Graph Construction**: Identifies and maps line-level dependencies within the code, building a directed graph using `petgraph`.
- **Metric Calculation**: Computes various metrics based on the dependency graph:
    - **Total Dependencies**: Number of lines each line depends on.
    - **Dependency Distance Cost**: Cost based on the distance (line numbers) between dependent lines.
    - **Dependency Tree Complexity**:
        - **Depth**: Maximum depth of the dependency tree.
        - **Transitive Dependency Size**: Number of transitive dependencies.
    - **Overall Complexity Score**: A combined metric derived from the above.

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