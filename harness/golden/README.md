# Golden Module

## Purpose

Stores golden artifacts - reference outputs that represent expected behavior. Golden files serve as baseline comparisons for verification.

## Directory Structure

```
golden/
└── <suite>/
    └── <test-case>.golden  # Reference output files
```

## Usage

Golden artifacts are created during known-good runs and stored for future comparisons. They represent:
- Expected output for given inputs
- Reference error messages
- Baseline performance metrics

## Path Convention

Golden files follow: `harness/golden/<suite>/<name>.golden`
