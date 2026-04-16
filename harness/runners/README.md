# Runners Module

## Purpose

Defines the Runner interface for executing tasks and commands. Provides abstraction for running opencode-rs and opencode implementations.

## Key Types

- **Runner**: Trait for executing commands
- **RunnerConfig**: Configuration for runner execution
- **ExecutionResult**: Outcome of a runner execution

## Directory Structure

```
runners/
├── src/
│   ├── lib.rs
│   └── <runner-implementation>.rs
└── Cargo.toml
```

## Usage

Runners implement the `Runner` trait to execute commands in both opencode-rs and opencode contexts. The harness compares execution results between implementations.

## Interface

```rust
pub trait Runner: Send + Sync {
    fn execute(&self, command: &str, cwd: &Path) -> Result<ExecutionResult>;
    fn get_version(&self) -> String;
}
```
