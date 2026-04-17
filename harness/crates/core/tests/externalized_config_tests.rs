use opencode_core::config::{AppConfig, AppConfigError};
use opencode_core::reporting::gate::{CIGate, GateConfig, GateLevel};
use opencode_core::reporting::report::{ParityReport, TaskResult};
use opencode_core::types::parity_verdict::{DiffCategory, ParityVerdict};
use std::fs;
use tempfile::TempDir;

fn create_test_config_file(content: &str, name: &str) -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join(name);
    fs::write(&config_path, content).unwrap();
    temp_dir
}

fn create_pass_report(pass_count: u32, fail_count: u32) -> ParityReport {
    let mut report = ParityReport::new("TestRunner");
    for i in 0..pass_count {
        report.add_task(TaskResult::new(
            format!("PASS-{}", i),
            ParityVerdict::Pass,
            100,
        ));
    }
    for i in 0..fail_count {
        report.add_task(TaskResult::new(
            format!("FAIL-{}", i),
            ParityVerdict::Fail {
                category: DiffCategory::OutputText,
                details: "Mismatch".to_string(),
            },
            50,
        ));
    }
    report.compute_summary();
    report
}

mod pass_rate_config_tests {
    use super::*;

    #[test]
    fn test_pass_rate_threshold_read_from_config_file() {
        let temp_dir = create_test_config_file(
            r#"
gate_thresholds:
  pr_pass_rate: 0.85
  nightly_pass_rate: 0.70
  release_pass_rate: 0.95
  pr_max_warnings: 3
  nightly_max_warnings: 8
  release_max_warnings: 1
  error_rate_threshold: 0.15
timeout:
  default_timeout_seconds: 500
  suite_timeout_seconds: 120
  max_timeout_seconds: 1800
"#,
            "config.yaml",
        );

        let config =
            AppConfig::load_from_file(&temp_dir.path().join("config.yaml")).unwrap();

        assert!(
            (config.gate_thresholds.pr_pass_rate - 0.85).abs() < 0.01,
            "PR pass rate should be 0.85 from config"
        );
        assert!(
            (config.gate_thresholds.nightly_pass_rate - 0.70).abs() < 0.01,
            "Nightly pass rate should be 0.70 from config"
        );
        assert!(
            (config.gate_thresholds.release_pass_rate - 0.95).abs() < 0.01,
            "Release pass rate should be 0.95 from config"
        );
    }

    #[test]
    fn test_gate_uses_config_pass_rate_threshold() {
        let temp_dir = create_test_config_file(
            r#"
gate_thresholds:
  pr_pass_rate: 0.85
  nightly_pass_rate: 0.70
  release_pass_rate: 0.95
  pr_max_warnings: 3
  nightly_max_warnings: 8
  release_max_warnings: 1
  error_rate_threshold: 0.15
timeout:
  default_timeout_seconds: 500
  suite_timeout_seconds: 120
  max_timeout_seconds: 1800
"#,
            "config.yaml",
        );

        let app_config =
            AppConfig::load_from_file(&temp_dir.path().join("config.yaml")).unwrap();
        let gate_config = GateConfig::from_app_config(GateLevel::PR, &app_config);

        assert!(
            (gate_config.pass_rate_threshold - 0.85).abs() < 0.01,
            "GateConfig should use PR pass rate 0.85 from config"
        );

        let gate_config = GateConfig::from_app_config(GateLevel::Nightly, &app_config);
        assert!(
            (gate_config.pass_rate_threshold - 0.70).abs() < 0.01,
            "GateConfig should use Nightly pass rate 0.70 from config"
        );

        let gate_config = GateConfig::from_app_config(GateLevel::Release, &app_config);
        assert!(
            (gate_config.pass_rate_threshold - 0.95).abs() < 0.01,
            "GateConfig should use Release pass rate 0.95 from config"
        );
    }

    #[test]
    fn test_gate_eval_passes_with_custom_threshold_from_config() {
        let temp_dir = create_test_config_file(
            r#"
gate_thresholds:
  pr_pass_rate: 0.80
  nightly_pass_rate: 0.70
  release_pass_rate: 0.95
  pr_max_warnings: 3
  nightly_max_warnings: 8
  release_max_warnings: 1
  error_rate_threshold: 0.15
timeout:
  default_timeout_seconds: 500
  suite_timeout_seconds: 120
  max_timeout_seconds: 1800
"#,
            "config.yaml",
        );

        let app_config =
            AppConfig::load_from_file(&temp_dir.path().join("config.yaml")).unwrap();
        let gate_config = GateConfig::from_app_config(GateLevel::PR, &app_config);

        let report = create_pass_report(8, 2);
        let gate = CIGate::evaluate(&report, &gate_config);

        assert!(
            gate.is_passed(),
            "PR gate should pass with 80% (8/10) when configured with 0.80 threshold"
        );
    }

    #[test]
    fn test_gate_fails_below_custom_threshold_from_config() {
        let temp_dir = create_test_config_file(
            r#"
gate_thresholds:
  pr_pass_rate: 0.90
  nightly_pass_rate: 0.70
  release_pass_rate: 0.95
  pr_max_warnings: 3
  nightly_max_warnings: 8
  release_max_warnings: 1
  error_rate_threshold: 0.15
timeout:
  default_timeout_seconds: 500
  suite_timeout_seconds: 120
  max_timeout_seconds: 1800
"#,
            "config.yaml",
        );

        let app_config =
            AppConfig::load_from_file(&temp_dir.path().join("config.yaml")).unwrap();
        let gate_config = GateConfig::from_app_config(GateLevel::PR, &app_config);

        let report = create_pass_report(8, 2);
        let gate = CIGate::evaluate(&report, &gate_config);

        assert!(
            !gate.is_passed(),
            "PR gate should fail with 80% (8/10) when configured with 0.90 threshold"
        );
    }
}

mod timeout_config_tests {
    use super::*;

    #[test]
    fn test_timeout_values_read_from_config_file() {
        let temp_dir = create_test_config_file(
            r#"
gate_thresholds:
  pr_pass_rate: 0.90
  nightly_pass_rate: 0.80
  release_pass_rate: 1.0
  pr_max_warnings: 5
  nightly_max_warnings: 10
  release_max_warnings: 0
  error_rate_threshold: 0.1
timeout:
  default_timeout_seconds: 500
  suite_timeout_seconds: 120
  max_timeout_seconds: 1800
"#,
            "config.yaml",
        );

        let config =
            AppConfig::load_from_file(&temp_dir.path().join("config.yaml")).unwrap();

        assert_eq!(
            config.timeout.default_timeout_seconds, 500,
            "default_timeout_seconds should be 500 from config"
        );
        assert_eq!(
            config.timeout.suite_timeout_seconds, 120,
            "suite_timeout_seconds should be 120 from config"
        );
        assert_eq!(
            config.timeout.max_timeout_seconds, 1800,
            "max_timeout_seconds should be 1800 from config"
        );
    }

    #[test]
    fn test_timeout_values_use_defaults_when_not_specified() {
        let temp_dir = create_test_config_file(
            r#"
gate_thresholds:
  pr_pass_rate: 0.90
"#,
            "partial.yaml",
        );

        let config =
            AppConfig::load_from_file(&temp_dir.path().join("partial.yaml")).unwrap();

        assert_eq!(
            config.timeout.default_timeout_seconds, 300,
            "default_timeout_seconds should default to 300"
        );
        assert_eq!(
            config.timeout.suite_timeout_seconds, 60,
            "suite_timeout_seconds should default to 60"
        );
        assert_eq!(
            config.timeout.max_timeout_seconds, 3600,
            "max_timeout_seconds should default to 3600"
        );
    }
}

mod default_config_tests {
    use super::*;

    #[test]
    fn test_default_config_values_used_when_config_file_missing() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent_path = temp_dir.path().join("nonexistent.yaml");

        let result = AppConfig::load_from_file(&nonexistent_path);
        assert!(
            matches!(result, Err(AppConfigError::NotFound { .. })),
            "Should return NotFound error when config file does not exist"
        );
    }

    #[test]
    fn test_default_gate_thresholds() {
        let config = AppConfig::default();

        assert!(
            (config.gate_thresholds.pr_pass_rate - 0.90).abs() < 0.01,
            "Default PR pass rate should be 0.90"
        );
        assert!(
            (config.gate_thresholds.nightly_pass_rate - 0.80).abs() < 0.01,
            "Default nightly pass rate should be 0.80"
        );
        assert!(
            (config.gate_thresholds.release_pass_rate - 1.0).abs() < 0.01,
            "Default release pass rate should be 1.0"
        );
        assert_eq!(
            config.gate_thresholds.pr_max_warnings, 5,
            "Default PR max warnings should be 5"
        );
        assert_eq!(
            config.gate_thresholds.nightly_max_warnings, 10,
            "Default nightly max warnings should be 10"
        );
        assert_eq!(
            config.gate_thresholds.release_max_warnings, 0,
            "Default release max warnings should be 0"
        );
        assert!(
            (config.gate_thresholds.error_rate_threshold - 0.1).abs() < 0.01,
            "Default error rate threshold should be 0.1"
        );
    }

    #[test]
    fn test_default_timeout_values() {
        let config = AppConfig::default();

        assert_eq!(
            config.timeout.default_timeout_seconds, 300,
            "Default default_timeout_seconds should be 300"
        );
        assert_eq!(
            config.timeout.suite_timeout_seconds, 60,
            "Default suite_timeout_seconds should be 60"
        );
        assert_eq!(
            config.timeout.max_timeout_seconds, 3600,
            "Default max_timeout_seconds should be 3600"
        );
    }

    #[test]
    fn test_gate_config_from_default_config() {
        let app_config = AppConfig::default();
        let gate_config = GateConfig::from_app_config(GateLevel::PR, &app_config);

        assert!(
            (gate_config.pass_rate_threshold - 0.90).abs() < 0.01,
            "GateConfig should use default PR pass rate 0.90"
        );
        assert_eq!(
            gate_config.max_warnings, 5,
            "GateConfig should use default PR max warnings 5"
        );
        assert!(
            (gate_config.error_rate_threshold - 0.1).abs() < 0.01,
            "GateConfig should use default error rate threshold 0.1"
        );
    }
}

mod malformed_config_tests {
    use super::*;

    #[test]
    fn test_handles_malformed_yaml_config_gracefully() {
        let temp_dir = create_test_config_file(
            "invalid: yaml: content: [\n  incomplete",
            "malformed.yaml",
        );

        let result = AppConfig::load_from_file(&temp_dir.path().join("malformed.yaml"));
        assert!(
            matches!(result, Err(AppConfigError::YamlParse(_))),
            "Should return YamlParse error for malformed YAML"
        );
    }

    #[test]
    fn test_handles_malformed_json_config_gracefully() {
        let temp_dir = create_test_config_file(
            "{ invalid json }",
            "malformed.json",
        );

        let result = AppConfig::load_from_file(&temp_dir.path().join("malformed.json"));
        assert!(
            matches!(result, Err(AppConfigError::JsonParse(_))),
            "Should return JsonParse error for malformed JSON"
        );
    }

    #[test]
    fn test_handles_missing_config_file_gracefully() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent_path = temp_dir.path().join("does_not_exist.yaml");

        let result = AppConfig::load_from_file(&nonexistent_path);
        assert!(
            matches!(result, Err(AppConfigError::NotFound { .. })),
            "Should return NotFound error for missing config file"
        );

        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("does_not_exist.yaml"),
            "Error message should contain the missing file path"
        );
    }

    #[test]
    fn test_partial_config_does_not_panic() {
        let temp_dir = create_test_config_file(
            r#"
gate_thresholds:
  pr_pass_rate: 0.92
  nightly_pass_rate: 0.75
"#,
            "partial.yaml",
        );

        let result = AppConfig::load_from_file(&temp_dir.path().join("partial.yaml"));
        assert!(
            result.is_ok(),
            "Partial config should load successfully"
        );

        let config = result.unwrap();
        assert!(
            (config.gate_thresholds.pr_pass_rate - 0.92).abs() < 0.01,
            "PR pass rate from partial config should be 0.92"
        );
        assert!(
            (config.gate_thresholds.nightly_pass_rate - 0.75).abs() < 0.01,
            "Nightly pass rate from partial config should be 0.75"
        );
    }
}

mod config_file_format_tests {
    use super::*;

    #[test]
    fn test_loads_yaml_config_by_extension() {
        let temp_dir = create_test_config_file(
            r#"
gate_thresholds:
  pr_pass_rate: 0.88
  nightly_pass_rate: 0.72
  release_pass_rate: 0.96
  pr_max_warnings: 4
  nightly_max_warnings: 9
  release_max_warnings: 2
  error_rate_threshold: 0.12
timeout:
  default_timeout_seconds: 400
  suite_timeout_seconds: 90
  max_timeout_seconds: 2400
"#,
            "config.yaml",
        );

        let config =
            AppConfig::load_from_file(&temp_dir.path().join("config.yaml")).unwrap();

        assert!(
            (config.gate_thresholds.pr_pass_rate - 0.88).abs() < 0.01,
            "YAML config should load correctly"
        );
    }

    #[test]
    fn test_loads_json_config_by_extension() {
        let temp_dir = create_test_config_file(
            r#"{
                "gate_thresholds": {
                    "pr_pass_rate": 0.87,
                    "nightly_pass_rate": 0.73,
                    "release_pass_rate": 0.97,
                    "pr_max_warnings": 6,
                    "nightly_max_warnings": 11,
                    "release_max_warnings": 1,
                    "error_rate_threshold": 0.08
                },
                "timeout": {
                    "default_timeout_seconds": 350,
                    "suite_timeout_seconds": 75,
                    "max_timeout_seconds": 2100
                }
            }"#,
            "config.json",
        );

        let config =
            AppConfig::load_from_file(&temp_dir.path().join("config.json")).unwrap();

        assert!(
            (config.gate_thresholds.pr_pass_rate - 0.87).abs() < 0.01,
            "JSON config should load correctly"
        );
        assert_eq!(
            config.timeout.default_timeout_seconds, 350,
            "JSON config timeout should load correctly"
        );
    }

    #[test]
    fn test_loads_yml_extension_config() {
        let temp_dir = create_test_config_file(
            r#"
gate_thresholds:
  pr_pass_rate: 0.86
  nightly_pass_rate: 0.74
  release_pass_rate: 0.98
  pr_max_warnings: 7
  nightly_max_warnings: 12
  release_max_warnings: 3
  error_rate_threshold: 0.06
timeout:
  default_timeout_seconds: 250
  suite_timeout_seconds: 55
  max_timeout_seconds: 1500
"#,
            "config.yml",
        );

        let config =
            AppConfig::load_from_file(&temp_dir.path().join("config.yml")).unwrap();

        assert!(
            (config.gate_thresholds.pr_pass_rate - 0.86).abs() < 0.01,
            "YML config should load correctly"
        );
    }
}