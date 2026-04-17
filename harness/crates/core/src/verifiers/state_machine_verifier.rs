use crate::types::parity_verdict::{DiffCategory, ParityVerdict};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMachineContract {
    pub contract_id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub state_definitions: Vec<StateDefinition>,
    pub transitions: Vec<TransitionDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDefinition {
    pub id: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionDefinition {
    pub from: String,
    pub to: String,
    pub trigger: String,
}

impl StateMachineContract {
    pub fn new(
        contract_id: String,
        name: String,
        version: String,
        description: String,
        state_definitions: Vec<StateDefinition>,
        transitions: Vec<TransitionDefinition>,
    ) -> Self {
        Self {
            contract_id,
            name,
            version,
            description,
            state_definitions,
            transitions,
        }
    }

    pub fn is_valid_transition(&self, from_state: &str, to_state: &str) -> bool {
        self.transitions
            .iter()
            .any(|t| t.from == from_state && t.to == to_state)
    }

    pub fn get_allowed_transitions_from(&self, from_state: &str) -> Vec<&TransitionDefinition> {
        self.transitions
            .iter()
            .filter(|t| t.from == from_state)
            .collect()
    }

    pub fn get_valid_states(&self) -> Vec<&str> {
        self.state_definitions
            .iter()
            .map(|s| s.id.as_str())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateTransition {
    pub from_state: String,
    pub to_state: String,
    pub trigger: String,
    pub timestamp: DateTime<Utc>,
}

impl StateTransition {
    pub fn new(from_state: String, to_state: String, trigger: String) -> Self {
        Self {
            from_state,
            to_state,
            trigger,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidTransition {
    pub from_state: String,
    pub attempted_to: String,
    pub reason: String,
}

impl InvalidTransition {
    pub fn new(from_state: String, attempted_to: String, reason: String) -> Self {
        Self {
            from_state,
            attempted_to,
            reason,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMachineVerificationResult {
    pub verdict: ParityVerdict,
    pub valid_transitions: Vec<StateTransition>,
    pub invalid_transitions: Vec<InvalidTransition>,
    pub missing_transitions: Vec<StateTransition>,
    pub extra_transitions: Vec<StateTransition>,
}

impl StateMachineVerificationResult {
    pub fn pass() -> Self {
        Self {
            verdict: ParityVerdict::Pass,
            valid_transitions: Vec::new(),
            invalid_transitions: Vec::new(),
            missing_transitions: Vec::new(),
            extra_transitions: Vec::new(),
        }
    }

    pub fn fail(
        details: String,
        valid_transitions: Vec<StateTransition>,
        invalid_transitions: Vec<InvalidTransition>,
        missing_transitions: Vec<StateTransition>,
        extra_transitions: Vec<StateTransition>,
    ) -> Self {
        Self {
            verdict: ParityVerdict::Fail {
                category: DiffCategory::Protocol,
                details,
            },
            valid_transitions,
            invalid_transitions,
            missing_transitions,
            extra_transitions,
        }
    }
}

pub trait StateMachineVerifier: Send + Sync {
    fn verify_state_transition(
        &self,
        contract: &StateMachineContract,
        expected_sequence: &[StateTransition],
        actual_sequence: &[StateTransition],
    ) -> StateMachineVerificationResult;
}

#[derive(Debug, Clone, Default)]
pub struct DefaultStateMachineVerifier;

impl DefaultStateMachineVerifier {
    pub fn new() -> Self {
        Self
    }
}

impl StateMachineVerifier for DefaultStateMachineVerifier {
    fn verify_state_transition(
        &self,
        contract: &StateMachineContract,
        expected_sequence: &[StateTransition],
        actual_sequence: &[StateTransition],
    ) -> StateMachineVerificationResult {
        let mut valid_transitions = Vec::new();
        let mut invalid_transitions = Vec::new();
        let mut missing_transitions = Vec::new();
        let mut extra_transitions = Vec::new();

        for actual in actual_sequence {
            if contract.is_valid_transition(&actual.from_state, &actual.to_state) {
                valid_transitions.push(actual.clone());
            } else {
                let allowed = contract.get_allowed_transitions_from(&actual.from_state);
                let reason = if allowed.is_empty() {
                    format!(
                        "State '{}' has no valid outgoing transitions",
                        actual.from_state
                    )
                } else {
                    format!(
                        "Invalid transition from '{}' to '{}'. Allowed transitions: {:?}",
                        actual.from_state,
                        actual.to_state,
                        allowed
                            .iter()
                            .map(|t| format!("{} -> {}", t.from, t.to))
                            .collect::<Vec<_>>()
                    )
                };
                invalid_transitions.push(InvalidTransition::new(
                    actual.from_state.clone(),
                    actual.to_state.clone(),
                    reason,
                ));
            }
        }

        for expected in expected_sequence {
            if !actual_sequence.iter().any(|a| {
                a.from_state == expected.from_state
                    && a.to_state == expected.to_state
                    && a.trigger == expected.trigger
            }) {
                missing_transitions.push(expected.clone());
            }
        }

        for actual in actual_sequence {
            if !expected_sequence.iter().any(|e| {
                e.from_state == actual.from_state
                    && e.to_state == actual.to_state
                    && e.trigger == actual.trigger
            }) {
                extra_transitions.push(actual.clone());
            }
        }

        let has_invalid = !invalid_transitions.is_empty();
        let has_missing = !missing_transitions.is_empty();
        let has_extra = !extra_transitions.is_empty();

        let verdict = if has_invalid {
            let details = format!(
                "Invalid transitions detected: {:?}",
                invalid_transitions
                    .iter()
                    .map(|i| format!("{} -> {}: {}", i.from_state, i.attempted_to, i.reason))
                    .collect::<Vec<_>>()
            );
            ParityVerdict::Fail {
                category: DiffCategory::Protocol,
                details,
            }
        } else if has_missing && has_extra {
            ParityVerdict::Fail {
                category: DiffCategory::Protocol,
                details: format!(
                    "Missing transitions: {:?}, Extra transitions: {:?}",
                    missing_transitions, extra_transitions
                ),
            }
        } else if has_missing {
            ParityVerdict::Fail {
                category: DiffCategory::Protocol,
                details: format!("Missing transitions: {:?}", missing_transitions),
            }
        } else if has_extra {
            ParityVerdict::Fail {
                category: DiffCategory::Protocol,
                details: format!("Extra transitions: {:?}", extra_transitions),
            }
        } else {
            ParityVerdict::Pass
        };

        StateMachineVerificationResult {
            verdict,
            valid_transitions,
            invalid_transitions,
            missing_transitions,
            extra_transitions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_contract() -> StateMachineContract {
        StateMachineContract::new(
            "TEST-001".to_string(),
            "Test Contract".to_string(),
            "1.0.0".to_string(),
            "Test contract for state machine verification".to_string(),
            vec![
                StateDefinition {
                    id: "NEW".to_string(),
                    description: "Fresh session".to_string(),
                },
                StateDefinition {
                    id: "PROJECT_BOUND".to_string(),
                    description: "Project bound".to_string(),
                },
                StateDefinition {
                    id: "AGENT_ACTIVE".to_string(),
                    description: "Agent active".to_string(),
                },
                StateDefinition {
                    id: "DONE".to_string(),
                    description: "Task completed".to_string(),
                },
            ],
            vec![
                TransitionDefinition {
                    from: "NEW".to_string(),
                    to: "PROJECT_BOUND".to_string(),
                    trigger: "project_command".to_string(),
                },
                TransitionDefinition {
                    from: "PROJECT_BOUND".to_string(),
                    to: "AGENT_ACTIVE".to_string(),
                    trigger: "task_start".to_string(),
                },
                TransitionDefinition {
                    from: "AGENT_ACTIVE".to_string(),
                    to: "DONE".to_string(),
                    trigger: "task_complete".to_string(),
                },
            ],
        )
    }

    fn create_state_transition(from_state: &str, to_state: &str, trigger: &str) -> StateTransition {
        StateTransition::new(
            from_state.to_string(),
            to_state.to_string(),
            trigger.to_string(),
        )
    }

    #[test]
    fn test_state_transition_struct_captures_fields() {
        let transition = create_state_transition("NEW", "PROJECT_BOUND", "project_command");
        assert_eq!(transition.from_state, "NEW");
        assert_eq!(transition.to_state, "PROJECT_BOUND");
        assert_eq!(transition.trigger, "project_command");
        assert!(transition.timestamp <= Utc::now());
    }

    #[test]
    fn test_invalid_transition_struct_captures_fields() {
        let invalid = InvalidTransition::new(
            "NEW".to_string(),
            "DONE".to_string(),
            "Invalid transition".to_string(),
        );
        assert_eq!(invalid.from_state, "NEW");
        assert_eq!(invalid.attempted_to, "DONE");
        assert_eq!(invalid.reason, "Invalid transition");
    }

    #[test]
    fn test_state_machine_verification_result_pass() {
        let result = StateMachineVerificationResult::pass();
        assert!(result.verdict.is_identical());
        assert!(result.valid_transitions.is_empty());
        assert!(result.invalid_transitions.is_empty());
        assert!(result.missing_transitions.is_empty());
        assert!(result.extra_transitions.is_empty());
    }

    #[test]
    fn test_state_machine_verification_result_fail() {
        let result = StateMachineVerificationResult::fail(
            "Test failure".to_string(),
            vec![],
            vec![],
            vec![],
            vec![],
        );
        assert!(result.verdict.is_different());
    }

    #[test]
    fn test_state_machine_verifier_trait_is_defined() {
        fn assert_state_machine_verifier<T: StateMachineVerifier>() {}
        assert_state_machine_verifier::<DefaultStateMachineVerifier>();
    }

    #[test]
    fn test_default_state_machine_verifier_detects_invalid_transitions() {
        let verifier = DefaultStateMachineVerifier::new();
        let contract = create_test_contract();

        let actual_sequence = vec![create_state_transition("NEW", "DONE", "invalid_jump")];

        let result = verifier.verify_state_transition(&contract, &[], &actual_sequence);

        assert!(result.verdict.is_different());
        assert!(!result.invalid_transitions.is_empty());
        assert_eq!(result.invalid_transitions[0].from_state, "NEW");
        assert_eq!(result.invalid_transitions[0].attempted_to, "DONE");
    }

    #[test]
    fn test_default_state_machine_verifier_detects_missing_transitions() {
        let verifier = DefaultStateMachineVerifier::new();
        let contract = create_test_contract();

        let expected_sequence = vec![
            create_state_transition("NEW", "PROJECT_BOUND", "project_command"),
            create_state_transition("PROJECT_BOUND", "AGENT_ACTIVE", "task_start"),
        ];
        let actual_sequence = vec![create_state_transition(
            "NEW",
            "PROJECT_BOUND",
            "project_command",
        )];

        let result =
            verifier.verify_state_transition(&contract, &expected_sequence, &actual_sequence);

        assert!(result.verdict.is_different());
        assert!(!result.missing_transitions.is_empty());
    }

    #[test]
    fn test_default_state_machine_verifier_detects_extra_transitions() {
        let verifier = DefaultStateMachineVerifier::new();
        let contract = create_test_contract();

        let expected_sequence = vec![create_state_transition(
            "NEW",
            "PROJECT_BOUND",
            "project_command",
        )];
        let actual_sequence = vec![
            create_state_transition("NEW", "PROJECT_BOUND", "project_command"),
            create_state_transition("PROJECT_BOUND", "AGENT_ACTIVE", "task_start"),
        ];

        let result =
            verifier.verify_state_transition(&contract, &expected_sequence, &actual_sequence);

        assert!(result.verdict.is_different());
        assert!(!result.extra_transitions.is_empty());
    }

    #[test]
    fn test_state_machine_verifier_valid_sequence() {
        let verifier = DefaultStateMachineVerifier::new();
        let contract = create_test_contract();

        let expected_sequence = vec![
            create_state_transition("NEW", "PROJECT_BOUND", "project_command"),
            create_state_transition("PROJECT_BOUND", "AGENT_ACTIVE", "task_start"),
        ];
        let actual_sequence = vec![
            create_state_transition("NEW", "PROJECT_BOUND", "project_command"),
            create_state_transition("PROJECT_BOUND", "AGENT_ACTIVE", "task_start"),
        ];

        let result =
            verifier.verify_state_transition(&contract, &expected_sequence, &actual_sequence);

        assert!(result.verdict.is_identical());
        assert_eq!(result.valid_transitions.len(), 2);
        assert!(result.invalid_transitions.is_empty());
        assert!(result.missing_transitions.is_empty());
        assert!(result.extra_transitions.is_empty());
    }

    #[test]
    fn test_state_machine_contract_is_valid_transition() {
        let contract = create_test_contract();
        assert!(contract.is_valid_transition("NEW", "PROJECT_BOUND"));
        assert!(contract.is_valid_transition("PROJECT_BOUND", "AGENT_ACTIVE"));
        assert!(!contract.is_valid_transition("NEW", "DONE"));
        assert!(!contract.is_valid_transition("AGENT_ACTIVE", "NEW"));
    }

    #[test]
    fn test_state_machine_contract_get_allowed_transitions() {
        let contract = create_test_contract();
        let allowed = contract.get_allowed_transitions_from("NEW");
        assert_eq!(allowed.len(), 1);
        assert_eq!(allowed[0].to, "PROJECT_BOUND");

        let allowed_from_agent = contract.get_allowed_transitions_from("AGENT_ACTIVE");
        assert_eq!(allowed_from_agent.len(), 1);
        assert_eq!(allowed_from_agent[0].to, "DONE");

        let no_transitions = contract.get_allowed_transitions_from("DONE");
        assert!(no_transitions.is_empty());
    }

    #[test]
    fn test_state_machine_verifier_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<DefaultStateMachineVerifier>();
    }

    #[test]
    fn state_machine_smoke_tests() {
        let verifier = DefaultStateMachineVerifier::new();
        let contract = create_test_contract();

        let expected_sequence = vec![
            create_state_transition("NEW", "PROJECT_BOUND", "project_command"),
            create_state_transition("PROJECT_BOUND", "AGENT_ACTIVE", "task_start"),
            create_state_transition("AGENT_ACTIVE", "DONE", "task_complete"),
        ];

        let actual_sequence = vec![
            create_state_transition("NEW", "PROJECT_BOUND", "project_command"),
            create_state_transition("PROJECT_BOUND", "AGENT_ACTIVE", "task_start"),
            create_state_transition("AGENT_ACTIVE", "DONE", "task_complete"),
        ];

        let result =
            verifier.verify_state_transition(&contract, &expected_sequence, &actual_sequence);

        assert!(result.verdict.is_identical());
        assert_eq!(result.valid_transitions.len(), 3);
        assert!(result.invalid_transitions.is_empty());
        assert!(result.missing_transitions.is_empty());
        assert!(result.extra_transitions.is_empty());

        fn assert_state_machine_verifier<T: StateMachineVerifier>() {}
        assert_state_machine_verifier::<DefaultStateMachineVerifier>();
    }
}
