# Verifiers Module

## Purpose

Verifies behavioral consistency between opencode-rs and opencode implementations. Applies contracts and comparison results to determine pass/fail status.

## Key Types

- **Verifier**: Trait for verifying consistency
- **VerificationResult**: Outcome of verification
- **VerificationContext**: Context for verification process

## Directory Structure

```
verifiers/
├── src/
│   ├── lib.rs
│   └── <verifier-implementation>.rs
└── Cargo.toml
```

## Usage

Verifiers consume contracts, comparison results, and execution outputs to produce final verification verdicts. Each verifier implements domain-specific validation logic.

## Interface

```rust
pub trait Verifier: Send + Sync {
    fn verify(&self, context: &VerificationContext) -> VerificationResult;
}
```
