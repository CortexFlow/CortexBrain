//! Shared OpenTelemetry metrics initialisation.
//!
//! This module configures and bootstraps the OpenTelemetry SDK (OTel SDK)
//! for every CortexBrain service that exports metrics.  Its goal is to expose
//! a [`Meter`] --- the primary entry-point for creating counters, gauges and
//! histograms --- backed by an OTLP metric exporter (gRPC or HTTP, depending
//! on configuration).
//!
//! `init_opentelemetry()` is invoked **once** per process, before any eBPF
//! program is loaded.  The returned [`Meter`] is then passed to
//! [`crate::otel_metrics::Metrics::new`] and shared between the asynchronous
//! tasks that read eBPF perf-buffers.
//!
//! When the application exits, [`shutdown_opentelemetry`] flushes every
//! remaining aggregated metric to the OTLP collector before the process
//! terminates.

use opentelemetry::global;
use opentelemetry::metrics::{Meter, MeterProvider};
use opentelemetry_otlp::{MetricExporter, WithExportConfig};
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use std::env;
use std::sync::OnceLock;
use std::time::Duration;

/// Environment variable that holds the OTLP collector endpoint.
///
/// Expected format: `"http://collector:4317"` for gRPC or `"http://collector:4318"` for HTTP.
pub const OTEL_EXPORTER_OTLP_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_ENDPOINT";
pub const OTEL_EXPORTER_OTLP_PROTOCOL: &str = "OTEL_EXPORTER_OTLP_PROTOCOL";

/// Default OTLP endpoints used when [`OTEL_EXPORTER_OTLP_ENDPOINT`] is not
/// present in the environment.
pub const DEFAULT_OTLP_GRPC_ENDPOINT: &str = "http://localhost:4317";
pub const DEFAULT_OTLP_HTTP_ENDPOINT: &str = "http://localhost:4318";

fn default_otlp_endpoint_from_protocol() -> &'static str {
    match env::var(OTEL_EXPORTER_OTLP_PROTOCOL)
        .ok()
        .map(|value| value.to_ascii_lowercase())
        .as_deref()
    {
        Some("http/protobuf") | Some("http/json") => DEFAULT_OTLP_HTTP_ENDPOINT,
        _ => DEFAULT_OTLP_GRPC_ENDPOINT,
    }
}

fn resolved_otlp_protocol() -> String {
    env::var(OTEL_EXPORTER_OTLP_PROTOCOL)
        .ok()
        .map(|value| value.to_ascii_lowercase())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "grpc".to_string())
}

/// Singleton that owns the concrete `SdkMeterProvider` instance.
///
/// `OnceLock` guarantees single initialisation, avoiding accidentally creating
/// two providers (and two background export tasks) if `init_opentelemetry()`
/// were ever called twice.
static METER_PROVIDER: OnceLock<SdkMeterProvider> = OnceLock::new();

/// Initialise the OpenTelemetry SDK, wire up the OTLP exporter, and return a
/// [`Meter`] registered under `meter_name`.
///
/// 1. Read the endpoint from [`OTEL_EXPORTER_OTLP_ENDPOINT`] with the
///    protocol-aware defaults [`DEFAULT_OTLP_GRPC_ENDPOINT`] and
///    [`DEFAULT_OTLP_HTTP_ENDPOINT`].
/// 2. Build a `MetricExporter` using the transport resolved by
///    `OTEL_EXPORTER_OTLP_PROTOCOL`.
/// 3. Wrap the exporter in a `PeriodicReader` that collects aggregated metrics
///    every 5 seconds and pushes them to the collector.
/// 4. Construct an `SdkMeterProvider` and register it as the global meter
///    provider.
/// 5. Keep a clone of the concrete provider in `METER_PROVIDER` so that
///    [`shutdown_opentelemetry`] can later call `SdkMeterProvider::shutdown()`.
/// 6. Create and return the `Meter` named `meter_name`.
pub fn init_opentelemetry(meter_name: &'static str) -> Result<Meter, anyhow::Error> {
    let endpoint = env::var(OTEL_EXPORTER_OTLP_ENDPOINT)
        .unwrap_or_else(|_| default_otlp_endpoint_from_protocol().to_string());
    let protocol = resolved_otlp_protocol();

    let exporter = match protocol.as_str() {
        "http/protobuf" | "http/json" => MetricExporter::builder()
            .with_http()
            .with_endpoint(endpoint)
            .with_timeout(Duration::from_secs(10))
            .build()?,
        _ => MetricExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint)
            .with_timeout(Duration::from_secs(10))
            .build()?,
    };

    let reader = PeriodicReader::builder(exporter)
        .with_interval(Duration::from_secs(5))
        .build();

    let provider = SdkMeterProvider::builder().with_reader(reader).build();

    // Make the provider globally discoverable. This clone is cheap because
    // SdkMeterProvider is an Arc-backed handle.
    global::set_meter_provider(provider.clone());

    // Stash the concrete handle so shutdown_opentelemetry can flush.
    METER_PROVIDER
        .set(provider.clone())
        .map_err(|_| anyhow::anyhow!("OpenTelemetry meter provider already initialised"))?;

    let meter = provider.meter(meter_name);
    Ok(meter)
}

/// Flush every buffered metric to the OTLP collector and shut down the SDK.
pub fn shutdown_opentelemetry() {
    if let Some(provider) = METER_PROVIDER.get()
        && let Err(e) = provider.shutdown()
    {
        tracing::error!("Failed to shut down OpenTelemetry meter provider: {:?}", e);
    }
}
