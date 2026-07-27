use anyhow::{Ok, Result};
/// main prometheus client
/// accept a request as a query
use reqwest::Client;
#[derive(Clone)]
pub struct PromClient {
    baseurl: String,
    client: Client,
}

const PROMETHEUS_SERVER: &str = "http://localhost:9090/api/v1/query";

impl PromClient {
    pub fn new() -> Result<Self> {
        Ok(PromClient {
            client: Client::new(),
            baseurl: PROMETHEUS_SERVER.to_string(),
        })
    }
    /// creates a query using promQL language
    pub async fn query(&self, promql_query: &str) -> Result<serde_json::Value> {
        let response = self
            .client
            .get(format!("{}", self.baseurl))
            .query(&[("query", promql_query)])
            .send()
            .await?;
        Ok(response.json().await?)
    }
    pub async fn query_get_cpu_bytes(&self, container_name: &str, timeframe: &str) -> Result<String> {
        //query: sum by(container_name) (rate(cortexbrain_cpu_bytes_alloc[10m]))
        let promql = format!(
            r#"sum by(container_name) (rate(cortexbrain_cpu_bytes_alloc{{container_name=~".*{container_name}.*"}}[{timeframe}]))"#
        );
        let res = serde_json::to_string_pretty(&self.query(&promql).await?)?;

        Ok(res)
    }
    pub async fn query_get_memory_allocated_bytes(
        &self,
        container_name: &str,
        timeframe: &str,
    ) -> Result<String> {
        //query: sum by(container_name) (rate(cortexbrain_enter_mem_alloc{container_name="..."}[10m]))
        let promql = format!(
            r#"sum by(container_name) (rate(cortexbrain_enter_mem_alloc{{container_name=~".*{container_name}.*"}}[{timeframe}]))"#
        );
        let res = serde_json::to_string_pretty(&self.query(&promql).await?)?;
        Ok(res)
    }
    pub async fn query_get_events(&self, container_name: &str, timeframe: &str) -> Result<String> {
        //query: sum by(container_name) (rate(cortexbrain_events_total{container_name="..."}[1m]))
        let promql = format!(
            r#"sum by(container_name) (rate(cortexbrain_events_total{{container_name=~".*{container_name}.*"}}[{timeframe}]))"#
        );
        let res = serde_json::to_string_pretty(&self.query(&promql).await?)?;
        Ok(res)
    }
    pub async fn query_get_l4_events(&self, container_name: &str, timeframe: &str) -> Result<String> {
        //query: sum by(container_name) (rate(cortexbrain_socket_events_total{container_name="..."}[10m]))
        let promql = format!(
            r#"sum by(container_name) (rate(cortexbrain_socket_events_total{{container_name=~".*{container_name}.*"}}[{timeframe}]))"#
        );
        let res = serde_json::to_string_pretty(&self.query(&promql).await?)?;
        Ok(res)
    }
    pub async fn query_get_ssl_write_events(
        &self,
        container_name: &str,
        timeframe: &str,
    ) -> Result<String> {
        //query: sum by(container_name) (rate(cortexbrain_ssl_write_bytes{container_name="..."}[10m]))
        let promql = format!(
            r#"sum by(container_name) (rate(cortexbrain_ssl_write_bytes{{container_name=~".*{container_name}.*"}}[{timeframe}]))"#
        );
        let res = serde_json::to_string_pretty(&self.query(&promql).await?)?;
        Ok(res)
    }
    pub async fn query_get_ssl_read_events(
        &self,
        container_name: &str,
        timeframe: &str,
    ) -> Result<String> {
        //query: sum by(container_name) (rate(cortexbrain_ssl_read_bytes{container_name="..."}[10m]))
        let promql = format!(
            r#"sum by(container_name) (rate(cortexbrain_ssl_read_bytes{{container_name=~".*{container_name}.*"}}[{timeframe}]))"#
        );
        let res = serde_json::to_string_pretty(&self.query(&promql).await?)?;
        Ok(res)
    }
}
