use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{Comparator, ComparisonOutcome, ComparisonResult, NormalizedComparator};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptPattern {
    pub name: String,
    pub pattern: String,
    pub prompt_type: PromptType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromptType {
    PermissionRequest,
    Confirmation,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionPrompt {
    pub prompt_type: PromptType,
    pub message: String,
    pub line_number: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromptSequence {
    pub prompts: Vec<PermissionPrompt>,
}

impl PromptSequence {
    pub fn new() -> Self {
        Self {
            prompts: Vec::new(),
        }
    }

    pub fn from_prompts(prompts: Vec<PermissionPrompt>) -> Self {
        Self { prompts }
    }

    pub fn is_empty(&self) -> bool {
        self.prompts.is_empty()
    }

    pub fn len(&self) -> usize {
        self.prompts.len()
    }

    pub fn count_by_type(&self, prompt_type: PromptType) -> usize {
        self.prompts
            .iter()
            .filter(|p| p.prompt_type == prompt_type)
            .count()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionComparatorConfig {
    pub prompt_patterns: Vec<PromptPattern>,
    pub case_sensitive: bool,
}

impl Default for PermissionComparatorConfig {
    fn default() -> Self {
        Self {
            prompt_patterns: vec![
                PromptPattern {
                    name: "permission_request".to_string(),
                    pattern: r"(?i)\b(permission|authorize|allow|grant)\b".to_string(),
                    prompt_type: PromptType::PermissionRequest,
                },
                PromptPattern {
                    name: "confirmation".to_string(),
                    pattern: r"(?i)\b(confirm|yes|no|proceed|cancel)\b".to_string(),
                    prompt_type: PromptType::Confirmation,
                },
                PromptPattern {
                    name: "warning".to_string(),
                    pattern: r"(?i)\bwarning\b".to_string(),
                    prompt_type: PromptType::Warning,
                },
                PromptPattern {
                    name: "error".to_string(),
                    pattern: r"(?i)\b(error|failed|failure)\b".to_string(),
                    prompt_type: PromptType::Error,
                },
            ],
            case_sensitive: false,
        }
    }
}

impl PermissionComparatorConfig {
    pub fn new(prompt_patterns: Vec<PromptPattern>) -> Self {
        Self {
            prompt_patterns,
            case_sensitive: false,
        }
    }

    pub fn with_case_sensitivity(mut self, case_sensitive: bool) -> Self {
        self.case_sensitive = case_sensitive;
        self
    }
}

#[derive(Debug, Clone)]
pub struct PromptPatternMatcher {
    patterns: Vec<PromptPattern>,
    case_sensitive: bool,
}

impl PromptPatternMatcher {
    pub fn new(config: &PermissionComparatorConfig) -> Self {
        Self {
            patterns: config.prompt_patterns.clone(),
            case_sensitive: config.case_sensitive,
        }
    }

    pub fn detect_prompts(&self, output: &str) -> PromptSequence {
        let mut prompts = Vec::new();

        for (line_number, line) in output.lines().enumerate() {
            let search_text = if self.case_sensitive {
                line.to_string()
            } else {
                line.to_lowercase()
            };

            for pattern in &self.patterns {
                let pattern_text = if self.case_sensitive {
                    pattern.pattern.clone()
                } else {
                    pattern.pattern.to_lowercase()
                };

                if let Ok(regex) = Regex::new(&pattern_text) {
                    if regex.is_match(&search_text) {
                        prompts.push(PermissionPrompt {
                            prompt_type: pattern.prompt_type,
                            message: line.to_string(),
                            line_number,
                        });
                        break;
                    }
                }
            }
        }

        PromptSequence::from_prompts(prompts)
    }

    pub fn has_permission_prompt(&self, output: &str) -> bool {
        let sequence = self.detect_prompts(output);
        !sequence.prompts.is_empty()
    }

    pub fn count_permission_prompts(&self, output: &str) -> usize {
        let sequence = self.detect_prompts(output);
        sequence.count_by_type(PromptType::PermissionRequest)
    }
}

#[derive(Debug, Clone)]
pub struct BehaviorComparator {
    output_comparator: NormalizedComparator,
}

impl BehaviorComparator {
    pub fn new() -> Self {
        Self {
            output_comparator: NormalizedComparator,
        }
    }

    pub fn compare_behavior(&self, output1: &str, output2: &str) -> ComparisonResult {
        self.output_comparator.compare(output1, output2)
    }
}

impl Default for BehaviorComparator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PermissionComparator {
    prompt_detector: PromptPatternMatcher,
    behavior_comparator: BehaviorComparator,
}

impl PermissionComparator {
    pub fn new(config: PermissionComparatorConfig) -> Self {
        Self {
            prompt_detector: PromptPatternMatcher::new(&config),
            behavior_comparator: BehaviorComparator::new(),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(PermissionComparatorConfig::default())
    }

    pub fn detect_prompts(&self, output: &str) -> PromptSequence {
        self.prompt_detector.detect_prompts(output)
    }

    pub fn compare_behavior(&self, output1: &str, output2: &str) -> ComparisonResult {
        self.behavior_comparator.compare_behavior(output1, output2)
    }

    pub fn compare_with_prompt_tracking(&self, output1: &str, output2: &str) -> ComparisonResult {
        let prompts1 = self.prompt_detector.detect_prompts(output1);
        let prompts2 = self.prompt_detector.detect_prompts(output2);

        if prompts1.is_empty() && prompts2.is_empty() {
            return self.behavior_comparator.compare_behavior(output1, output2);
        }

        if prompts1.is_empty() || prompts2.is_empty() {
            return ComparisonResult::severely_incompatible(format!(
                "Prompt presence differs: output1 has {} prompts, output2 has {} prompts",
                prompts1.len(),
                prompts2.len()
            ));
        }

        if prompts1.len() == prompts2.len() {
            let same_types = prompts1
                .prompts
                .iter()
                .zip(prompts2.prompts.iter())
                .all(|(p1, p2)| p1.prompt_type == p2.prompt_type);

            if same_types {
                let behavior_result = self.behavior_comparator.compare_behavior(output1, output2);
                if behavior_result.outcome == ComparisonOutcome::StronglyEquivalent
                    || behavior_result.outcome == ComparisonOutcome::SemanticallyEquivalent
                {
                    return ComparisonResult::strongly_equivalent();
                }
                return ComparisonResult::allowed_variance(format!(
                    "Same prompt sequence ({} prompts), behavior differs: {}",
                    prompts1.len(),
                    behavior_result.diff.unwrap_or_default()
                ));
            }
        }

        let diff = format!(
            "Prompt sequence mismatch: output1 has {} prompts, output2 has {} prompts",
            prompts1.len(),
            prompts2.len()
        );

        ComparisonResult::mildly_incompatible(diff)
    }

    pub fn compare_permission_flows(&self, output1: &str, output2: &str) -> ComparisonResult {
        let prompts1 = self.prompt_detector.detect_prompts(output1);
        let prompts2 = self.prompt_detector.detect_prompts(output2);

        let has_prompt1 = !prompts1.is_empty();
        let has_prompt2 = !prompts2.is_empty();

        match (has_prompt1, has_prompt2) {
            (true, true) => {
                let perm_count1 = prompts1.count_by_type(PromptType::PermissionRequest);
                let perm_count2 = prompts2.count_by_type(PromptType::PermissionRequest);

                if perm_count1 == perm_count2 {
                    ComparisonResult::strongly_equivalent()
                } else {
                    ComparisonResult::mildly_incompatible(format!(
                        "Permission prompt count differs: {} vs {}",
                        perm_count1, perm_count2
                    ))
                }
            }
            (false, false) => self.behavior_comparator.compare_behavior(output1, output2),
            _ => ComparisonResult::severely_incompatible(format!(
                "Permission prompt presence differs: output1={}, output2={}",
                has_prompt1, has_prompt2
            )),
        }
    }
}

impl Comparator for PermissionComparator {
    fn compare(&self, output1: &str, output2: &str) -> ComparisonResult {
        self.compare_with_prompt_tracking(output1, output2)
    }

    fn name(&self) -> &str {
        "permission"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_comparator<T: Comparator>() {}

    #[test]
    fn permission_comparator_implements_comparator_trait() {
        assert_comparator::<PermissionComparator>();
    }

    #[test]
    fn permission_comparator_default_config() {
        let comparator = PermissionComparator::with_default_config();
        assert_eq!(comparator.name(), "permission");
    }

    #[test]
    fn prompt_pattern_matcher_detects_permission_request() {
        let matcher = PromptPatternMatcher::new(&PermissionComparatorConfig::default());
        let output = "This action requires permission to proceed.";
        let sequence = matcher.detect_prompts(output);
        assert!(!sequence.is_empty());
    }

    #[test]
    fn prompt_pattern_matcher_detects_confirmation() {
        let matcher = PromptPatternMatcher::new(&PermissionComparatorConfig::default());
        let output = "Do you want to continue? yes/no";
        let sequence = matcher.detect_prompts(output);
        assert!(!sequence.is_empty());
    }

    #[test]
    fn prompt_pattern_matcher_detects_warning() {
        let matcher = PromptPatternMatcher::new(&PermissionComparatorConfig::default());
        let output = "WARNING: This action cannot be undone.";
        let sequence = matcher.detect_prompts(output);
        assert!(!sequence.is_empty());
    }

    #[test]
    fn prompt_pattern_matcher_detects_error() {
        let matcher = PromptPatternMatcher::new(&PermissionComparatorConfig::default());
        let output = "Error: Permission denied.";
        let sequence = matcher.detect_prompts(output);
        assert!(!sequence.is_empty());
    }

    #[test]
    fn prompt_pattern_matcher_no_prompts() {
        let matcher = PromptPatternMatcher::new(&PermissionComparatorConfig::default());
        let output = "Just a regular output message here.";
        let sequence = matcher.detect_prompts(output);
        assert!(sequence.is_empty());
    }

    #[test]
    fn prompt_pattern_matcher_has_permission_prompt() {
        let matcher = PromptPatternMatcher::new(&PermissionComparatorConfig::default());

        assert!(matcher.has_permission_prompt("Permission required"));
        assert!(matcher.has_permission_prompt("AUTHORIZE the action"));
        assert!(!matcher.has_permission_prompt("Normal output"));
    }

    #[test]
    fn prompt_pattern_matcher_count_permission_prompts() {
        let matcher = PromptPatternMatcher::new(&PermissionComparatorConfig::default());
        let output = "Permission 1\nPermission 2\nNo prompt here\nPermission 3";
        assert_eq!(matcher.count_permission_prompts(output), 3);
    }

    #[test]
    fn permission_comparator_detects_prompts() {
        let comparator = PermissionComparator::with_default_config();
        let output = "Do you want to grant permission?";
        let sequence = comparator.detect_prompts(output);
        assert!(!sequence.is_empty());
    }

    #[test]
    fn permission_comparator_both_have_same_prompts() {
        let comparator = PermissionComparator::with_default_config();
        let output1 = "Permission required\nDo you approve?";
        let output2 = "Permission required\nDo you approve?";

        let result = comparator.compare(output1, output2);
        assert_eq!(result.outcome, ComparisonOutcome::StronglyEquivalent);
    }

    #[test]
    fn permission_comparator_both_have_no_prompts() {
        let comparator = PermissionComparator::with_default_config();
        let output1 = "Normal operation completed";
        let output2 = "Normal operation completed";

        let result = comparator.compare(output1, output2);
        assert_eq!(result.outcome, ComparisonOutcome::SemanticallyEquivalent);
    }

    #[test]
    fn permission_comparator_prompt_count_differs() {
        let comparator = PermissionComparator::with_default_config();
        let output1 = "Permission 1\nPermission 2";
        let output2 = "Permission 1";

        let result = comparator.compare(output1, output2);
        assert_eq!(result.outcome, ComparisonOutcome::MildlyIncompatible);
    }

    #[test]
    fn permission_comparator_one_has_prompt_other_not() {
        let comparator = PermissionComparator::with_default_config();
        let output1 = "Permission required to proceed";
        let output2 = "All clear, proceeding";

        let result = comparator.compare(output1, output2);
        assert_eq!(result.outcome, ComparisonOutcome::SeverelyIncompatible);
    }

    #[test]
    fn permission_comparator_compare_behavior() {
        let comparator = PermissionComparator::with_default_config();
        let output1 = "  Same  output  ";
        let output2 = "Same output";

        let result = comparator.compare_behavior(output1, output2);
        assert_eq!(result.outcome, ComparisonOutcome::SemanticallyEquivalent);
    }

    #[test]
    fn permission_comparator_compare_permission_flows() {
        let comparator = PermissionComparator::with_default_config();

        let output1 = "Permission request 1\nPermission request 2";
        let output2 = "Permission request 1\nPermission request 2";

        let result = comparator.compare_permission_flows(output1, output2);
        assert_eq!(result.outcome, ComparisonOutcome::StronglyEquivalent);
    }

    #[test]
    fn permission_comparator_permission_flows_count_differs() {
        let comparator = PermissionComparator::with_default_config();

        let output1 = "Permission request 1\nPermission request 2";
        let output2 = "Permission request 1";

        let result = comparator.compare_permission_flows(output1, output2);
        assert_eq!(result.outcome, ComparisonOutcome::MildlyIncompatible);
    }

    #[test]
    fn prompt_sequence_len() {
        let prompts = vec![
            PermissionPrompt {
                prompt_type: PromptType::PermissionRequest,
                message: "msg1".to_string(),
                line_number: 0,
            },
            PermissionPrompt {
                prompt_type: PromptType::Confirmation,
                message: "msg2".to_string(),
                line_number: 1,
            },
        ];
        let sequence = PromptSequence::from_prompts(prompts);
        assert_eq!(sequence.len(), 2);
    }

    #[test]
    fn prompt_sequence_count_by_type() {
        let prompts = vec![
            PermissionPrompt {
                prompt_type: PromptType::PermissionRequest,
                message: "msg1".to_string(),
                line_number: 0,
            },
            PermissionPrompt {
                prompt_type: PromptType::PermissionRequest,
                message: "msg2".to_string(),
                line_number: 1,
            },
            PermissionPrompt {
                prompt_type: PromptType::Confirmation,
                message: "msg3".to_string(),
                line_number: 2,
            },
        ];
        let sequence = PromptSequence::from_prompts(prompts);
        assert_eq!(sequence.count_by_type(PromptType::PermissionRequest), 2);
        assert_eq!(sequence.count_by_type(PromptType::Confirmation), 1);
    }
}
