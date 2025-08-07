# Lintric: Implementation Tasks for Line-based Dependency Code Metrics Tool

Based on the article: [行単位依存関係に基づくコードメトリクスの定義（案）](https://zenn.dev/unhappychoice/articles/0f2437226fe802)

## 1. Project Initialization
- [ ] Set up Rust project (e.g., add dependencies to `Cargo.toml`)
- [ ] Create necessary directory structure (`src/`, `docs/`, etc.)

## 2. Code Analysis Feature Implementation
- [ ] Function to read source code line by line
- [ ] Function to identify dependencies for each line
    - [ ] Identify variable definitions
    - [ ] Identify function calls
    - [ ] Identify import statements
    - [ ] Consider and introduce AST (Abstract Syntax Tree) parsing
    - [ ] Consider and introduce regular expression pattern matching
- [ ] Design and implement data structure to represent dependencies as a directed graph

## 3. Metrics Calculation Feature Implementation
- [ ] **Total Dependencies**: Function to calculate the total number of lines each line depends on
- [ ] **Dependency Distance Cost**: Function to calculate cost based on dependency distance (difference in line numbers)
- [ ] **Dependency Tree Complexity**:
    - [ ] Depth: Function to calculate the maximum depth of the dependency tree
    - [ ] Transitive Dependency Size: Function to calculate the number of transitive dependencies

## 4. Complexity Score Calculation
- [ ] Implement a formula to calculate the file's complexity score by combining the above metrics

## 5. Output Functionality
- [ ] Function to output calculated metrics and complexity score to the console
- [ ] Function to output calculated metrics and complexity score in JSON format

## 6. Testing
- [ ] Create and execute unit tests for each feature
- [ ] Create and execute integration tests for actual codebases

## 7. Documentation
- [ ] Create `README.md` describing tool usage
- [ ] Create `README.md` describing calculated metrics
- [ ] Create `README.md` describing installation instructions