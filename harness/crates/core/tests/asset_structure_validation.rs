use std::path::PathBuf;

fn get_workspace_root() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    PathBuf::from(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn get_harness_golden_path() -> PathBuf {
    get_workspace_root().join("harness").join("golden")
}

fn get_harness_regression_path() -> PathBuf {
    get_workspace_root().join("harness").join("regression")
}

#[test]
fn test_harness_golden_directory_exists() {
    let golden_path = get_harness_golden_path();
    assert!(
        golden_path.exists(),
        "harness/golden/ directory should exist at {:?}",
        golden_path
    );
    assert!(
        golden_path.is_dir(),
        "harness/golden/ should be a directory"
    );
}

#[test]
fn test_harness_regression_directory_exists() {
    let regression_path = get_harness_regression_path();
    assert!(
        regression_path.exists(),
        "harness/regression/ directory should exist at {:?}",
        regression_path
    );
    assert!(
        regression_path.is_dir(),
        "harness/regression/ should be a directory"
    );
}

#[test]
fn test_golden_directory_has_proper_structure() {
    let golden_path = get_harness_golden_path();
    assert!(
        golden_path.join("baselines").is_dir(),
        "harness/golden/baselines/ subdirectory should exist"
    );
    assert!(
        golden_path.join("normalized").is_dir(),
        "harness/golden/normalized/ subdirectory should exist"
    );
    assert!(
        golden_path.join("raw").is_dir(),
        "harness/golden/raw/ subdirectory should exist"
    );
}

#[test]
fn test_regression_directory_has_proper_structure() {
    let regression_path = get_harness_regression_path();
    assert!(
        regression_path.join("bugs").is_dir(),
        "harness/regression/bugs/ subdirectory should exist"
    );
    assert!(
        regression_path.join("incidents").is_dir(),
        "harness/regression/incidents/ subdirectory should exist"
    );
    assert!(
        regression_path.join("issues").is_dir(),
        "harness/regression/issues/ subdirectory should exist"
    );
}
