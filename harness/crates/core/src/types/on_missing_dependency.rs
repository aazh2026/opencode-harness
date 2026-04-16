#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OnMissingDependency {
    Fail,
    Skip,
    Warn,
    Blocked,
}
