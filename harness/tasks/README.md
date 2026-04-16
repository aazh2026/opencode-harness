# Tasks Module

## Purpose

Defines task schemas for opencode-rs vs opencode behavioral verification. Contains task definition structures in YAML/JSON format.

## Key Types

- **TaskDefinition**: Task schema with id, title, status, commands
- **TaskStatus**: Todo, InProgress, Done, ManualCheck, Blocked, Skipped

## Directory Structure

```
tasks/
├── <category>/
│   └── <task-id>.yaml   # Task definition files
```

## Usage

Task files are loaded by the core module to execute verification workflows. Each task defines expected behavior for comparison between opencode-rs and opencode implementations.

## Path Convention

Task definitions follow: `harness/tasks/**/*.yaml|json`
