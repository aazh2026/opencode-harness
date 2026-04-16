# Comparators Module

## Purpose

Provides comparison logic for execution outputs. Compares results between opencode-rs and opencode to identify behavioral differences.

## Key Types

- **Comparator**: Trait for comparing outputs
- **ComparisonResult**: Outcome of comparison
- **Diff**: Structured difference representation

## Directory Structure

```
comparators/
├── src/
│   ├── lib.rs
│   └── <comparator-implementation>.rs
└── Cargo.toml
```

## Usage

Comparators take execution results from runners and produce structured diffs. Different comparators may handle:
- Exit code comparison
- Output text diffing
- JSON/structured data comparison

## Interface

```rust
pub trait Comparator: Send + Sync {
    fn compare(&self, expected: &ExecutionResult, actual: &ExecutionResult) -> ComparisonResult;
}
```
