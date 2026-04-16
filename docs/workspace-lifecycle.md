# Workspace Lifecycle Management

## Overview

The workspace lifecycle management system in opencode-harness handles the creation, execution, and cleanup of isolated runtime workspaces derived from fixture projects. This system ensures test isolation, reproducibility, and proper resource management.

## Core Concepts

### Fixture Project vs Runtime Workspace

| Aspect | Fixture Project | Runtime Workspace |
|--------|----------------|-------------------|
| Location | `harness/fixtures/projects/<name>` | Temporary directory or `workspace/` |
| Purpose | Template/source of truth | Runtime execution environment |
| Mutability | Never modified at runtime | Modified during test execution |
| Git State | May be dirty (controlled by policy) | Isolated copy |

```
┌─────────────────────────────────────────────────────────────┐
│                    Fixture Project                          │
│  Path: harness/fixtures/projects/<name>                     │
│  - Template/master copy                                     │
│  - Never modified by runtime                                │
│  - Git state controlled by workspace_policy.allow_dirty_git │
└─────────────────────────────────────────────────────────────┘
                           │
                    [FixtureLoader.init_workspace()]
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                  Runtime Workspace                           │
│  Path: /tmp/workspace-<uuid>/<name> or workspace/<name>     │
│  - Isolated copy of fixture                                 │
│  - Contains runtime artifacts and modifications             │
│  - Cleanup controlled by workspace_policy.preserve_on_failure │
└─────────────────────────────────────────────────────────────┘
```

## Data Structures

### Workspace

```rust
pub struct Workspace {
    pub id: String,           // Unique identifier (e.g., "ws-abc123")
    pub path: PathBuf,         // Absolute path to workspace directory
    pub fixture_name: String,   // Source fixture project name
    pub created_at: String,    // RFC3339 timestamp
}
```

### WorkspacePolicy

```rust
pub struct WorkspacePolicy {
    pub allow_dirty_git: bool,       // Allow fixture to have uncommitted changes
    pub allow_network: bool,          // Allow network access during execution
    pub preserve_on_failure: bool,   // Keep workspace after failed tests
}
```

### ResetStrategy

```rust
pub enum ResetStrategy {
    None,         // No reset - workspace persists as-is
    CleanClone,   // Re-clone entire fixture from source
    RestoreFiles, // Restore files to initial fixture state
}
```

## Lifecycle Phases

### 1. Create Phase

The `FixtureLoader` creates an isolated workspace directory:

```rust
fn create_workspace_directory(&self, fixture_name: &str) -> Result<Workspace> {
    let temp_dir = tempfile::Builder::new()
        .prefix("workspace-")
        .tempdir()?;
    let workspace_path = temp_dir.path().join(fixture_name);
    std::fs::create_dir_all(&workspace_path)?;
    let workspace_id = format!("ws-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());
    Workspace::new(workspace_id, workspace_path, fixture_name.to_string())
}
```

### 2. Copy Phase

Files are copied from the fixture to the workspace:

```rust
fn copy_fixture_files(&self, workspace: &Workspace, fixture: &FixtureProject) -> Result<()> {
    for file in &fixture.files {
        let file_path = workspace.path.join(&file.path);
        // Create parent directories and write file content
        std::fs::create_dir_all(parent)?;
        std::fs::write(&file_path, &file.content)?;
        // Set executable bit if needed (Unix only)
    }
    Ok(())
}
```

### 3. Setup Phase

The optional setup script is executed if present:

```rust
if let Some(setup_script) = &fixture.setup_script {
    let script_path = self.fixtures_base_path.join(&fixture.name).join(setup_script);
    if script_path.exists() {
        std::process::Command::new("bash")
            .arg(&script_path)
            .current_dir(&workspace.path)
            .output()?;
    }
}
```

### 4. Execute Phase

The task runs within the isolated workspace environment.

### 5. Teardown Phase

The optional teardown script is executed if present.

### 6. Cleanup Phase

Based on `workspace_policy.preserve_on_failure`:

| Policy | On Success | On Failure |
|--------|------------|-------------|
| `preserve_on_failure: false` | Delete workspace | Delete workspace |
| `preserve_on_failure: true` | Delete workspace | Preserve workspace |

```rust
fn cleanup_workspace(&self, workspace: &Workspace) -> Result<()> {
    if workspace.path.exists() {
        std::fs::remove_dir_all(&workspace.path)?;
    }
    Ok(())
}
```

## Reset Strategies

### None

No reset is performed. The workspace persists in its current state. Suitable for debugging mode.

### CleanClone

The entire fixture is re-cloned from source. Appropriate for tests requiring a pristine environment.

### RestoreFiles

Files are restored to their initial fixture state while preserving git history. Suitable for tests that need to maintain git history.

## FixtureLoader Trait

```rust
pub trait FixtureLoader: Send + Sync {
    fn load(&self, name: &str) -> Result<FixtureProject>;
    fn init_workspace(&self, fixture: &FixtureProject) -> Result<Workspace>;
    fn cleanup_workspace(&self, workspace: &Workspace) -> Result<()>;
}
```

### Methods

| Method | Description |
|--------|-------------|
| `load(name)` | Load fixture project configuration from `harness.toml` |
| `init_workspace(fixture)` | Create and initialize a runtime workspace from fixture |
| `cleanup_workspace(workspace)` | Remove workspace directory and release resources |

### Usage Example

```rust
let loader = DefaultFixtureLoader::new(PathBuf::from("harness/fixtures/projects"));

// Load fixture
let fixture = loader.load("cli-basic")?;

// Initialize workspace
let workspace = loader.init_workspace(&fixture)?;

// Execute tests in workspace...

// Cleanup
loader.cleanup_workspace(&workspace)?;
```

## Workspace Isolation Guarantees

1. **File System Isolation**: Each workspace has its own directory tree
2. **No Cross-Workspace Contamination**: Runtime modifications stay within workspace
3. **Clean State**: Workspace starts as exact copy of fixture
4. **Deterministic Cleanup**: Resources are always released (except on preserve_on_failure)

## Error Handling

| Error Scenario | Behavior |
|----------------|----------|
| Fixture not found | Return `ErrorType::Config` |
| Setup script fails | Return `ErrorType::Config` with stderr |
| Workspace creation fails | Return `ErrorType::Config` |
| File copy fails | Return `ErrorType::Config` |
| Cleanup fails | Return `ErrorType::Config` |
