# Contracts Module

## Purpose

Defines behavioral contracts between opencode-rs and opencode. Contracts specify expected behavior, interfaces, and acceptance criteria.

## Key Types

- **Contract**: Definition of expected behavior
- **ContractAssertion**: Verification criteria

## Directory Structure

```
contracts/
└── <contract-name>.yaml  # Contract definitions
```

## Usage

Contracts are loaded by verifiers to validate behavior consistency. Each contract specifies:
- Input/output expectations
- Error handling requirements
- Performance criteria

## Path Convention

Contracts follow: `harness/contracts/**/*.yaml|json`
