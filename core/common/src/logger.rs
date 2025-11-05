use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

/// Initialize the default logger configuration used across CortexBrain components.
///
/// This configures tracing with:
/// - INFO level logging
/// - Pretty formatting
/// - No target, file, or line number information
/// - Environment-based filtering
pub fn init_default_logger() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_level(true)
        .with_span_events(FmtSpan::NONE)
        .with_file(false)
        .pretty()
        .with_env_filter(EnvFilter::new("info"))
        .with_line_number(false)
        .init();
}

/// Initialize logger without timestamp information.
/// Used by components that don't need timestamp logging.
pub fn init_logger_without_time() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_level(true)
        .with_span_events(FmtSpan::NONE)
        .with_file(false)
        .without_time()
        .pretty()
        .with_env_filter(EnvFilter::new("info"))
        .with_line_number(false)
        .init();
}
