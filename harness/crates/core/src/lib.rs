pub mod config;
pub mod error;
pub mod loaders;
pub mod normalizers;
pub mod runners;
pub mod types;
pub mod verifiers;

pub use loaders::{FixtureLoader, TaskLoader, TaskSchemaValidator};
pub use normalizers::{NormalizedOutput, Normalizer, VarianceNormalizer, WhitespaceNormalizer};
pub use runners::{DifferentialResult, DifferentialRunner};
pub use types::entry_mode::EntryMode;
pub use types::fixture::{FixtureFile, FixtureProject, ResetStrategy, Workspace, WorkspacePolicy};
pub use types::report::{Report, TestCase, TestCaseStatus};
pub use types::task::Task;
pub use types::task::TaskCategory;
pub use verifiers::{AssertionResult, DefaultVerifier, VerificationResult, Verifier};
