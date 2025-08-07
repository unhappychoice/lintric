# Line-Level Dependency Code Metrics Definition

This document defines the code metrics calculated by Lintric, focusing on line-level dependencies within source code. These metrics aim to provide insights into code modifiability and maintainability.

## 1. Definition of Line-Level Dependencies

Each line of code can depend on other lines through various forms, including variable references, function calls, class instantiations, and module imports.

### 1.1. Direct Dependency

If line A directly references a variable or function defined in line B, then line A is directly dependent on line B.

Example:
```rust
1: let x = 10;
2: let y = x + 5; // Line 2 directly depends on Line 1
```

### 1.2. Indirect Dependency (Transitive Dependency)

If line A directly depends on line B, and line B directly depends on line C, then line A is indirectly dependent on line C.

Example:
```rust
1: let a = 1;
2: let b = a + 1; // Line 2 directly depends on Line 1
3: let c = b + 1; // Line 3 directly depends on Line 2, and indirectly depends on Line 1
```

## 2. Code Metrics

Based on line-level dependencies, Lintric calculates the following metrics:

### 2.1. Total Dependencies

The total number of other lines that a given line directly or indirectly depends on. A larger value indicates a higher likelihood that changes to that line will affect many other lines.

### 2.2. Dependency Distance Cost

The sum of the differences in line numbers between a given line and each line it depends on. By assigning a higher cost to dependencies that are further apart, this metric evaluates code locality. A larger value suggests that related code is scattered across distant locations, potentially making it harder to understand and maintain.

Example:
```rust
1: fn foo() { ... }
10: foo(); // Dependency Distance Cost = |10 - 1| = 9
```

### 2.3. Dependency Tree Complexity

#### 2.3.1. Depth

The length of the longest path of dependencies starting from a given line. A larger value indicates that the line is at the end of many chained dependencies, suggesting a broader impact of changes.

#### 2.3.2. Transitive Dependency Size

The size of the set of all lines that a given line directly or indirectly depends on. Similar to Total Dependencies, but this more clearly indicates the "spread" of dependencies.

## 3. Overall Complexity Score

This is a combined metric that aggregates the above individual metrics into a single score. It provides a holistic view of the line's complexity and maintainability. The exact weighting of individual metrics can be adjusted based on further analysis and project needs.

Example:
`Complexity Score = w1 * (Total Dependencies) + w2 * (Dependency Distance Cost) + w3 * (Depth) + w4 * (Transitive Dependency Size)`
(w1, w2, w3, w4 are weighting coefficients)