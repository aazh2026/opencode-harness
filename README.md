# opencode-harness

A neutral verification framework for comparing and validating behavioral consistency between opencode-rs and opencode. This project does not implement features for either side—it serves solely as a "judge" to record differences.

## Project Description

opencode-harness acts as a validation harness that:
- Defines task schemas and fixtures for testing
- Provides runner interfaces for executing comparative tests
- Collects and normalizes outputs from both implementations
- Reports mismatches, contract gaps, and regression candidates

**Core Constraint:** This repository only develops harness capabilities. It must not implement, complete, or replace product features of opencode-rs.

## Directory Structure

```
opencode-harness/
├── Cargo.toml                      # Rust workspace root configuration
├── README.md                       # This file
├── PRD.md                          # Product requirements document
├── harness/
│   ├── crates/
│   │   ├── core/                  # Core shared types
│   │   │   ├── src/
│   │   │   │   ├── lib.rs
│   │   │   │   ├── error.rs       # Error type definitions
│   │   │   │   ├── config/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── loader.rs  # Configuration loader
│   │   │   │   └── types/
│   │   │   │       ├── mod.rs
│   │   │   │       ├── task_status.rs
│   │   │   │       ├── failure_classification.rs
│   │   │   │       ├── path_convention.rs
│   │   │   │       └── environment.rs
│   │   │   └── Cargo.toml
│   │   └── cli/                   # CLI entry point
│   │       ├── src/
│   │       │   └── main.rs
│   │       └── Cargo.toml
│   ├── tasks/                     # Task definitions (schema)
│   │   ├── api/
│   │   ├── cli/
│   │   ├── permissions/
│   │   ├── recovery/
│   │   ├── session/
│   │   ├── web/
│   │   └── workspace/
│   ├── fixtures/                  # Test fixtures
│   │   └── projects/
│   ├── contracts/                 # Contract definitions
│   │   ├── api/
│   │   ├── cli/
│   │   ├── events/
│   │   ├── permissions/
│   │   ├── side_effects/
│   │   └── state_machine/
│   ├── runners/                   # Runner interfaces
│   │   ├── legacy/
│   │   ├── rust/
│   │   └── shared/
│   ├── comparators/               # Output comparators
│   ├── verifiers/                 # Behavior verifiers
│   ├── normalizers/               # Output normalizers
│   ├── providers/
│   │   ├── deterministic/
│   │   └── replay/
│   ├── golden/                    # Golden assets
│   │   ├── baselines/
│   │   ├── normalized/
│   │   └── raw/
│   ├── regression/                # Regression assets
│   │   ├── bugs/
│   │   ├── incidents/
│   │   └── issues/
│   ├── reports/                   # Report structures
│   │   ├── schemas/
│   │   └── templates/
│   ├── configs/
│   ├── workspaces/
│   └── ci/
├── docs/
│   ├── PRD/
│   │   ├── iterations/
│   │   └── split/
│   └── architecture/
├── iterations/                    # Iteration documents
│   └── iteration-1/
│       ├── spec_v1.md            # Iteration 1 specification
│       ├── gap-analysis.md
│       ├── plan_v1.md
│       ├── tasks_v1.md
│       └── tasks_v1.json
├── scripts/                       # Utility scripts
├── sessions/                      # Session data
│   └── iteration-1/
├── target/                        # Build output
└── artifacts/                    # Runtime artifacts
    └── run-<id>/
```

## Acceptance Commands

```bash
# Build the project
cargo build

# Run tests
cargo test

# Show CLI help
cargo run -- --help
```

## Specifications

- [Iteration 1 Specification](./iterations/iteration-1/spec_v1.md)
