use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::OnceCell;

/// Endpoint used when `PROMETHEUS_URL` is not set.
pub const DEFAULT_PROMETHEUS_SERVER: &str = "http://localhost:9090";

/// Labels able to identify the workload a series belongs to, most specific
/// first. Docker hosts expose `container_name`, Kubernetes exposes
/// `k8s_pod_name`; the `transform/identity` processor in the otel-collector
/// configs backfills the former from the latter, so on a current deployment
/// the first entry always wins. The remaining entries keep the server usable
/// against collectors that have not been rolled out yet.
pub const IDENTITY_LABELS: [&str; 3] = ["container_name", "k8s_pod_name", "k8s_namespace_name"];

/// Counter used for discovery queries on the busiest signal.
pub const EVENTS_TOTAL: &str = "cortexbrain_events_total";

/// Prometheus histogram of the OTel instrument kind, which decides the
/// aggregation applied to a metric.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetricKind {
    /// Monotonic counter, aggregated with `rate()`.
    Counter,
    /// Instantaneous value, aggregated with `avg_over_time()`. `rate()` is not
    /// valid here: it is defined for counters and yields meaningless output
    /// for gauges.
    Gauge,
}

/// How the workload argument is compared against the identity label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchMode {
    /// `label=~".*workload.*"`, so `otel-agent` also matches
    /// `otel-agent-qsm7c`.
    Substring,
    /// `label="workload"`, an exact match.
    Exact,
}

impl MatchMode {
    pub fn parse(value: Option<&str>) -> Result<Self> {
        match value.map(str::trim).filter(|v| !v.is_empty()) {
            None | Some("substring") => Ok(Self::Substring),
            Some("exact") => Ok(Self::Exact),
            Some(other) => bail!("invalid match_mode `{other}`, expected `substring` or `exact`"),
        }
    }

    fn selector(self, label: &str, workload: &str) -> String {
        match self {
            Self::Substring => format!("{label}=~\".*{}.*\"", escape_regex(workload)),
            Self::Exact => format!("{label}=\"{}\"", escape_string(workload)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub timestamp: i64,
    pub value: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeSeries {
    /// The label set of the series, including the identity label, so a
    /// response is self-describing.
    pub labels: Map<String, Value>,
    pub values: Vec<TimeSeriesPoint>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeSeriesResponse {
    pub metric_name: String,
    /// Label the workload was matched on.
    pub identity_label: String,
    pub workload: String,
    pub match_mode: String,
    /// Range vector used for the aggregation, e.g. `5m`.
    pub window: String,
    pub start: String,
    pub end: String,
    pub step: String,
    pub series: Vec<TimeSeries>,
}

/// Workload names discoverable in Prometheus, keyed by label.
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkloadListing {
    /// Label the metric tools will use for the current deployment.
    pub identity_label: String,
    pub labels: BTreeMap<String, Vec<String>>,
    /// Workloads with at least one event inside the discovery window.
    pub active: Vec<String>,
}

#[derive(Clone)]
pub struct PromClient {
    baseurl: String,
    client: Client,
    /// Resolved once per process: the label set does not change while the
    /// server runs, and a failed query would otherwise re-probe Prometheus on
    /// every call.
    identity_label: Arc<OnceCell<String>>,
}

impl PromClient {
    pub fn new() -> Result<Self> {
        let baseurl = std::env::var("PROMETHEUS_URL")
            .unwrap_or_else(|_| DEFAULT_PROMETHEUS_SERVER.to_string());

        Ok(PromClient {
            client: Client::new(),
            baseurl,
            identity_label: Arc::new(OnceCell::new()),
        })
    }

    /// Label used to select workloads, probed once and cached.
    pub async fn identity_label(&self) -> Result<&str> {
        self.identity_label
            .get_or_try_init(|| async {
                for candidate in IDENTITY_LABELS {
                    if !self.label_values(candidate).await?.is_empty() {
                        return Ok(candidate.to_string());
                    }
                }
                bail!(
                    "no cortexbrain series found in {} for any of {:?}; the metrics \
                     service may not be running, or the otel-collector may be \
                     missing the transform/identity processor",
                    self.baseurl,
                    IDENTITY_LABELS
                )
            })
            .await
            .map(String::as_str)
    }

    /// Values currently present for a label.
    pub async fn label_values(&self, label: &str) -> Result<Vec<String>> {
        let response = self
            .client
            .get(format!("{}/api/v1/label/{label}/values", self.baseurl))
            .send()
            .await
            .with_context(|| format!("GET /api/v1/label/{label}/values failed"))?
            .error_for_status()
            .with_context(|| format!("GET /api/v1/label/{label}/values returned an error status"))?;

        let body: Value = response
            .json()
            .await
            .with_context(|| format!("GET /api/v1/label/{label}/values returned invalid JSON"))?;

        check_status(&body).with_context(|| format!("listing values of label `{label}`"))?;

        Ok(body
            .get("data")
            .and_then(Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default())
    }

    /// Instant query built from a PromQL expression.
    pub async fn query(&self, promql_query: &str) -> Result<Value> {
        let response = self
            .client
            .get(format!("{}/api/v1/query", self.baseurl))
            .query(&[("query", promql_query)])
            .send()
            .await
            .with_context(|| format!("GET /api/v1/query failed for `{promql_query}`"))?
            .error_for_status()
            .with_context(|| format!("query `{promql_query}` returned an error status"))?;

        let body: Value = response
            .json()
            .await
            .with_context(|| format!("query `{promql_query}` returned invalid JSON"))?;

        check_status(&body).with_context(|| format!("running query `{promql_query}`"))?;

        Ok(body)
    }

    /// Range query built from a PromQL expression.
    pub async fn query_range(
        &self,
        promql_query: &str,
        start: &str,
        end: &str,
        step: &str,
    ) -> Result<Value> {
        let response = self
            .client
            .get(format!("{}/api/v1/query_range", self.baseurl))
            .query(&[
                ("query", promql_query),
                ("start", start),
                ("end", end),
                ("step", step),
            ])
            .send()
            .await
            .with_context(|| format!("GET /api/v1/query_range failed for `{promql_query}`"))?
            .error_for_status()
            .with_context(|| format!("range query `{promql_query}` returned an error status"))?;

        let body: Value = response
            .json()
            .await
            .with_context(|| format!("range query `{promql_query}` returned invalid JSON"))?;

        check_status(&body).with_context(|| format!("running range query `{promql_query}`"))?;

        Ok(body)
    }

    /// Run a windowed aggregation over one metric and normalise the matrix
    /// into a typed response.
    #[allow(clippy::too_many_arguments)]
    pub async fn windowed_query(
        &self,
        metric: &str,
        kind: MetricKind,
        identity_label: &str,
        mode: MatchMode,
        workload: &str,
        window: &str,
        start: &str,
        end: &str,
        step: &str,
    ) -> Result<TimeSeriesResponse> {
        let promql = build_promql(metric, kind, identity_label, mode, workload, window);
        let raw = self.query_range(&promql, start, end, step).await?;
        let series = parse_series(&raw, identity_label);

        Ok(TimeSeriesResponse {
            metric_name: metric.to_string(),
            identity_label: identity_label.to_string(),
            workload: workload.to_string(),
            match_mode: match mode {
                MatchMode::Substring => "substring",
                MatchMode::Exact => "exact",
            }
            .to_string(),
            window: window.to_string(),
            start: start.to_string(),
            end: end.to_string(),
            step: step.to_string(),
            series,
        })
    }

    /// Workloads visible in Prometheus, both as raw label values and as those
    /// that reported events inside the discovery window.
    pub async fn list_workloads(&self, lookback: &str) -> Result<WorkloadListing> {
        let identity_label = self.identity_label().await?.to_string();

        let mut labels = BTreeMap::new();
        for label in IDENTITY_LABELS {
            labels.insert(label.to_string(), self.label_values(label).await?);
        }

        let active_promql = format!("group by({identity_label}) ({EVENTS_TOTAL}[{lookback}])");
        let mut active = Vec::new();
        if let Ok(raw) = self.query(&active_promql).await {
            for series in result_vector(&raw) {
                if let Some(name) = series.get(&identity_label).and_then(Value::as_str) {
                    active.push(name.to_string());
                }
            }
        }
        active.sort();
        active.dedup();

        Ok(WorkloadListing {
            identity_label,
            labels,
            active,
        })
    }
}

/// Build the aggregation for one metric. Counters use `rate()`, gauges use
/// `avg_over_time()`, and both are summed by the identity label so a
/// multi-match regex collapses every matching workload into one series.
pub fn build_promql(
    metric: &str,
    kind: MetricKind,
    identity_label: &str,
    mode: MatchMode,
    workload: &str,
    window: &str,
) -> String {
    let selector = mode.selector(identity_label, workload);
    let inner = match kind {
        MetricKind::Counter => format!("rate({metric}{{{selector}}}[{window}])"),
        MetricKind::Gauge => format!("avg_over_time({metric}{{{selector}}}[{window}])"),
    };
    format!("sum by({identity_label}) ({inner})")
}

/// Escape a value interpolated into a PromQL string or a regex literal.
fn escape_regex(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        if r"\.+*?()|[]{}^$-".contains(ch) {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped
}

/// Escape a value interpolated into a double-quoted PromQL string literal.
fn escape_string(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        if ch == '\\' || ch == '"' || ch == '\n' {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped
}

/// Fail when Prometheus reports a query error, which is otherwise easy to
/// mistake for an empty result set.
fn check_status(body: &Value) -> Result<()> {
    match body.get("status").and_then(Value::as_str) {
        Some("success") => Ok(()),
        _ => {
            let kind = body
                .get("errorType")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let message = body
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("no error message");
            bail!("prometheus returned `{kind}`: {message}")
        }
    }
}

/// Every series of a matrix, or every point of the first series of an instant
/// vector. Unlike a first-series-only parse, a regex matching several
/// workloads keeps all of them.
fn parse_series(raw: &Value, identity_label: &str) -> Vec<TimeSeries> {
    result_vector(raw)
        .iter()
        .map(|series| {
            let labels = series
                .get("metric")
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default();

            // A matrix carries `values`, an instant vector a single `value`.
            let points: Vec<&Value> = series
                .get("values")
                .and_then(Value::as_array)
                .map(|values| values.iter().collect())
                .unwrap_or_else(|| series.get("value").into_iter().collect());

            let values = points.into_iter().filter_map(parse_point).collect();

            TimeSeries {
                labels,
                values,
            }
        })
        .filter(|series| {
            // Drop series with no usable samples, and require the identity
            // label so an unrelated series cannot be reported as a workload.
            !series.values.is_empty()
                && series
                    .labels
                    .get(identity_label)
                    .and_then(Value::as_str)
                    .is_some()
        })
        .collect()
}

fn result_vector(raw: &Value) -> Vec<&Value> {
    raw.get("data")
        .and_then(|v| v.get("result"))
        .and_then(Value::as_array)
        .map(|result| result.iter().collect())
        .unwrap_or_default()
}

fn parse_point(point: &Value) -> Option<TimeSeriesPoint> {
    let pair = point.as_array()?;
    if pair.len() < 2 {
        return None;
    }
    let timestamp = match &pair[0] {
        Value::Number(n) => n.as_i64()?,
        Value::String(s) => s.parse().ok()?,
        _ => return None,
    };
    let value = match &pair[1] {
        Value::Number(n) => n.as_f64()?,
        Value::String(s) => s.parse().ok()?,
        _ => return None,
    };
    Some(TimeSeriesPoint { timestamp, value })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn counter_queries_use_rate() {
        assert_eq!(
            build_promql(
                EVENTS_TOTAL,
                MetricKind::Counter,
                "container_name",
                MatchMode::Substring,
                "grafana",
                "5m",
            ),
            r#"sum by(container_name) (rate(cortexbrain_events_total{container_name=~".*grafana.*"}[5m]))"#
        );
    }

    #[test]
    fn gauge_queries_do_not_use_rate() {
        // rate() is undefined for gauges, this is the regression guard.
        let query = build_promql(
            "cortexbrain_enter_mem_alloc",
            MetricKind::Gauge,
            "k8s_pod_name",
            MatchMode::Substring,
            "grafana",
            "5m",
        );
        assert!(!query.contains("rate("), "{query}");
        assert_eq!(
            query,
            r#"sum by(k8s_pod_name) (avg_over_time(cortexbrain_enter_mem_alloc{k8s_pod_name=~".*grafana.*"}[5m]))"#
        );
    }

    #[test]
    fn exact_match_is_not_a_regex() {
        assert_eq!(
            build_promql(
                EVENTS_TOTAL,
                MetricKind::Counter,
                "container_name",
                MatchMode::Exact,
                "grafana",
                "1m",
            ),
            r#"sum by(container_name) (rate(cortexbrain_events_total{container_name="grafana"}[1m]))"#
        );
    }

    #[test]
    fn match_mode_parsing_defaults_to_substring() {
        assert_eq!(MatchMode::parse(None).unwrap(), MatchMode::Substring);
        assert_eq!(MatchMode::parse(Some("")).unwrap(), MatchMode::Substring);
        assert_eq!(
            MatchMode::parse(Some("exact")).unwrap(),
            MatchMode::Exact
        );
        assert!(MatchMode::parse(Some("fuzzy")).is_err());
    }

    #[test]
    fn workload_is_escaped() {
        // A workload name must not be able to break out of the regex literal.
        let query = build_promql(
            EVENTS_TOTAL,
            MetricKind::Counter,
            "container_name",
            MatchMode::Substring,
            "a.b",
            "1m",
        );
        assert!(query.contains(r#".*a\.b.*"#), "{query}");
    }

    #[test]
    fn every_series_is_kept() {
        let raw = json!({
            "status": "success",
            "data": { "resultType": "matrix", "result": [
                { "metric": { "container_name": "grafana-1" },
                  "values": [[1, "0.5"], [2, "0.7"]] },
                { "metric": { "container_name": "grafana-2" },
                  "values": [[1, "0.1"]] },
            ]}
        });

        let series = parse_series(&raw, "container_name");
        assert_eq!(series.len(), 2);
        assert_eq!(series[0].labels["container_name"], "grafana-1");
        assert_eq!(series[0].values.len(), 2);
        assert_eq!(series[0].values[0].value, 0.5);
        assert_eq!(series[1].values[0].timestamp, 1);
    }

    #[test]
    fn series_without_samples_or_identity_are_dropped() {
        let raw = json!({
            "status": "success",
            "data": { "resultType": "matrix", "result": [
                { "metric": { "container_name": "grafana" }, "values": [] },
                { "metric": { "command": "node" }, "values": [[1, "1"]] },
                { "metric": { "container_name": "otel" }, "values": [[1, "2"]] },
            ]}
        });

        let series = parse_series(&raw, "container_name");
        assert_eq!(series.len(), 1);
        assert_eq!(series[0].labels["container_name"], "otel");
    }

    #[test]
    fn instant_vector_is_parsed() {
        let raw = json!({
            "status": "success",
            "data": { "resultType": "vector", "result": [
                { "metric": { "container_name": "grafana" }, "value": [7, "3.5"] },
            ]}
        });

        let series = parse_series(&raw, "container_name");
        assert_eq!(series.len(), 1);
        assert_eq!(series[0].values[0].timestamp, 7);
        assert_eq!(series[0].values[0].value, 3.5);
    }

    #[test]
    fn non_numeric_samples_are_skipped() {
        let raw = json!({
            "status": "success",
            "data": { "resultType": "matrix", "result": [
                { "metric": { "container_name": "grafana" },
                  "values": [[1, "NaN"], [2, "1.25"]] },
            ]}
        });

        let series = parse_series(&raw, "container_name");
        assert_eq!(series[0].values.len(), 1);
        assert_eq!(series[0].values[0].value, 1.25);
    }

    #[test]
    fn prometheus_errors_are_surfaced() {
        let body = json!({ "status": "error", "errorType": "bad_data", "error": "parse error" });
        let err = check_status(&body).unwrap_err().to_string();
        assert!(err.contains("bad_data"), "{err}");
        assert!(err.contains("parse error"), "{err}");
    }
}
