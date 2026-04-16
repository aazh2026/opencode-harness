# Fixtures Module

## Purpose

Provides test fixtures for verification workflows. Contains sample projects and test data used across different test scenarios.

## Key Types

- **FixtureProject**: Sample project structure for testing
- **FixtureConfig**: Harness configuration for fixture projects

## Directory Structure

```
fixtures/
└── projects/
    └── <name>/
        └── harness.toml  # Fixture-specific config
```

## Usage

Fixtures are referenced by task definitions and provide standardized test inputs. Each fixture project may contain:
- Source code samples
- Configuration files
- Expected outputs

## Path Convention

Fixtures follow: `harness/fixtures/projects/<name>/`
