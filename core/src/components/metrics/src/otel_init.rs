//! Thin wrapper around the shared OpenTelemetry metrics initialisation in
//! `cortexbrain-common`. The concrete implementation lives in
//! [`cortexbrain_common::otel_init`] so that every CortexBrain service uses the
//! same exporter configuration.

use opentelemetry::metrics::Meter;

/// Initialise the OpenTelemetry metrics pipeline for the metrics service.
pub fn init_opentelemetry() -> Result<Meter, anyhow::Error> {
    cortexbrain_common::otel_init::init_opentelemetry("cortexbrain-metrics")
}

/// Flush every buffered metric to the OTLP collector and shut down the SDK.
pub fn shutdown_opentelemetry() {
    cortexbrain_common::otel_init::shutdown_opentelemetry()
}
