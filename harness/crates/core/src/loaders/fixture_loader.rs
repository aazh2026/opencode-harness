use crate::error::{ErrorType, Result};
use crate::types::fixture::{FixtureFile, FixtureProject, ResetStrategy, WorkspacePolicy};
use crate::types::workspace::Workspace;
use std::path::Path;
use std::path::PathBuf;

pub trait FixtureLoader: Send + Sync {
    fn load(&self, name: &str) -> Result<FixtureProject>;
    fn init_workspace(&self, fixture: &FixtureProject) -> Result<Workspace>;
    fn cleanup_workspace(&self, workspace: &Workspace) -> Result<()>;
}

pub struct DefaultFixtureLoader {
    fixtures_base_path: PathBuf,
}

impl DefaultFixtureLoader {
    pub fn new(fixtures_base_path: PathBuf) -> Self {
        Self { fixtures_base_path }
    }

    fn load_toml(&self, path: &Path) -> Result<FixtureProject> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ErrorType::Config(format!("Failed to read fixture TOML: {}", e)))?;

        let toml_value: toml::Value = toml::from_str(&content)
            .map_err(|e| ErrorType::Config(format!("Failed to parse fixture TOML: {}", e)))?;

        let name = toml_value
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ErrorType::Config("Missing 'name' field in fixture".to_string()))?
            .to_string();

        let description = toml_value
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let workspace_policy = self.parse_workspace_policy(&toml_value)?;
        let reset_strategy = self.parse_reset_strategy(&toml_value)?;

        let setup_script = toml_value
            .get("setup_script")
            .and_then(|v| v.as_str())
            .map(String::from);
        let teardown_script = toml_value
            .get("teardown_script")
            .and_then(|v| v.as_str())
            .map(String::from);

        let files = self.parse_files(&toml_value)?;

        Ok(FixtureProject {
            name,
            description,
            workspace_policy,
            reset_strategy,
            setup_script,
            teardown_script,
            files,
            configs: Vec::new(),
            transcripts: Vec::new(),
        })
    }

    fn parse_workspace_policy(&self, toml_value: &toml::Value) -> Result<WorkspacePolicy> {
        let workspace_policy_table = toml_value
            .get("workspace_policy")
            .and_then(|v| v.as_table())
            .ok_or_else(|| {
                ErrorType::Config("Missing 'workspace_policy' section in fixture".to_string())
            })?;

        let allow_dirty_git = workspace_policy_table
            .get("allow_dirty_git")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let allow_network = workspace_policy_table
            .get("allow_network")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let preserve_on_failure = workspace_policy_table
            .get("preserve_on_failure")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(WorkspacePolicy {
            allow_dirty_git,
            allow_network,
            preserve_on_failure,
        })
    }

    fn parse_reset_strategy(&self, toml_value: &toml::Value) -> Result<ResetStrategy> {
        let strategy_str = toml_value
            .get("reset_strategy")
            .and_then(|v| v.as_str())
            .unwrap_or("none");

        match strategy_str {
            "clean_clone" => Ok(ResetStrategy::CleanClone),
            "restore_files" => Ok(ResetStrategy::RestoreFiles),
            _ => Ok(ResetStrategy::None),
        }
    }

    fn parse_files(&self, toml_value: &toml::Value) -> Result<Vec<FixtureFile>> {
        let mut files = Vec::new();

        if let Some(files_array) = toml_value.get("files").and_then(|v| v.as_array()) {
            for item in files_array {
                let path = item
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ErrorType::Config("Missing 'path' in file entry".to_string()))?
                    .to_string();

                let content = item
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let executable = item
                    .get("executable")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                files.push(FixtureFile {
                    path,
                    content,
                    executable,
                });
            }
        }

        Ok(files)
    }

    fn create_workspace_directory(&self, fixture_name: &str) -> Result<Workspace> {
        let temp_dir = tempfile::Builder::new()
            .prefix("workspace-")
            .tempdir()
            .map_err(|e| ErrorType::Config(format!("Failed to create temp directory: {}", e)))?;

        let workspace_path = temp_dir.path().join(fixture_name);
        std::fs::create_dir_all(&workspace_path).map_err(|e| {
            ErrorType::Config(format!("Failed to create workspace directory: {}", e))
        })?;

        let workspace_id = format!(
            "ws-{}",
            uuid::Uuid::new_v4().to_string().split('-').next().unwrap()
        );

        let workspace = Workspace::new(workspace_id, workspace_path, fixture_name.to_string());

        std::mem::forget(temp_dir);

        Ok(workspace)
    }

    fn copy_fixture_files(&self, workspace: &Workspace, fixture: &FixtureProject) -> Result<()> {
        for file in &fixture.files {
            let file_path = workspace.path.join(&file.path);
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| ErrorType::Config(format!("Failed to create directory: {}", e)))?;
            }
            std::fs::write(&file_path, &file.content)
                .map_err(|e| ErrorType::Config(format!("Failed to write file: {}", e)))?;

            if file.executable {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = std::fs::metadata(&file_path)
                        .map_err(|e| {
                            ErrorType::Config(format!("Failed to get permissions: {}", e))
                        })?
                        .permissions();
                    perms.set_mode(0o755);
                    std::fs::set_permissions(&file_path, perms).map_err(|e| {
                        ErrorType::Config(format!("Failed to set permissions: {}", e))
                    })?;
                }
            }
        }
        Ok(())
    }
}

impl FixtureLoader for DefaultFixtureLoader {
    fn load(&self, name: &str) -> Result<FixtureProject> {
        let fixture_path = self.fixtures_base_path.join(name).join("harness.toml");
        if !fixture_path.exists() {
            return Err(ErrorType::Config(format!(
                "Fixture '{}' not found at {}",
                name,
                fixture_path.display()
            )));
        }
        self.load_toml(&fixture_path)
    }

    fn init_workspace(&self, fixture: &FixtureProject) -> Result<Workspace> {
        let mut workspace = self.create_workspace_directory(&fixture.name)?;
        workspace.fixture_name = fixture.name.clone();

        self.copy_fixture_files(&workspace, fixture)?;

        if let Some(setup_script) = &fixture.setup_script {
            let script_path = self
                .fixtures_base_path
                .join(&fixture.name)
                .join(setup_script);
            if script_path.exists() {
                let output = std::process::Command::new("bash")
                    .arg(&script_path)
                    .current_dir(&workspace.path)
                    .output()
                    .map_err(|e| ErrorType::Config(format!("Failed to run setup script: {}", e)))?;

                if !output.status.success() {
                    return Err(ErrorType::Config(format!(
                        "Setup script failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    )));
                }
            }
        }

        Ok(workspace)
    }

    fn cleanup_workspace(&self, workspace: &Workspace) -> Result<()> {
        if workspace.path.exists() {
            std::fs::remove_dir_all(&workspace.path)
                .map_err(|e| ErrorType::Config(format!("Failed to cleanup workspace: {}", e)))?;
        }
        Ok(())
    }
}
