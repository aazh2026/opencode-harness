pub mod line_endings;
pub mod normalizer;
pub mod paths;
pub mod variance;
pub mod whitespace;

pub use normalizer::{
    normalize_for_comparison, normalize_output, NormalizedOutput, Normalizer, WhitespaceNormalizer,
};
pub use variance::VarianceNormalizer;
