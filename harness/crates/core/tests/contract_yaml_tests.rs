use opencode_core::loaders::{ContractLoader, DefaultContractLoader};
use std::path::PathBuf;

fn get_contract_path(contract_name: &str) -> PathBuf {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.join("../../contracts").join(contract_name)
}

#[test]
fn test_cli_contract_yaml_valid() {
    let loader = DefaultContractLoader::new();
    let path = get_contract_path("cli/cli_contract.yaml");
    let result = loader.load_single(&path);
    assert!(
        result.is_ok(),
        "Failed to load cli_contract.yaml: {:?}",
        result.err()
    );
    let contract = result.unwrap();
    assert_eq!(contract.contract_id, "CLI-CONTRACT-001");
    assert_eq!(contract.name, "CLI Basic Execution Contract");
}

#[test]
fn test_permission_contract_yaml_valid() {
    let loader = DefaultContractLoader::new();
    let path = get_contract_path("permissions/permission_contract.yaml");
    let result = loader.load_single(&path);
    assert!(
        result.is_ok(),
        "Failed to load permission_contract.yaml: {:?}",
        result.err()
    );
    let contract = result.unwrap();
    assert_eq!(contract.contract_id, "PERMISSION-CONTRACT-001");
    assert_eq!(contract.name, "Permission Flow Contract");
}

#[test]
fn test_workspace_contract_yaml_valid() {
    let loader = DefaultContractLoader::new();
    let path = get_contract_path("side_effects/workspace_contract.yaml");
    let result = loader.load_single(&path);
    assert!(
        result.is_ok(),
        "Failed to load workspace_contract.yaml: {:?}",
        result.err()
    );
    let contract = result.unwrap();
    assert_eq!(contract.contract_id, "WORKSPACE-SIDE-EFFECT-001");
    assert_eq!(contract.name, "Workspace Side Effect Contract");
}

#[test]
fn test_session_contract_yaml_valid() {
    let loader = DefaultContractLoader::new();
    let path = get_contract_path("state_machine/session_contract.yaml");
    let result = loader.load_single(&path);
    assert!(
        result.is_ok(),
        "Failed to load session_contract.yaml: {:?}",
        result.err()
    );
    let contract = result.unwrap();
    assert_eq!(contract.contract_id, "SESSION-STATE-MACHINE-001");
    assert_eq!(contract.name, "Session State Machine Contract");
}

#[test]
fn test_all_contracts_loadable() {
    let loader = DefaultContractLoader::new();
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let contracts_dir = manifest_dir.join("../../contracts");

    let cli_path = contracts_dir.join("cli/cli_contract.yaml");
    let perm_path = contracts_dir.join("permissions/permission_contract.yaml");
    let ws_path = contracts_dir.join("side_effects/workspace_contract.yaml");
    let sm_path = contracts_dir.join("state_machine/session_contract.yaml");

    assert!(
        loader.load_single(&cli_path).is_ok(),
        "cli_contract.yaml should be loadable"
    );
    assert!(
        loader.load_single(&perm_path).is_ok(),
        "permission_contract.yaml should be loadable"
    );
    assert!(
        loader.load_single(&ws_path).is_ok(),
        "workspace_contract.yaml should be loadable"
    );
    assert!(
        loader.load_single(&sm_path).is_ok(),
        "session_contract.yaml should be loadable"
    );
}
