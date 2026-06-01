//! docs
//! This module configures and bootstraps the OpenTelemetry SDK (OTel SDK)
//! within the `metrics` binary. Its goal is to expose a [`Meter`] --- the
//! primary entry-point for creating counters, gauges and histograms ---
//! backed by an **OTLP/gRPC** metric exporter.
//!
//! # Relationship to the rest of the crate
//!
//! `otel_init::init_opentelemetry()` is invoked **once** in [`main`], before
//! any eBPF program is loaded. The returned [`Meter`] is then passed through
//! the call chain into [`event_listener`](crate::helpers::event_listener)
//! where it is used by the async tasks that read eBPF perf-buffers.  See
//! [`crate::helpers`] for the consumption side.
//!
//! When the application exits (either because `Ctrl-C` was received or because
//! an error bubbled up), [`shutdown_opentelemetry`] is called.  This flushes
//! every remaining aggregated metric to the OTLP collector before the process
//! terminates.
//!

use opentelemetry::global;
use opentelemetry::metrics::{Meter, MeterProvider};
use opentelemetry_otlp::{MetricExporter, WithExportConfig};
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use std::env;
use std::sync::OnceLock;
use std::time::Duration;

/// Environment variable that holds the OTLP collector endpoint.
///
/// Expected format: `"http://collector:4317"` (gRPC transport).
///
pub const OTEL_EXPORTER_OTLP_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_ENDPOINT";

/// Default OTLP endpoint used when [`OTEL_EXPORTER_OTLP_ENDPOINT`] is not
/// present in the environment.
///
/// Points to a locally-running OpenTelemetry Collector on the standard
/// **gRPC** port `4317`.  Note that OTLP over HTTP typically uses `4318` ---
/// make sure your Collector is actually listening for **gRPC** traffic on the
/// port you configure.
pub const DEFAULT_OTLP_ENDPOINT: &str = "http://localhost:4317";

/// Singleton that owns the concrete `SdkMeterProvider` instance.
/// OnceLock guarantees single initialisation, we avoid accidentally creating two providers (and
/// two background export tasks) if `init_opentelemetry()` were ever called
/// twice.
///
/// # Thread safety
///
/// `OnceLock<T>` is `Sync`, so the static can be read safely from any thread
/// or Tokio task once populated.
static METER_PROVIDER: OnceLock<SdkMeterProvider> = OnceLock::new();
/// docs:
/// Initialise the OpenTelemetry SDK, wire up the OTLP/gRPC exporter, and
/// return a [`Meter`] ready for instrumenting the `metrics` crate.
///
/// 1. Read the endpoint from [`OTEL_EXPORTER_OTLP_ENDPOINT`] with the
///    hard-coded default [`DEFAULT_OTLP_ENDPOINT`].
/// 2. Build a `MetricExporter` using the Tonic / gRPC transport:
///    - with_tonic()` enables the Tonic-based gRPC client.
///    - `with_endpoint()` sets the target Collector URL.
///    - `with_timeout(Duration::from_secs(10))` caps each export RPC to 10
///      seconds; if the Collector is unreachable the RPC aborts instead of
///      hanging indefinitely.
/// 3. Wrap the exporter in a `PeriodicReader`.  The reader collects
///    aggregated metrics from every instrument every 5 seconds and hands
///    them to the exporter.  This is the "push" model --- metrics leave the
///    process automatically without an external scraper.
/// 4. Construct an `SdkMeterProvider` and register it as the global
///    meter provider (`global::set_meter_provider`).  The global handle is
///    needed for instrumenting code spawned in other Tokio tasks (see
///    [`helpers::event_listener`](crate::helpers::event_listener)).
/// 5. Keep a clone of the concrete provider in `METER_PROVIDER` so that
///    [`shutdown_opentelemetry`] can later call `SdkMeterProvider::shutdown()`.
/// 6. Create a `Meter named `"cortexbrain-metrics"` and return it.
///
/// Potential causes of errors:
///
/// * An invalid endpoint URL (malformed string).
/// * Network-level failure during exporter construction.
/// * The provider already having been initialised
///
pub fn init_opentelemetry() -> Result<Meter, anyhow::Error> {
    let endpoint =
        env::var(OTEL_EXPORTER_OTLP_ENDPOINT).unwrap_or_else(|_| DEFAULT_OTLP_ENDPOINT.to_string());

    let exporter = MetricExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .with_timeout(Duration::from_secs(10))
        .build()?;

    let reader = PeriodicReader::builder(exporter)
        .with_interval(Duration::from_secs(5))
        .build();

    let provider = SdkMeterProvider::builder().with_reader(reader).build();

    // Make the provider globally discoverable.  This clone is cheap because
    // SdkMeterProvider is an Arc-backed handle.
    global::set_meter_provider(provider.clone());

    // Stash the concrete handle so shutdown_opentelemetry can flush.
    METER_PROVIDER
        .set(provider.clone())
        .map_err(|_| anyhow::anyhow!("OpenTelemetry meter provider already initialised"))?;

    let meter = provider.meter("cortexbrain-metrics");
    Ok(meter)
}
/// docs:
/// Flush every buffered metric to the OTLP collector and shut down the SDK.
pub fn shutdown_opentelemetry() {
    if let Some(provider) = METER_PROVIDER.get()
        && let Err(e) = provider.shutdown()
    {
        tracing::error!("Failed to shut down OpenTelemetry meter provider: {:?}", e);
    }
}
