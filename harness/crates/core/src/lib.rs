pub mod config;
pub mod error;
pub mod loaders;
pub mod logging;
pub mod normalizers;
pub mod runners;
pub mod types;
pub mod verifiers;

pub use loaders::{FixtureLoader, TaskLoader, TaskSchemaValidator};
pub use normalizers::{NormalizedOutput, Normalizer, VarianceNormalizer, WhitespaceNormalizer};
pub use runners::{
    BinaryResolver, DifferentialResult, DifferentialRunner, LegacyRunner, LegacyRunnerResult,
    RustRunner, RustRunnerResult,
};
pub use types::artifact::{Artifact, ArtifactKind};
pub use types::entry_mode::EntryMode;
pub use types::fixture::{FixtureFile, FixtureProject, ResetStrategy, Workspace, WorkspacePolicy};
pub use types::parity_verdict::{DiffCategory, ParityVerdict};
pub use types::report::{Report, TestCase, TestCaseStatus};
pub use types::task::Task;
pub use types::task::TaskCategory;
pub use verifiers::{
    AssertionResult, DefaultStateMachineVerifier, DefaultVerifier, StateMachineContract,
    StateMachineVerificationResult, StateMachineVerifier, StateTransition, VerificationResult,
    Verifier,
};
pub use verifiers::{
    DefaultSideEffectVerifier, ExpectedSideEffects, SideEffectVerificationResult,
    SideEffectVerifier,
};
