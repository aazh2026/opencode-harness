use opencode_core::loaders::FixtureLoader;
use opencode_core::types::fixture::{FixtureFile, FixtureProject, ResetStrategy, WorkspacePolicy};
use opencode_core::types::workspace::Workspace;
use std::path::PathBuf;
use tempfile::TempDir;

struct TestFixtureLoader {
    fixtures_path: PathBuf,
}

impl TestFixtureLoader {
    fn new(fixtures_path: PathBuf) -> Self {
        Self { fixtures_path }
    }
}

impl FixtureLoader for TestFixtureLoader {
    fn load(&self, name: &str) -> opencode_core::error::Result<FixtureProject> {
        let fixture_path = self.fixtures_path.join(name).join("harness.toml");
        if !fixture_path.exists() {
            return Err(opencode_core::error::ErrorType::Config(format!(
                "Fixture '{}' not found",
                name
            )));
        }

        let content = std::fs::read_to_string(&fixture_path)?;
        let toml_value: toml::Value = toml::from_str(&content).map_err(|e| {
            opencode_core::error::ErrorType::Config(format!("TOML parse error: {}", e))
        })?;

        let name_str = toml_value
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(name)
            .to_string();
        let description = toml_value
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let workspace_policy = WorkspacePolicy {
            allow_dirty_git: false,
            allow_network: false,
            preserve_on_failure: false,
        };

        let reset_strategy = match toml_value.get("reset_strategy").and_then(|v| v.as_str()) {
            Some("clean_clone") => ResetStrategy::CleanClone,
            Some("restore_files") => ResetStrategy::RestoreFiles,
            _ => ResetStrategy::None,
        };

        let files: Vec<FixtureFile> = toml_value
            .get("files")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        Some(FixtureFile {
                            path: item.get("path")?.as_str()?.to_string(),
                            content: item.get("content")?.as_str()?.to_string(),
                            executable: item.get("executable")?.as_bool().unwrap_or(false),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(FixtureProject {
            name: name_str,
            description,
            workspace_policy,
            reset_strategy,
            setup_script: None,
            teardown_script: None,
            files,
        })
    }

    fn init_workspace(&self, fixture: &FixtureProject) -> opencode_core::error::Result<Workspace> {
        let temp_dir = tempfile::Builder::new()
            .prefix("workspace-")
            .tempdir()
            .map_err(|e| {
                opencode_core::error::ErrorType::Config(format!("TempDir error: {}", e))
            })?;
        let workspace_path = temp_dir.path().join(&fixture.name);
        std::fs::create_dir_all(&workspace_path).map_err(|e| {
            opencode_core::error::ErrorType::Config(format!("Create dir error: {}", e))
        })?;

        for file in &fixture.files {
            let file_path = workspace_path.join(&file.path);
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&file_path, &file.content)?;
        }

        std::mem::forget(temp_dir);

        Ok(Workspace::new(
            format!(
                "ws-{}",
                uuid::Uuid::new_v4().to_string().split('-').next().unwrap()
            ),
            workspace_path,
            fixture.name.clone(),
        ))
    }

    fn cleanup_workspace(&self, workspace: &Workspace) -> opencode_core::error::Result<()> {
        if workspace.path.exists() {
            std::fs::remove_dir_all(&workspace.path)?;
        }
        Ok(())
    }
}

fn create_test_fixture_dir(temp_dir: &TempDir, name: &str) {
    let fixture_path = temp_dir.path().join(name);
    std::fs::create_dir_all(&fixture_path).unwrap();
    std::fs::write(
        fixture_path.join("harness.toml"),
        format!(
            r##"
name = "{}"
description = "Test fixture for {}"
reset_strategy = "none"

[workspace_policy]
allow_dirty_git = false
allow_network = false
preserve_on_failure = false

[[files]]
path = "README.md"
content = "Test Fixture"
executable = false
"##,
            name, name
        ),
    )
    .unwrap();
}

#[test]
fn test_fixture_loader_trait_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    let temp_dir = TempDir::new().unwrap();
    let loader = TestFixtureLoader::new(temp_dir.path().to_path_buf());
    assert_send_sync::<TestFixtureLoader>();
}

#[test]
fn test_load_fixture_project_correctly() {
    let temp_dir = TempDir::new().unwrap();
    create_test_fixture_dir(&temp_dir, "test-fixture");

    let loader = TestFixtureLoader::new(temp_dir.path().to_path_buf());
    let result = loader.load("test-fixture");

    assert!(result.is_ok());
    let fixture = result.unwrap();
    assert_eq!(fixture.name, "test-fixture");
    assert!(!fixture.description.is_empty());
    assert_eq!(fixture.reset_strategy, ResetStrategy::None);
    assert!(!fixture.files.is_empty());
    assert_eq!(fixture.files[0].path, "README.md");
}

#[test]
fn test_load_fixture_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let loader = TestFixtureLoader::new(temp_dir.path().to_path_buf());
    let result = loader.load("non-existent-fixture");
    assert!(result.is_err());
}

#[test]
fn test_init_workspace_correctly() {
    let temp_dir = TempDir::new().unwrap();
    create_test_fixture_dir(&temp_dir, "test-fixture");

    let loader = TestFixtureLoader::new(temp_dir.path().to_path_buf());
    let fixture = loader.load("test-fixture").unwrap();
    let workspace_result = loader.init_workspace(&fixture);

    assert!(workspace_result.is_ok());
    let workspace = workspace_result.unwrap();
    assert_eq!(workspace.fixture_name, "test-fixture");
    assert!(workspace.path.exists());
    assert!(workspace.path.join("README.md").exists());

    loader.cleanup_workspace(&workspace).unwrap();
}

#[test]
fn test_cleanup_workspace_correctly() {
    let temp_dir = TempDir::new().unwrap();
    create_test_fixture_dir(&temp_dir, "test-fixture");

    let loader = TestFixtureLoader::new(temp_dir.path().to_path_buf());
    let fixture = loader.load("test-fixture").unwrap();
    let workspace = loader.init_workspace(&fixture).unwrap();
    let workspace_path = workspace.path.clone();

    assert!(workspace_path.exists());
    loader.cleanup_workspace(&workspace).unwrap();
    assert!(!workspace_path.exists());
}

#[test]
fn test_workspace_files_copied_correctly() {
    let temp_dir = TempDir::new().unwrap();
    create_test_fixture_dir(&temp_dir, "multi-file-fixture");

    let loader = TestFixtureLoader::new(temp_dir.path().to_path_buf());
    let fixture = loader.load("multi-file-fixture").unwrap();
    let workspace = loader.init_workspace(&fixture).unwrap();

    assert!(workspace.path.join("README.md").exists());
    let readme_content = std::fs::read_to_string(workspace.path.join("README.md")).unwrap();
    assert!(readme_content.contains("Test Fixture"));

    loader.cleanup_workspace(&workspace).unwrap();
}
