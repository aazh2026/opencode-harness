# Architecture Overview

## Five-Layer Architecture

The opencode-harness framework is organized into five distinct layers, each with specific responsibilities and clear module boundaries.

### Layer 1: Asset Management

**Purpose:** Manage golden fixtures, regression tests, and test artifacts.

| Module | Responsibility |
|--------|----------------|
| `harness/golden/` | Store baseline expected outputs for comparison |
| `harness/fixtures/` | Provide test fixture projects and data |
| `harness/regression/` | Store regression test cases and historical failures |

**Boundary:** Layer 1 only stores and retrieves assets; it does not execute or compare.

---

### Layer 2: Execution

**Purpose:** Execute tests and runners against target implementations.

| Module | Responsibility |
|--------|----------------|
| `harness/runners/` | Define runner interface for executing tasks |
| `harness/providers/` | Provide execution context and runtime environment |

**Boundary:** Layer 2 handles execution lifecycle but does not perform comparison or validation.

---

### Layer 3: Comparison

**Purpose:** Compare outputs and verify behavior consistency.

| Module | Responsibility |
|--------|----------------|
| `harness/comparators/` | Compare actual outputs against expected baselines |
| `harness/verifiers/` | Verify behavior consistency between implementations |
| `harness/normalizers/` | Normalize output format for consistent comparison |

**Boundary:** Layer 3 takes execution results and produces comparison reports.

---

### Layer 4: Governance

**Purpose:** Define contracts, generate reports, and track governance data.

| Module | Responsibility |
|--------|----------------|
| `harness/contracts/` | Define behavioral contracts between components |
| `harness/reports/` | Generate structured test reports and audit trails |

**Boundary:** Layer 4 captures metadata and does not modify execution or comparison logic.

---

### Layer 5: Integration

**Purpose:** Provide external interfaces and CI/CD integration.

| Module | Responsibility |
|--------|----------------|
| `harness/crates/cli/` | CLI entry point for human and script interaction |
| `harness/ci/` | CI/CD configuration and integration scripts |

**Boundary:** Layer 5 is the outermost layer, orchestrating all other layers through user-facing interfaces.

---

## Module Boundary Table

| Layer | Components | Inputs | Outputs |
|-------|------------|--------|---------|
| Layer 1 | golden, fixtures, regression | Asset definitions | Raw assets |
| Layer 2 | runners, providers | Assets, configuration | Execution results |
| Layer 3 | comparators, verifiers, normalizers | Execution results | Comparison reports |
| Layer 4 | contracts, reports | Comparison reports, contracts | Governance artifacts |
| Layer 5 | CLI, CI | User commands, triggers | Final reports, exit codes |

---

## Data Flow Diagram

```
                           ASCII Data Flow Diagram
                           ========================

    +--------+      +------------------------------------------------------+
    |   CI   |      |                      Layer 5: Integration             |
    +--------+      |  +--------------------------------------------+       |
        |           |  |                  CLI                       |       |
        |           |  +--------------------------------------------+       |
        |           +------------------------------------------------------+
        |                                    |
        v                                    v
    +---------------------------------------------------------------+
    |                        Layer 4: Governance                     |
    |  +----------------+              +-------------------------+    |
    |  |   contracts/   |              |        reports/         |    |
    |  +----------------+              +-------------------------+    |
    +---------------------------------------------------------------+
                                    |
                                    v
    +---------------------------------------------------------------+
    |                       Layer 3: Comparison                     |
    |  +----------------+  +----------------+  +------------------+   |
    |  | comparators/   |  |  verifiers/   |  |  normalizers/   |   |
    |  +----------------+  +----------------+  +------------------+   |
    +---------------------------------------------------------------+
                                    |
                                    v
    +---------------------------------------------------------------+
    |                        Layer 2: Execution                      |
    |  +----------------+              +-------------------------+    |
    |  |   runners/     |              |       providers/       |    |
    |  +----------------+              +-------------------------+    |
    +---------------------------------------------------------------+
                                    |
                                    v
    +---------------------------------------------------------------+
    |                     Layer 1: Asset Management                  |
    |  +----------------+  +----------------+  +------------------+   |
    |  |    golden/     |  |   fixtures/    |  |   regression/   |   |
    |  +----------------+  +----------------+  +------------------+   |
    +---------------------------------------------------------------+

    Flow: CI/CLI --> Layer 5 --> Layer 4 --> Layer 3 --> Layer 2 --> Layer 1
          CLI --> Layer 5 --> Layer 3 --> Layer 2 --> Layer 1
          Layer 1 --> Layer 2 --> Layer 3 --> Layer 4 --> Layer 5 --> Reports
```

---

## Key Design Principles

1. **Layer Isolation:** Each layer only depends on layers beneath it
2. **Single Responsibility:** Each module has one clearly defined purpose
3. **Testability:** Clear boundaries enable isolated testing of each layer
4. **Extensibility:** New modules can be added to any layer without modifying others
