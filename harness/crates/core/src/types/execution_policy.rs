#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExecutionPolicy {
    ManualCheck,
    Blocked,
    Skip,
}
