# Iteration 2 Task List

**Project:** opencode-harness  
**Version:** 2.1  
**Date:** 2026-04-16  
**Priority:** P0 = Must Complete, P1 = Should Complete, P2 = Next Iteration

---

## P0 Blockers

| # | Task | Module | Status | Dependencies |
|---|------|--------|--------|--------------|
| P0-1 | Add EntryMode enum (CLI, API, Session, Permissions, Web, Workspace, Recovery) | types/ | ✅ Done | None |
| P0-2 | Add AgentMode enum (Interactive, Batch, Daemon, OneShot) | types/ | TODO | None |
| P0-3 | Add ProviderMode enum (OpenCode, OpenCodeRS, Both, Either) | types/ | TODO | None |
| P0-4 | Add Severity enum (critical, high, medium, low, cosmetic) | types/ | TODO | None |
| P0-5 | Create Assertion model with 6 types (exit_code_equals, stdout_contains, stderr_contains, file_changed, no_extra_files_changed, permission_prompt_seen) | types/assertion.rs | ✅ Done | None |
| P0-6 | Create ExecutionPolicy enum (manual_check, blocked, skip) | types/execution_policy.rs | TODO | None |
| P0-7 | Create OnMissingDependency enum (fail, skip, warn, blocked) | types/on_missing_dependency.rs | TODO | None |
| P0-8 | Create TaskInput struct (command, args, cwd) | types/ | TODO | None |
| P0-9 | Create AllowedVariance struct (exit_code, timing_ms, output_patterns) | types/ | TODO | None |
| P0-10 | Create TaskOutputs struct (stdout, stderr, files_created, files_modified) | types/ | TODO | None |
| P0-11 | Extend Task struct with all PRD required fields | types/task.rs | ✅ Done | P0-1 through P0-10 |
| P0-12 | Implement TaskSchemaValidator trait | loaders/task_validator.rs | ✅ Done | P0-5, P0-6, P0-7, P0-11 |
| P0-13 | Implement FixtureLoader trait | loaders/fixture_loader.rs | TODO | P0-14 |
| P0-14 | Add WorkspacePolicy struct to Fixture schema | types/fixture.rs | TODO | None |
| P0-15 | Add ResetStrategy enum to Fixture schema | types/fixture.rs | TODO | None |
| P0-16 | Complete FixtureProject struct with all missing fields | types/fixture.rs | TODO | P0-14, P0-15 |
| P0-17 | Create cli-basic fixture project with harness.toml and scripts | fixtures/projects/cli-basic/ | TODO | P0-13 |
| P0-18 | Create api-project fixture project with harness.toml and scripts | fixtures/projects/api-project/ | TODO | P0-13 |
| P0-19 | Create SMOKE-CLI-001 (CLI help command) | tasks/cli/ | TODO | P0-11 |
| P0-20 | Create SMOKE-CLI-002 (CLI version command) | tasks/cli/ | TODO | P0-11 |
| P0-21 | Create SMOKE-WS-001 (Workspace init clean) | tasks/workspace/ | TODO | P0-11 |
| P0-22 | Create SMOKE-WS-002 (Workspace cleanup) | tasks/workspace/ | TODO | P0-11 |
| P0-23 | Create SMOKE-SESSION-001 (Session start) | tasks/session/ | TODO | P0-11 |
| P0-24 | Create SMOKE-PERM-001 (Permission prompt) | tasks/permissions/ | TODO | P0-11 |
| P0-25 | Create SMOKE-API-001 (API health check) | tasks/api/ | TODO | P0-11 |
| P0-26 | Create 3-5 additional high-value smoke tasks per category | tasks/*/ | TODO | P0-11 |

---

## P1 High Priority

| # | Task | Module | Status | Dependencies |
|---|------|--------|--------|--------------|
| P1-1 | Implement TaskLoader trait (load_from_dir, load_single) | loaders/task_loader.rs | TODO | P0-11, P0-12 |
| P1-2 | Update module exports (mod.rs files) | types/mod.rs, loaders/mod.rs | TODO | P0-5 through P0-10, P0-12, P1-1 |
| P1-3 | Implement task_schema_tests | tests/task_schema_tests.rs | TODO | P0-12, P0-19 through P0-26 |
| P1-4 | Implement fixture_loader_tests | tests/fixture_loader_tests.rs | TODO | P0-13, P0-17, P0-18 |
| P1-5 | Implement smoke_task_loading_tests | tests/smoke_task_loading_tests.rs | TODO | P1-1, P0-19 through P0-26 |
| P1-6 | Update types/mod.rs exports | types/mod.rs | TODO | All P0 type tasks |
| P1-7 | Verify `cargo build --package core` passes | CI | TODO | All P0 and P1 items |
| P1-8 | Verify `cargo test` passes | CI | TODO | P1-3, P1-4, P1-5 |

---

## P2 Next Iteration

| # | Task | Module | Status | Dependencies |
|---|------|--------|--------|--------------|
| P2-1 | Implement DifferentialRunner | runners/differential_runner.rs | TODO | P1 complete |
| P2-2 | Implement Normalizer trait with actual logic | normalizers/ | TODO | P1 complete |
| P2-3 | Implement Comparator trait with actual logic | comparators/ | TODO | P1 complete |
| P2-4 | Implement Verifier trait with actual logic | verifiers/ | TODO | P1 complete |
| P2-5 | Connect CLI run command to TaskLoader/Runner | cli/ | TODO | P1 complete |
| P2-6 | Create workspace-lifecycle.md documentation | docs/ | TODO | P0-13, P0-16 |
| P2-7 | Implement LegacyRunner::execute() with actual binary invocation | runners/ | TODO | P1 complete |
| P2-8 | Implement RustRunner::execute() with actual binary invocation | runners/ | TODO | P1 complete |

---

## Task Details

### P0-5: ✅ Done

**File:** `harness/crates/core/src/types/assertion.rs`

```rust
pub enum AssertionType {
    ExitCodeEquals(u32),
    StdoutContains(String),
    StderrContains(String),
    FileChanged(String),
    NoExtraFilesChanged,
    PermissionPromptSeen(String),
}
```

### P0-6: ✅ Done

**File:** `harness/crates/core/src/types/execution_policy.rs`

```rust
pub enum ExecutionPolicy {
    ManualCheck,
    Blocked,
    Skip,
}
```

### P0-7: ✅ Done

**File:** `harness/crates/core/src/types/on_missing_dependency.rs`

```rust
pub enum OnMissingDependency {
    Fail,
    Skip,
    Warn,
    Blocked,
}
```

### P0-11: ✅ Done

**File:** `harness/crates/core/src/types/task.rs`

Extended Task struct with all required fields:
- preconditions: Vec<String>
- entry_mode: EntryMode
- agent_mode: AgentMode
- provider_mode: ProviderMode
- input: TaskInput
- expected_assertions: Vec<AssertionType>
- allowed_variance: Option<AllowedVariance>
- severity: Severity
- tags: Vec<String>
- execution_policy: ExecutionPolicy
- timeout_seconds: u64
- on_missing_dependency: OnMissingDependency
- expected_outputs: Option<TaskOutputs>

### P0-12: ✅ Done

**File:** `harness/crates/core/src/loaders/task_validator.rs`

```rust
pub trait TaskSchemaValidator: Send + Sync {
    fn validate(&self, task: &Task) -> Result<()>;
    fn validate_file(&self, path: &Path) -> Result<()>;
}
```

### P0-13: ✅ Done

**File:** `harness/crates/core/src/loaders/fixture_loader.rs`

```rust
pub trait FixtureLoader: Send + Sync {
    fn load(&self, name: &str) -> Result<FixtureProject>;
    fn init_workspace(&self, fixture: &FixtureProject) -> Result<Workspace>;
    fn cleanup_workspace(&self, workspace: &Workspace) -> Result<()>;
}
```

### P0-14: WorkspacePolicy

**File:** `harness/crates/core/src/types/fixture.rs`

```rust
pub struct WorkspacePolicy {
    pub allow_dirty_git: bool,
    pub allow_network: bool,
    pub preserve_on_failure: bool,
}
```

### P0-15: ResetStrategy

**File:** `harness/crates/core/src/types/fixture.rs`

```rust
pub enum ResetStrategy {
    None,
    CleanClone,
    RestoreFiles,
}
```

---

## Smoke Task YAML Templates

### CLI Task Template

```yaml
id: <CATEGORY>-CLI-<NNN>
title: <Human readable title>
category: smoke
fixture_project: fixtures/projects/cli-basic
preconditions:
  - opencode binary exists
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: opencode
  args: ["<args>"]
  cwd: "/project"
expected_assertions:
  - type: exit_code_equals
    value: "0"
  - type: stdout_contains
    value: "<expected_output>"
allowed_variance:
  exit_code: [0]
severity: high
tags:
  - smoke
  - cli
execution_policy: manual_check
timeout_seconds: 30
on_missing_dependency: fail
```

---

## Verification Checklist

- [ ] `cargo build --package core` succeeds
- [ ] `cargo test task_schema_tests` passes
- [ ] `cargo test fixture_loader_tests` passes
- [ ] `cargo test smoke_task_loading_tests` passes
- [ ] All task YAML files parse without error
- [ ] All fixture harness.toml files parse without error

---

## Out of Scope (opencode-rs product functionality)

These are NOT tasks - report as mismatch/contract gap only:

- opencode-rs CLI implementation details
- opencode-rs workspace management implementation
- opencode-rs permission model implementation
- opencode-rs session state management
- opencode-rs API server implementation
- Any bug fixes to opencode-rs itself
