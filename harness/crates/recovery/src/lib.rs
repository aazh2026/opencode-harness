pub mod integration_tests;
pub mod error_handling_tests;

#[cfg(test)]
mod tests {
    #[test]
    fn smoke_recovery_001() {
        super::integration_tests::session_reconnect_after_connection_interruption();
    }
}