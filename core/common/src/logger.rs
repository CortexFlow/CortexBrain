use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};

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

use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, WithExportConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::logs::SdkLoggerProvider;

pub fn otlp_logger_init(service_name: String) -> SdkLoggerProvider {
    //exporter and provider initialization
    let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    let exporter = LogExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_endpoint)
        .build()
        .expect("Failed to create OTLP exporter");

    //needs a service name
    let provider = SdkLoggerProvider::builder()
        .with_resource(Resource::builder().with_service_name(service_name).build())
        .with_batch_exporter(exporter)
        .build();

    //maybe we will need some filter later
    //init otel_filter and layer
    let otel_layer = OpenTelemetryTracingBridge::new(&provider);

    // init fmt filter and layer
    let fmt_filter = EnvFilter::new("info").add_directive("opentelemetry=debug".parse().unwrap());
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_thread_names(true)
        .with_line_number(false)
        .with_target(false)
        .pretty()
        .with_filter(fmt_filter);

    //init tracing subscriber with otel layer
    tracing_subscriber::registry()
        .with(otel_layer)
        .with(fmt_layer)
        .init();

    provider
}
