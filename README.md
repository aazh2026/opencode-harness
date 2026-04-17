# opencode-harness

`opencode-harness` is a dedicated verification project for checking whether **`opencode-rs` behaves consistently with `opencode`**.

It is not a product implementation repository for either side. Its job is to act as a neutral judge: define comparison tasks, execute both sides, collect artifacts, classify differences, and report what is aligned versus what is not.

## What this project is for

The purpose of this repository is to answer questions like:

- Does `opencode-rs` expose the same CLI contract as `opencode`?
- Does `opencode-rs` produce the same meaningful behavior for the same task and fixture?
- Are differences caused by implementation gaps, environment issues, or allowed variance?
- Can we turn discovered mismatches into repeatable regression assets?

In short:

> `opencode-harness` exists to validate parity between `opencode-rs` and `opencode`.

## What this project is not

This repository must **not**:

- implement `opencode-rs` product features
- replace or patch `opencode`
- act as a second development repository for `opencode-rs`
- absorb business logic that belongs in the systems under test

If a mismatch is found, the harness should **record it, classify it, and report it**. It should not “fix `opencode-rs` from inside the harness project”.

## Current maturity

`opencode-harness` is under active iteration.

Current state, in practical terms:

- it already has a real project structure
- it already has core schemas, runners, comparator/verifier foundations, and report structures
- it can support manual real-world parity checks
- it is still moving toward a fully reliable automated comparison workflow

That means the project is already useful for:
- designing and iterating the parity-testing workflow
- running small real checks manually or semi-manually
- building out the automated differential execution pipeline

But it should not yet be described as a fully mature, production-grade parity platform.

## Core workflow

The intended workflow is:

1. define a task
2. prepare a fixture/workspace
3. execute the same task against `opencode`
4. execute the same task against `opencode-rs`
5. collect stdout / stderr / exit codes / artifacts / side effects
6. normalize irrelevant differences
7. compare meaningful outputs and behavior
8. verify against contracts and expectations
9. emit a verdict and report

Typical verdicts include:
- `pass`
- `fail`
- `manual_check`
- `blocked`
- `pass_with_allowed_variance`

## Repository layout

Core project directories are intentionally grouped under `harness/`, while support material stays at the repository root.

```text
opencode-harness/
├── Cargo.toml
├── README.md
├── PRD.md
├── iterate-prd.sh
├── harness/
│   ├── crates/
│   │   ├── core/
│   │   └── cli/
│   ├── tasks/
│   ├── fixtures/
│   ├── contracts/
│   ├── runners/
│   ├── comparators/
│   ├── verifiers/
│   ├── normalizers/
│   ├── providers/
│   ├── golden/
│   ├── regression/
│   ├── reports/
│   ├── configs/
│   ├── workspaces/
│   └── ci/
├── docs/
│   ├── PRD/
│   └── architecture/
├── scripts/
├── iterations/
├── sessions/
└── artifacts/
```

## Key directories

### `harness/crates/`
Rust implementation for harness-owned logic.

- `core/`: shared types, execution primitives, verdict/artifact models, and other harness internals
- `cli/`: CLI entry point for harness operations

### `harness/tasks/`
Task definitions and task schema. These describe *what* parity check should be run.

### `harness/fixtures/`
Fixture definitions and example projects. These define the controlled environments used for parity checks.

### `harness/contracts/`
Behavioral contracts the harness will verify, such as CLI/API/permission/state/side-effect expectations.

### `harness/runners/`
Execution layer for driving:
- `opencode` (reference side)
- `opencode-rs` (target side)

### `harness/comparators/`, `harness/verifiers/`, `harness/normalizers/`
The judging layer:
- comparators decide what differs
- normalizers remove irrelevant noise
- verifiers classify the result into meaningful verdicts

### `harness/golden/` and `harness/regression/`
Long-lived testing assets:
- baselines
- normalized outputs
- regression cases
- historical mismatch preservation

### `docs/PRD/`
The project planning and iteration documents live here.

Important files include:
- `docs/PRD/00-overview.md`
- `docs/PRD/02-supplement-remaining-work-for-production-readiness.md`
- `docs/PRD/iterations/*.md`

## Build and test

```bash
# Build the workspace
cargo build

# Run tests
cargo test

# Show harness CLI help
cargo run -- --help
```

## Planning documents

Useful starting points:

- [PRD Overview](./docs/PRD/00-overview.md)
- [Production Readiness Supplement](./docs/PRD/02-supplement-remaining-work-for-production-readiness.md)
- [Iteration 1 Specification](./iterations/iteration-1/spec_v1.md)

## Practical note on iteration numbering

This repository currently has two numbering notions that can differ:

1. **PRD stage number**
   - the conceptual phase in `docs/PRD/iterations/*.md`
2. **actual output directory number**
   - the concrete `iterations/iteration-N/` folder created by the iteration script

When reporting progress, always distinguish the two when they diverge.

## Development rule that matters most

If you only remember one rule, remember this one:

> `opencode-harness` develops the testing and verification system itself, not the product behavior of `opencode-rs`.
