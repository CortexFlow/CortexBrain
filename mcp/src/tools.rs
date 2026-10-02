use crate::prometheus::{
    MatchMode, MetricKind, PromClient, TimeSeriesResponse, EVENTS_TOTAL,
};
use anyhow::Result as AnyResult;
use rmcp::handler::server::ServerHandler;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{Implementation, ServerCapabilities, ServerInfo};
use rmcp::{tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

/// Outcome of a tool call. An `Err` is reported to the assistant as a tool
/// error carrying the real cause, instead of the previous `Err(())` that made
/// a broken Prometheus indistinguishable from a workload with no activity.
type ToolResult = Result<String, String>;

/// Render an error with its context chain, e.g.
/// `GET /api/v1/label/... failed: connection refused`.
fn describe(err: anyhow::Error) -> String {
    format!("{err:#}")
}

/// Serialisation of an already built response should not fail; report it
/// plainly instead of pretending the query failed.
fn describe_serde(err: serde_json::Error) -> String {
    err.to_string()
}

/// Request contract shared by every metric tool. `window` is the range vector
/// of the aggregation and defaults to `step`; keeping it separate lets a query
/// return fine-grained samples over a stable rate window.
#[derive(Deserialize, JsonSchema)]
struct Params {
    /// Workload to report on, matched against the identity label
    /// (`container_name`, or `k8s_pod_name` on collectors that have not been
    /// normalised yet). Use `list_workloads` to discover valid values.
    container_name: String,
    /// Range start, RFC 3339 timestamp or unix seconds, e.g.
    /// `2026-09-27T00:00:00Z`.
    start: String,
    /// Range end, same format as `start`.
    end: String,
    /// Query resolution, e.g. `1m`. Also the default aggregation window.
    step: String,
    /// Aggregation window, e.g. `5m`. Defaults to `step`.
    #[serde(default)]
    window: Option<String>,
    /// `substring` (default) matches `container_name=~".*<value>.*"`, so
    /// `otel-agent` also matches `otel-agent-qsm7c`. `exact` requires a full
    /// match.
    #[serde(default)]
    match_mode: Option<String>,
}

impl Params {
    fn window(&self) -> &str {
        self.window.as_deref().unwrap_or(&self.step)
    }
}

#[derive(Deserialize, JsonSchema, Default)]
struct ListParams {
    /// How far back to look for active workloads, e.g. `1h`. Defaults to `1h`.
    #[serde(default)]
    lookback: Option<String>,
}

#[derive(Clone)]
pub struct PrometheusTool {
    prometheus: PromClient,
    tool_router: ToolRouter<PrometheusTool>,
}

impl PrometheusTool {
    pub fn new() -> AnyResult<Self> {
        Ok(Self {
            prometheus: PromClient::new()?,
            tool_router: Self::tool_router(),
        })
    }
}

impl PrometheusTool {
    /// Resolve the identity label, run the aggregation and render it.
    async fn run(&self, params: &Params, metric: &str, kind: MetricKind) -> ToolResult {
        let label = self
            .prometheus
            .identity_label()
            .await
            .map_err(describe)?
            .to_string();
        let mode = MatchMode::parse(params.match_mode.as_deref()).map_err(describe)?;
        let response: TimeSeriesResponse = self
            .prometheus
            .windowed_query(
                metric,
                kind,
                &label,
                mode,
                &params.container_name,
                params.window(),
                &params.start,
                &params.end,
                &params.step,
            )
            .await
            .map_err(describe)?;
        serde_json::to_string_pretty(&response).map_err(describe_serde)
    }
}

#[tool_router]
impl PrometheusTool {
    #[tool(
        name = "get_cpu_bytes",
        description = "CPU bytes allocation per event, averaged over the window"
    )]
    pub async fn get_cpu_bytes(
        &self,
        Parameters(params): Parameters<Params>,
    ) -> ToolResult {
        self.run(
            &params,
            "cortexbrain_cpu_bytes_alloc",
            MetricKind::Gauge,
        )
        .await
    }

    #[tool(
        name = "get_memory_allocated_bytes",
        description = "Bytes requested via mmap syscalls, averaged over the window"
    )]
    pub async fn get_memory_allocated_bytes(
        &self,
        Parameters(params): Parameters<Params>,
    ) -> ToolResult {
        self.run(
            &params,
            "cortexbrain_enter_mem_alloc",
            MetricKind::Gauge,
        )
        .await
    }

    #[tool(
        name = "get_events",
        description = "Total number of eBPF events processed across all perf buffers, as a per-second rate"
    )]
    pub async fn get_events(
        &self,
        Parameters(params): Parameters<Params>,
    ) -> ToolResult {
        self.run(&params, EVENTS_TOTAL, MetricKind::Counter).await
    }

    #[tool(
        name = "get_l4_events",
        description = "Total number of socket state events processed, as a per-second rate"
    )]
    pub async fn get_l4_events(
        &self,
        Parameters(params): Parameters<Params>,
    ) -> ToolResult {
        self.run(
            &params,
            "cortexbrain_socket_events_total",
            MetricKind::Counter,
        )
        .await
    }

    #[tool(
        name = "get_ssl_write_events",
        description = "Total bytes requested by the ssl_write function, averaged over the window"
    )]
    pub async fn get_ssl_write_events(
        &self,
        Parameters(params): Parameters<Params>,
    ) -> ToolResult {
        self.run(&params, "cortexbrain_ssl_write_bytes", MetricKind::Gauge)
            .await
    }

    #[tool(
        name = "get_ssl_read_events",
        description = "Total bytes requested by the ssl_read function, averaged over the window"
    )]
    pub async fn get_ssl_read_events(
        &self,
        Parameters(params): Parameters<Params>,
    ) -> ToolResult {
        self.run(&params, "cortexbrain_ssl_read_bytes", MetricKind::Gauge)
            .await
    }

    #[tool(
        name = "list_workloads",
        description = "List the workloads CortexBrain observes, with the identity label in use and which of them are active. Call this first instead of guessing container names."
    )]
    pub async fn list_workloads(
        &self,
        Parameters(params): Parameters<ListParams>,
    ) -> ToolResult {
        let lookback = params.lookback.as_deref().unwrap_or("1h");
        let listing = self
            .prometheus
            .list_workloads(lookback)
            .await
            .map_err(describe)?;
        serde_json::to_string_pretty(&listing).map_err(describe_serde)
    }
}

#[tool_handler]
impl ServerHandler for PrometheusTool {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            server_info: Implementation {
                name: "cortexflow-mcp".to_string(),
                title: Some("Cortexflow MCP Server".to_string()),
                version: env!("CARGO_PKG_VERSION").to_string(),
                ..Default::default()
            },
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }
}
