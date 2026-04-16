use opencode_core::loaders::fixture_loader::DefaultFixtureLoader;
use opencode_core::loaders::FixtureLoader;
use opencode_core::types::fixture::ResetStrategy;
use opencode_core::types::workspace::Workspace;
use std::path::PathBuf;

fn get_fixtures_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let core_path = PathBuf::from(manifest_dir);
    let harness_path = core_path
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    harness_path.join("harness/fixtures/projects")
}

#[test]
fn test_verify_fixture_loader_can_load_cli_basic_fixture() {
    let fixtures_path = get_fixtures_path();
    let loader = DefaultFixtureLoader::new(fixtures_path);

    let result = loader.load("cli-basic");
    assert!(
        result.is_ok(),
        "Failed to load cli-basic fixture: {:?}",
        result.err()
    );

    let fixture = result.unwrap();
    assert_eq!(fixture.name, "cli-basic");
    assert!(!fixture.description.is_empty());
    assert_eq!(fixture.reset_strategy, ResetStrategy::CleanClone);
    assert!(!fixture.workspace_policy.allow_network);
    assert!(!fixture.workspace_policy.allow_dirty_git);
    assert!(!fixture.workspace_policy.preserve_on_failure);
}

#[test]
fn test_verify_fixture_loader_can_load_api_project_fixture() {
    let fixtures_path = get_fixtures_path();
    let loader = DefaultFixtureLoader::new(fixtures_path);

    let result = loader.load("api-project");
    assert!(
        result.is_ok(),
        "Failed to load api-project fixture: {:?}",
        result.err()
    );

    let fixture = result.unwrap();
    assert_eq!(fixture.name, "api-project");
    assert!(!fixture.description.is_empty());
    assert_eq!(fixture.reset_strategy, ResetStrategy::RestoreFiles);
    assert!(fixture.workspace_policy.allow_network);
    assert!(fixture.workspace_policy.preserve_on_failure);
}

#[test]
fn test_verify_init_workspace_creates_correct_workspace_structure() {
    let fixtures_path = get_fixtures_path();
    let loader = DefaultFixtureLoader::new(fixtures_path);
    let fixture = loader.load("cli-basic").unwrap();

    let workspace_result = loader.init_workspace(&fixture);
    assert!(
        workspace_result.is_ok(),
        "Failed to init workspace: {:?}",
        workspace_result.err()
    );

    let workspace = workspace_result.unwrap();
    assert!(
        !workspace.id.is_empty(),
        "Workspace should have a non-empty id"
    );
    assert!(
        workspace.path.exists(),
        "Workspace path should exist after init_workspace"
    );
    assert_eq!(
        workspace.fixture_name, "cli-basic",
        "Workspace fixture_name should match the fixture"
    );

    assert!(
        workspace.path.is_dir(),
        "Workspace path should be a directory"
    );

    loader.cleanup_workspace(&workspace).unwrap();
}

#[test]
fn test_verify_cleanup_workspace_removes_workspace_correctly() {
    let fixtures_path = get_fixtures_path();
    let loader = DefaultFixtureLoader::new(fixtures_path);
    let fixture = loader.load("cli-basic").unwrap();

    let workspace = loader.init_workspace(&fixture).unwrap();
    let workspace_path = workspace.path.clone();

    assert!(
        workspace_path.exists(),
        "Workspace path should exist before cleanup"
    );

    let cleanup_result = loader.cleanup_workspace(&workspace);
    assert!(
        cleanup_result.is_ok(),
        "cleanup_workspace should succeed: {:?}",
        cleanup_result.err()
    );

    assert!(
        !workspace_path.exists(),
        "Workspace path should not exist after cleanup"
    );
}

#[test]
fn test_fixture_loading_with_missing_files_returns_error() {
    let temp_dir = tempfile::Builder::new()
        .prefix("empty-fixtures")
        .tempdir()
        .unwrap();
    let loader = DefaultFixtureLoader::new(temp_dir.path().to_path_buf());

    let result = loader.load("non-existent-fixture");
    assert!(
        result.is_err(),
        "Loading non-existent fixture should return error"
    );

    let error_result = result.unwrap_err();
    let error_msg = format!("{}", error_result);
    assert!(
        error_msg.contains("non-existent-fixture") || error_msg.contains("not found"),
        "Error message should mention the fixture name or 'not found'"
    );
}

#[test]
fn test_init_workspace_with_api_project_fixture() {
    let fixtures_path = get_fixtures_path();
    let loader = DefaultFixtureLoader::new(fixtures_path);
    let fixture = loader.load("api-project").unwrap();

    let workspace_result = loader.init_workspace(&fixture);
    assert!(
        workspace_result.is_ok(),
        "Failed to init workspace for api-project: {:?}",
        workspace_result.err()
    );

    let workspace = workspace_result.unwrap();
    assert_eq!(workspace.fixture_name, "api-project");
    assert!(workspace.path.exists());

    loader.cleanup_workspace(&workspace).unwrap();
}

#[test]
fn test_cleanup_workspace_with_api_project_fixture() {
    let fixtures_path = get_fixtures_path();
    let loader = DefaultFixtureLoader::new(fixtures_path);
    let fixture = loader.load("api-project").unwrap();

    let workspace = loader.init_workspace(&fixture).unwrap();
    let workspace_path = workspace.path.clone();

    assert!(workspace_path.exists());

    loader.cleanup_workspace(&workspace).unwrap();

    assert!(!workspace_path.exists());
}

#[test]
fn test_workspace_id_format() {
    let fixtures_path = get_fixtures_path();
    let loader = DefaultFixtureLoader::new(fixtures_path);
    let fixture = loader.load("cli-basic").unwrap();

    let workspace = loader.init_workspace(&fixture).unwrap();

    assert!(
        workspace.id.starts_with("ws-"),
        "Workspace id should start with 'ws-' prefix"
    );

    loader.cleanup_workspace(&workspace).unwrap();
}

#[test]
fn test_fixture_loader_trait_impl_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    let fixtures_path = get_fixtures_path();
    let _loader = DefaultFixtureLoader::new(fixtures_path);
    assert_send_sync::<DefaultFixtureLoader>();
}

#[test]
fn test_cleanup_non_existent_workspace_succeeds() {
    let temp_dir = tempfile::Builder::new()
        .prefix("non-existent-ws")
        .tempdir()
        .unwrap();
    let non_existent_path = temp_dir.path().join("non-existent-workspace");

    let fake_workspace = Workspace::new(
        "ws-test".to_string(),
        non_existent_path.clone(),
        "test-fixture".to_string(),
    );

    let fixtures_path = get_fixtures_path();
    let loader = DefaultFixtureLoader::new(fixtures_path);

    let result = loader.cleanup_workspace(&fake_workspace);
    assert!(
        result.is_ok(),
        "cleanup_workspace should succeed even for non-existent paths"
    );
}

#[test]
fn test_multiple_workspaces_can_coexist() {
    let fixtures_path = get_fixtures_path();
    let loader = DefaultFixtureLoader::new(fixtures_path);
    let fixture = loader.load("cli-basic").unwrap();

    let workspace1 = loader.init_workspace(&fixture).unwrap();
    let workspace2 = loader.init_workspace(&fixture).unwrap();

    assert!(
        workspace1.path.exists() && workspace2.path.exists(),
        "Both workspaces should exist simultaneously"
    );
    assert_ne!(
        workspace1.path, workspace2.path,
        "Workspaces should have different paths"
    );
    assert_ne!(
        workspace1.id, workspace2.id,
        "Workspaces should have different ids"
    );

    loader.cleanup_workspace(&workspace1).unwrap();
    assert!(
        workspace2.path.exists(),
        "Workspace2 should still exist after cleaning up workspace1"
    );

    loader.cleanup_workspace(&workspace2).unwrap();
}
