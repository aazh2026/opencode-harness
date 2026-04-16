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

fn get_reports_schema_path() -> PathBuf {
    get_workspace_root()
        .join("harness")
        .join("reports")
        .join("schema.json")
}

#[test]
fn test_reports_schema_file_exists() {
    let schema_path = get_reports_schema_path();
    assert!(
        schema_path.exists(),
        "harness/reports/schema.json should exist at {:?}",
        schema_path
    );
}

#[test]
fn test_reports_schema_is_valid_json() {
    let schema_path = get_reports_schema_path();
    let content = std::fs::read_to_string(&schema_path).expect("schema.json should be readable");
    let _parsed: serde_json::Value =
        serde_json::from_str(&content).expect("schema.json should be valid JSON");
}

#[test]
fn test_report_schema_defines_report_struct() {
    let schema_path = get_reports_schema_path();
    let content = std::fs::read_to_string(&schema_path).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&content).unwrap();

    let report_def = schema
        .pointer("/definitions/Report")
        .expect("Report definition should exist in schema");
    let props = report_def
        .pointer("/properties")
        .expect("Report should have properties");

    assert!(
        props.pointer("/timestamp").is_some(),
        "Report should have timestamp field"
    );
    assert!(
        props.pointer("/suite").is_some(),
        "Report should have suite field"
    );
    assert!(
        props.pointer("/total").is_some(),
        "Report should have total field"
    );
    assert!(
        props.pointer("/passed").is_some(),
        "Report should have passed field"
    );
    assert!(
        props.pointer("/failed").is_some(),
        "Report should have failed field"
    );
    assert!(
        props.pointer("/skipped").is_some(),
        "Report should have skipped field"
    );
    assert!(
        props.pointer("/mismatches").is_some(),
        "Report should have mismatches field"
    );
}

#[test]
fn test_report_schema_defines_testcase_struct() {
    let schema_path = get_reports_schema_path();
    let content = std::fs::read_to_string(&schema_path).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&content).unwrap();

    let testcase_def = schema
        .pointer("/definitions/TestCase")
        .expect("TestCase definition should exist in schema");
    let props = testcase_def
        .pointer("/properties")
        .expect("TestCase should have properties");

    assert!(
        props.pointer("/id").is_some(),
        "TestCase should have id field"
    );
    assert!(
        props.pointer("/status").is_some(),
        "TestCase should have status field"
    );
    assert!(
        props.pointer("/duration").is_some(),
        "TestCase should have duration field"
    );
    assert!(
        props.pointer("/failure_classification").is_some(),
        "TestCase should have failure_classification field"
    );
    assert!(
        props.pointer("/error_message").is_some(),
        "TestCase should have error_message field"
    );
}
