# Normalizers Module

## Purpose

Normalizes outputs from different implementations to enable fair comparison. Handles platform-specific differences, timestamps, and non-deterministic content.

## Key Types

- **Normalizer**: Trait for normalizing outputs
- **NormalizedOutput**: Standardized output representation

## Directory Structure

```
normalizers/
├── src/
│   ├── lib.rs
│   └── <normalizer-implementation>.rs
└── Cargo.toml
```

## Usage

Normalizers preprocess execution outputs before comparison. They handle:
- Timestamp normalization
- Path separator normalization
- Random seed normalization
- Version string normalization

## Interface

```rust
pub trait Normalizer: Send + Sync {
    fn normalize(&self, output: &str) -> NormalizedOutput;
}
```
