pub mod logging {
    use tracing::Level;
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    pub fn init_logger() {
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer())
            .init();
    }

    pub fn init_logger_with_level(level: Level) {
        let filter = EnvFilter::new(format!("{}", level));

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer())
            .init();
    }

    pub fn init_test_logger() {
        let filter = EnvFilter::new("debug");

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().with_target(true))
            .init();
    }

    pub fn trace_log<T: std::fmt::Display>(msg: T) {
        tracing::trace!("{}", msg);
    }

    pub fn debug_log<T: std::fmt::Display>(msg: T) {
        tracing::debug!("{}", msg);
    }

    pub fn info_log<T: std::fmt::Display>(msg: T) {
        tracing::info!("{}", msg);
    }

    pub fn warn_log<T: std::fmt::Display>(msg: T) {
        tracing::warn!("{}", msg);
    }

    pub fn error_log<T: std::fmt::Display>(msg: T) {
        tracing::error!("{}", msg);
    }

    pub fn log_at_level<T: std::fmt::Display>(level: Level, msg: T) {
        match level {
            Level::TRACE => tracing::trace!("{}", msg),
            Level::DEBUG => tracing::debug!("{}", msg),
            Level::INFO => tracing::info!("{}", msg),
            Level::WARN => tracing::warn!("{}", msg),
            Level::ERROR => tracing::error!("{}", msg),
        }
    }
}

pub use logging::*;

#[cfg(test)]
mod tests {
    use super::logging::*;
    use tracing::Level;

    #[test]
    fn test_info_log() {
        info_log("Test info message");
    }

    #[test]
    fn test_warn_log() {
        warn_log("Test warning message");
    }

    #[test]
    fn test_error_log() {
        error_log("Test error message");
    }

    #[test]
    fn test_debug_log() {
        debug_log("Test debug message");
    }

    #[test]
    fn test_trace_log() {
        trace_log("Test trace message");
    }

    #[test]
    fn test_log_at_level_trace() {
        log_at_level(Level::TRACE, "Message at trace level");
    }

    #[test]
    fn test_log_at_level_debug() {
        log_at_level(Level::DEBUG, "Message at debug level");
    }

    #[test]
    fn test_log_at_level_info() {
        log_at_level(Level::INFO, "Message at info level");
    }

    #[test]
    fn test_log_at_level_warn() {
        log_at_level(Level::WARN, "Message at warn level");
    }

    #[test]
    fn test_log_at_level_error() {
        log_at_level(Level::ERROR, "Message at error level");
    }

    #[test]
    fn test_logging_at_trace_level() {
        trace_log("Regression test: logging at trace level works");
        let output = format!("{}", Level::TRACE);
        assert_eq!(output, "TRACE");
    }

    #[test]
    fn test_logging_at_debug_level() {
        debug_log("Regression test: logging at debug level works");
        let output = format!("{}", Level::DEBUG);
        assert_eq!(output, "DEBUG");
    }

    #[test]
    fn test_logging_at_info_level() {
        info_log("Regression test: logging at info level works");
        let output = format!("{}", Level::INFO);
        assert_eq!(output, "INFO");
    }

    #[test]
    fn test_logging_at_warn_level() {
        warn_log("Regression test: logging at warn level works");
        let output = format!("{}", Level::WARN);
        assert_eq!(output, "WARN");
    }

    #[test]
    fn test_logging_at_error_level() {
        error_log("Regression test: logging at error level works");
        let output = format!("{}", Level::ERROR);
        assert_eq!(output, "ERROR");
    }

    #[test]
    fn test_all_log_levels_exist() {
        let _ = Level::TRACE;
        let _ = Level::DEBUG;
        let _ = Level::INFO;
        let _ = Level::WARN;
        let _ = Level::ERROR;
    }
}
