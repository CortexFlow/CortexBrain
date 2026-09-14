use anyhow::{Ok, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone)]
pub struct PromClient {
    baseurl: String,
    client: Client,
}

// >>> instruction-file update: typed time-series response model
#[derive(Debug, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub timestamp: i64,
    pub value: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeSeriesResponse {
    pub metric_name: String,
    pub container_name: String,
    pub start: String,
    pub end: String,
    pub step: String,
    pub values: Vec<TimeSeriesPoint>,
}

// >>> instruction-file update: environment driven endpoint fallback
const DEFAULT_PROMETHEUS_SERVER: &str = "http://localhost:9090";

impl PromClient {
    pub fn new() -> Result<Self> {
        // >>> instruction-file update: remove hardcoded endpoint by reading PROMETHEUS_URL
        let baseurl = std::env::var("PROMETHEUS_URL")
            .unwrap_or_else(|_| DEFAULT_PROMETHEUS_SERVER.to_string());

        Ok(PromClient {
            client: Client::new(),
            baseurl,
        })
    }

    /// creates an instant query using PromQL language
    pub async fn query(&self, promql_query: &str) -> Result<Value> {
        let response = self
            .client
            .get(format!("{}/api/v1/query", self.baseurl))
            .query(&[("query", promql_query)])
            .send()
            .await?;

        Ok(response.json().await?)
    }

    // >>> instruction-file update: new query_range path to achieve time-series data
    /// creates a time-series query using the Prometheus query_range endpoint
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
            .query(&[("query", promql_query), ("start", start), ("end", end), ("step", step)])
            .send()
            .await?;

        Ok(response.json().await?)
    }

    // >>> instruction-file update: build a typed response from the range query result
    fn build_time_series(
        &self,
        metric_name: &str,
        container_name: &str,
        start: &str,
        end: &str,
        step: &str,
        raw: &Value,
    ) -> Result<String> {
        let mut points = Vec::new();

        if let Some(result) = raw
            .get("data")
            .and_then(|v| v.get("result"))
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
        {
            if let Some(values) = result.get("values").and_then(|v| v.as_array()) {
                for item in values {
                    if let Some(pair) = item.as_array() {
                        if pair.len() >= 2 {
                            let ts = pair[0].as_i64().unwrap_or_default();
                            let val = pair[1].as_str().unwrap_or("0");
                            let value = val.parse::<f64>().unwrap_or_default();
                            points.push(TimeSeriesPoint { timestamp: ts, value });
                        }
                    }
                }
            }
        }

        let response = TimeSeriesResponse {
            metric_name: metric_name.to_string(),
            container_name: container_name.to_string(),
            start: start.to_string(),
            end: end.to_string(),
            step: step.to_string(),
            values: points,
        };

        Ok(serde_json::to_string_pretty(&response)?)
    }

    // >>> instruction-file update: query methods accept range fields start/end/step
    pub async fn query_get_cpu_bytes(
        &self,
        container_name: &str,
        start: &str,
        end: &str,
        step: &str,
    ) -> Result<String> {
        let promql = format!(
            r#"sum by(container_name) (rate(cortexbrain_cpu_bytes_alloc{{container_name=~".*{container_name}.*"}}[{step}]))"#
        );
        let raw = self.query_range(&promql, start, end, step).await?;
        self.build_time_series("cortexbrain_cpu_bytes_alloc", container_name, start, end, step, &raw)
    }

    pub async fn query_get_memory_allocated_bytes(
        &self,
        container_name: &str,
        start: &str,
        end: &str,
        step: &str,
    ) -> Result<String> {
        let promql = format!(
            r#"sum by(container_name) (rate(cortexbrain_enter_mem_alloc{{container_name=~".*{container_name}.*"}}[{step}]))"#
        );
        let raw = self.query_range(&promql, start, end, step).await?;
        self.build_time_series("cortexbrain_enter_mem_alloc", container_name, start, end, step, &raw)
    }

    pub async fn query_get_events(
        &self,
        container_name: &str,
        start: &str,
        end: &str,
        step: &str,
    ) -> Result<String> {
        let promql = format!(
            r#"sum by(container_name) (rate(cortexbrain_events_total{{container_name=~".*{container_name}.*"}}[{step}]))"#
        );
        let raw = self.query_range(&promql, start, end, step).await?;
        self.build_time_series("cortexbrain_events_total", container_name, start, end, step, &raw)
    }

    pub async fn query_get_l4_events(
        &self,
        container_name: &str,
        start: &str,
        end: &str,
        step: &str,
    ) -> Result<String> {
        let promql = format!(
            r#"sum by(container_name) (rate(cortexbrain_socket_events_total{{container_name=~".*{container_name}.*"}}[{step}]))"#
        );
        let raw = self.query_range(&promql, start, end, step).await?;
        self.build_time_series("cortexbrain_socket_events_total", container_name, start, end, step, &raw)
    }

    pub async fn query_get_ssl_write_events(
        &self,
        container_name: &str,
        start: &str,
        end: &str,
        step: &str,
    ) -> Result<String> {
        let promql = format!(
            r#"sum by(container_name) (rate(cortexbrain_ssl_write_bytes{{container_name=~".*{container_name}.*"}}[{step}]))"#
        );
        let raw = self.query_range(&promql, start, end, step).await?;
        self.build_time_series("cortexbrain_ssl_write_bytes", container_name, start, end, step, &raw)
    }

    pub async fn query_get_ssl_read_events(
        &self,
        container_name: &str,
        start: &str,
        end: &str,
        step: &str,
    ) -> Result<String> {
        let promql = format!(
            r#"sum by(container_name) (rate(cortexbrain_ssl_read_bytes{{container_name=~".*{container_name}.*"}}[{step}]))"#
        );
        let raw = self.query_range(&promql, start, end, step).await?;
        self.build_time_series("cortexbrain_ssl_read_bytes", container_name, start, end, step, &raw)
    }
}
