/// GitHub Actions annotations for CI feedback.
#[derive(Debug, Clone)]
pub struct GitHubAnnotations {
    /// Collected error annotations.
    errors: Vec<Annotation>,
    /// Collected warning annotations.
    warnings: Vec<Annotation>,
    /// Collected notice annotations.
    notices: Vec<Annotation>,
}

#[derive(Debug, Clone)]
struct Annotation {
    message: String,
    file: Option<String>,
    line: Option<u32>,
}

impl GitHubAnnotations {
    /// Creates a new empty GitHub annotations collector.
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            notices: Vec::new(),
        }
    }

    /// Adds an error annotation.
    pub fn add_error(&mut self, message: impl Into<String>) -> &mut Self {
        self.errors.push(Annotation {
            message: message.into(),
            file: None,
            line: None,
        });
        self
    }

    /// Adds an error annotation with file location.
    pub fn add_error_at(&mut self, message: impl Into<String>, file: &str, line: u32) -> &mut Self {
        self.errors.push(Annotation {
            message: message.into(),
            file: Some(file.to_string()),
            line: Some(line),
        });
        self
    }

    /// Adds a warning annotation.
    pub fn add_warning(&mut self, message: impl Into<String>) -> &mut Self {
        self.warnings.push(Annotation {
            message: message.into(),
            file: None,
            line: None,
        });
        self
    }

    /// Adds a warning annotation with file location.
    pub fn add_warning_at(
        &mut self,
        message: impl Into<String>,
        file: &str,
        line: u32,
    ) -> &mut Self {
        self.warnings.push(Annotation {
            message: message.into(),
            file: Some(file.to_string()),
            line: Some(line),
        });
        self
    }

    /// Adds a notice annotation.
    pub fn add_notice(&mut self, message: impl Into<String>) -> &mut Self {
        self.notices.push(Annotation {
            message: message.into(),
            file: None,
            line: None,
        });
        self
    }

    /// Adds a notice annotation with file location.
    pub fn add_notice_at(
        &mut self,
        message: impl Into<String>,
        file: &str,
        line: u32,
    ) -> &mut Self {
        self.notices.push(Annotation {
            message: message.into(),
            file: Some(file.to_string()),
            line: Some(line),
        });
        self
    }

    /// Returns the number of errors.
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Returns the number of warnings.
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    /// Returns the number of notices.
    pub fn notice_count(&self) -> usize {
        self.notices.len()
    }

    /// Returns true if there are no annotations.
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty() && self.warnings.is_empty() && self.notices.is_empty()
    }

    /// Returns the total number of annotations.
    pub fn len(&self) -> usize {
        self.errors.len() + self.warnings.len() + self.notices.len()
    }

    /// Converts annotations to GitHub Actions workflow commands format.
    pub fn to_steps_output(&self) -> String {
        let mut output = String::new();

        for error in &self.errors {
            if let (Some(file), Some(line)) = (&error.file, &error.line) {
                output.push_str(&format!(
                    "::error file={},line={}::{}\n",
                    file, line, error.message
                ));
            } else {
                output.push_str(&format!("::error ::{}\n", error.message));
            }
        }

        for warning in &self.warnings {
            if let (Some(file), Some(line)) = (&warning.file, &warning.line) {
                output.push_str(&format!(
                    "::warning file={},line={}::{}\n",
                    file, line, warning.message
                ));
            } else {
                output.push_str(&format!("::warning ::{}\n", warning.message));
            }
        }

        for notice in &self.notices {
            if let (Some(file), Some(line)) = (&notice.file, &notice.line) {
                output.push_str(&format!(
                    "::notice file={},line={}::{}\n",
                    file, line, notice.message
                ));
            } else {
                output.push_str(&format!("::notice ::{}\n", notice.message));
            }
        }

        output
    }

    /// Returns a summary of annotations for logging.
    pub fn summary(&self) -> String {
        format!(
            "{} error(s), {} warning(s), {} notice(s)",
            self.error_count(),
            self.warning_count(),
            self.notice_count()
        )
    }
}

impl Default for GitHubAnnotations {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_annotations_creation() {
        let annotations = GitHubAnnotations::new();
        assert!(annotations.is_empty());
        assert_eq!(annotations.len(), 0);
    }

    #[test]
    fn test_add_error() {
        let mut annotations = GitHubAnnotations::new();
        annotations.add_error("Test error message");

        assert_eq!(annotations.error_count(), 1);
        assert_eq!(annotations.warning_count(), 0);
        assert_eq!(annotations.notice_count(), 0);
    }

    #[test]
    fn test_add_warning() {
        let mut annotations = GitHubAnnotations::new();
        annotations.add_warning("Test warning message");

        assert_eq!(annotations.warning_count(), 1);
        assert!(annotations.errors.is_empty());
    }

    #[test]
    fn test_add_notice() {
        let mut annotations = GitHubAnnotations::new();
        annotations.add_notice("Test notice message");

        assert_eq!(annotations.notice_count(), 1);
    }

    #[test]
    fn test_add_error_at() {
        let mut annotations = GitHubAnnotations::new();
        annotations.add_error_at("Error on line 10", "test.rs", 10);

        let output = annotations.to_steps_output();
        assert!(output.contains("file=test.rs,line=10"));
        assert!(output.contains("::error"));
    }

    #[test]
    fn test_chaining() {
        let mut annotations = GitHubAnnotations::new();
        annotations
            .add_error("Error 1")
            .add_warning("Warning 1")
            .add_notice("Notice 1")
            .add_error("Error 2");

        assert_eq!(annotations.error_count(), 2);
        assert_eq!(annotations.warning_count(), 1);
        assert_eq!(annotations.notice_count(), 1);
    }

    #[test]
    fn test_to_steps_output_errors() {
        let mut annotations = GitHubAnnotations::new();
        annotations.add_error("Critical error");

        let output = annotations.to_steps_output();
        assert!(output.contains("::error ::Critical error"));
    }

    #[test]
    fn test_to_steps_output_warnings() {
        let mut annotations = GitHubAnnotations::new();
        annotations.add_warning("Important warning");

        let output = annotations.to_steps_output();
        assert!(output.contains("::warning ::Important warning"));
    }

    #[test]
    fn test_to_steps_output_notices() {
        let mut annotations = GitHubAnnotations::new();
        annotations.add_notice("Informational notice");

        let output = annotations.to_steps_output();
        assert!(output.contains("::notice ::Informational notice"));
    }

    #[test]
    fn test_to_steps_output_mixed() {
        let mut annotations = GitHubAnnotations::new();
        annotations
            .add_error("Error 1")
            .add_warning("Warning 1")
            .add_notice("Notice 1")
            .add_error_at("Error 2 at line 5", "src/main.rs", 5);

        let output = annotations.to_steps_output();
        assert!(output.contains("::error"));
        assert!(output.contains("::warning"));
        assert!(output.contains("::notice"));
    }

    #[test]
    fn test_summary() {
        let mut annotations = GitHubAnnotations::new();
        annotations
            .add_error("Error 1")
            .add_error("Error 2")
            .add_warning("Warning 1")
            .add_notice("Notice 1");

        let summary = annotations.summary();
        assert!(summary.contains("2 error(s)"));
        assert!(summary.contains("1 warning(s)"));
        assert!(summary.contains("1 notice(s)"));
    }

    #[test]
    fn test_len() {
        let mut annotations = GitHubAnnotations::new();
        assert_eq!(annotations.len(), 0);

        annotations.add_error("1");
        annotations.add_warning("2");
        annotations.add_notice("3");
        annotations.add_notice("4");

        assert_eq!(annotations.len(), 4);
    }

    #[test]
    fn test_empty_output() {
        let annotations = GitHubAnnotations::new();
        let output = annotations.to_steps_output();
        assert!(output.is_empty());
    }
}
