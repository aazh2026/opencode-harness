pub mod line_endings;
pub mod normalizer;
pub mod paths;
pub mod variance;
pub mod whitelist_validator;
pub mod whitespace;

pub use normalizer::{
    normalize_for_comparison, normalize_output, AppliedRule, LineEndingNormalizer, NoOpNormalizer,
    NormalizedOutput, Normalizer, NormalizerAudit, PathNormalizer, Transformation,
    WhitespaceNormalizer,
};
pub use variance::VarianceNormalizer;
pub use whitelist_validator::{ValidationError, ValidationResult, WhitelistValidator};
