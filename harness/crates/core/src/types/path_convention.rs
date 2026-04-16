#[derive(Debug, Clone)]
pub struct PathConvention;

impl PathConvention {
    pub const RUN_ARTIFACTS: &str = "artifacts/run-{id}";
    pub const SESSION_DATA: &str = "sessions/iteration-{n}";
    pub const REPORTS: &str = "harness/reports/{suite}/{timestamp}";
    pub const TASKS: &str = "tasks";
    pub const FIXTURES: &str = "fixtures/projects";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artifacts_directory_exists() {
        let artifacts_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("..")
            .join("artifacts");
        assert!(
            artifacts_path.exists(),
            "artifacts directory should exist at {:?}",
            artifacts_path
        );
        assert!(artifacts_path.is_dir(), "artifacts should be a directory");
    }

    #[test]
    fn test_run_placeholder_directory_exists() {
        let run_placeholder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("..")
            .join("artifacts")
            .join("run-placeholder");
        assert!(
            run_placeholder.exists(),
            "run-placeholder directory should exist at {:?}",
            run_placeholder
        );
        assert!(
            run_placeholder.is_dir(),
            "run-placeholder should be a directory"
        );
    }

    #[test]
    fn test_run_artifacts_path_convention() {
        assert_eq!(PathConvention::RUN_ARTIFACTS, "artifacts/run-{id}");
    }

    #[test]
    fn test_run_artifacts_pattern_matches_run_id() {
        let pattern = PathConvention::RUN_ARTIFACTS;
        assert!(pattern.starts_with("artifacts/run-"));
        assert!(pattern.contains("{id}"));
    }

    #[test]
    fn test_reports_constant_equals_expected_value() {
        assert_eq!(
            PathConvention::REPORTS,
            "harness/reports/{suite}/{timestamp}"
        );
    }

    #[test]
    fn test_reports_path_format_is_correctly_structured() {
        let pattern = PathConvention::REPORTS;
        assert!(
            pattern.starts_with("harness/reports/"),
            "REPORTS should start with harness/reports/"
        );
        assert!(
            pattern.contains("{suite}"),
            "REPORTS should contain {{suite}} placeholder"
        );
        assert!(
            pattern.contains("{timestamp}"),
            "REPORTS should contain {{timestamp}} placeholder"
        );
        assert_eq!(pattern, "harness/reports/{suite}/{timestamp}");
    }

    #[test]
    fn test_reports_preserves_placeholders() {
        let reports_path = PathConvention::REPORTS;
        let suite = "test-suite";
        let timestamp = "20260416-143052";
        let formatted = reports_path
            .replace("{suite}", suite)
            .replace("{timestamp}", timestamp);
        assert_eq!(
            formatted,
            format!("harness/reports/{}/{}", suite, timestamp)
        );
    }
}
