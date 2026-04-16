# Regression Module

## Purpose

Stores regression test cases that capture identified behavioral differences. These cases document known mismatches between opencode-rs and opencode.

## Directory Structure

```
regression/
└── <suite>/
    └── <test-case>.yaml  # Regression case definitions
```

## Usage

Regression cases are added when differences are discovered and accepted as known issues. Each case documents:
- Issue description
- Affected versions
- Workaround if available
- Related contract gaps

## Path Convention

Regression cases follow: `harness/regression/<suite>/<name>.yaml`
