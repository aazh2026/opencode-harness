use std::process::{Command, Stdio};
use tempfile::TempDir;

fn create_test_task_yaml(
    task_id: &str,
    command: &str,
    args: &[&str],
    expected_output: &str,
) -> String {
    format!(
        r#"
id: {}
title: Test Task
category: smoke
fixture_project: fixtures/projects/cli-basic
description: Test task for CLI integration
expected_outcome: Task executes successfully
preconditions: []
entry_mode: CLI
agent_mode: OneShot
provider_mode: Both
input:
  command: "{}"
  args: {:?}
  cwd: "/tmp"
expected_assertions:
  - type: stdout_contains
    value: "{}"
severity: High
tags: []
execution_policy: ManualCheck
timeout_seconds: 60
on_missing_dependency: Fail
"#,
        task_id, command, args, expected_output
    )
}

#[test]
fn test_cli_run_executes_task_loader_for_single_task() {
    let temp_dir = TempDir::new().unwrap();
    let task_yaml = temp_dir.path().join("test_task.yaml");

    let yaml_content = create_test_task_yaml("TEST-LOAD-001", "echo", &["hello"], "hello");
    std::fs::write(&task_yaml, yaml_content).unwrap();

    let binary_path =
        std::env::var("CARGO_BIN_OPENCODE_HARNESS").unwrap_or_else(|_| "cargo".to_string());

    let mut cmd = if binary_path == "cargo" {
        let mut c = Command::new("cargo");
        c.args(["run", "--bin", "opencode-harness", "--", "run"]);
        c
    } else {
        let mut c = Command::new(&binary_path);
        c.arg("run");
        c
    };

    cmd.arg("--task")
        .arg(task_yaml.to_str().unwrap())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = cmd.output().expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stdout.contains("TEST-LOAD-001") || stdout.contains("Task"),
        "Output should contain task info. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_cli_run_executes_task_loader_for_directory() {
    let temp_dir = TempDir::new().unwrap();

    let yaml1 = create_test_task_yaml("TEST-DIR-001", "echo", &["dir1"], "dir1");
    let yaml2 = create_test_task_yaml("TEST-DIR-002", "echo", &["dir2"], "dir2");

    std::fs::write(temp_dir.path().join("task1.yaml"), yaml1).unwrap();
    std::fs::write(temp_dir.path().join("task2.yaml"), yaml2).unwrap();

    let binary_path =
        std::env::var("CARGO_BIN_OPENCODE_HARNESS").unwrap_or_else(|_| "cargo".to_string());

    let mut cmd = if binary_path == "cargo" {
        let mut c = Command::new("cargo");
        c.args(["run", "--bin", "opencode-harness", "--", "run"]);
        c
    } else {
        let mut c = Command::new(&binary_path);
        c.arg("run");
        c
    };

    cmd.arg("--task")
        .arg(temp_dir.path().to_str().unwrap())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = cmd.output().expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stdout.contains("TEST-DIR-001") && stdout.contains("TEST-DIR-002"),
        "Output should contain both task IDs. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_cli_run_command_uses_runner_to_execute() {
    let temp_dir = TempDir::new().unwrap();
    let task_yaml = temp_dir.path().join("runner_test.yaml");

    let yaml_content = create_test_task_yaml("TEST-RUN-001", "echo", &["runner"], "runner");
    std::fs::write(&task_yaml, yaml_content).unwrap();

    let binary_path =
        std::env::var("CARGO_BIN_OPENCODE_HARNESS").unwrap_or_else(|_| "cargo".to_string());

    let mut cmd = if binary_path == "cargo" {
        let mut c = Command::new("cargo");
        c.args(["run", "--bin", "opencode-harness", "--", "run"]);
        c
    } else {
        let mut c = Command::new(&binary_path);
        c.arg("run");
        c
    };

    cmd.arg("--task")
        .arg(task_yaml.to_str().unwrap())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = cmd.output().expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success() || stdout.contains("exit_code"),
        "Command should execute and report exit_code. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_cli_run_full_task_execution_from_cli_to_output() {
    let temp_dir = TempDir::new().unwrap();
    let task_yaml = temp_dir.path().join("e2e_test.yaml");

    let yaml_content =
        create_test_task_yaml("E2E-TEST-001", "printf", &["hello world"], "hello world");
    std::fs::write(&task_yaml, yaml_content).unwrap();

    let binary_path =
        std::env::var("CARGO_BIN_OPENCODE_HARNESS").unwrap_or_else(|_| "cargo".to_string());

    let mut cmd = if binary_path == "cargo" {
        let mut c = Command::new("cargo");
        c.args(["run", "--bin", "opencode-harness", "--", "run"]);
        c
    } else {
        let mut c = Command::new(&binary_path);
        c.arg("run");
        c
    };

    cmd.arg("--task")
        .arg(task_yaml.to_str().unwrap())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = cmd.output().expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stdout.contains("E2E-TEST-001") && stdout.contains("exit_code"),
        "Full execution should produce task ID and exit_code. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_cli_run_with_nonexistent_task_file() {
    let binary_path =
        std::env::var("CARGO_BIN_OPENCODE_HARNESS").unwrap_or_else(|_| "cargo".to_string());

    let mut cmd = if binary_path == "cargo" {
        let mut c = Command::new("cargo");
        c.args(["run", "--bin", "opencode-harness", "--", "run"]);
        c
    } else {
        let mut c = Command::new(&binary_path);
        c.arg("run");
        c
    };

    cmd.arg("--task")
        .arg("/nonexistent/path/task.yaml")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = cmd.output().expect("Failed to execute command");

    assert!(
        !output.status.success() || String::from_utf8_lossy(&output.stderr).contains("Error"),
        "Should fail with error for nonexistent file"
    );
}

#[test]
fn test_cli_run_default_to_current_directory() {
    let binary_path =
        std::env::var("CARGO_BIN_OPENCODE_HARNESS").unwrap_or_else(|_| "cargo".to_string());

    let mut cmd = if binary_path == "cargo" {
        let mut c = Command::new("cargo");
        c.args(["run", "--bin", "opencode-harness", "--", "run"]);
        c
    } else {
        let mut c = Command::new(&binary_path);
        c.arg("run");
        c
    };

    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let output = cmd.output().expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stdout.contains("tasks") || stderr.is_empty() || output.status.success(),
        "Should handle default directory gracefully. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}
