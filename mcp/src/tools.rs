use crate::prometheus::PromClient;
use anyhow::Result;
use rmcp::handler::server::ServerHandler;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{Implementation, ServerCapabilities, ServerInfo};
use rmcp::{tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

// >>> instruction-file update: request contract now includes start/end/step
#[derive(Deserialize, JsonSchema)]
struct Params {
    container_name: String,
    start: String,
    end: String,
    step: String,
}

#[derive(Clone)]
pub struct PrometheusTool {
    prometheus: PromClient,
    tool_router: ToolRouter<PrometheusTool>,
}

impl PrometheusTool {
    pub fn new() -> Result<Self> {
        Ok(Self {
            prometheus: PromClient::new()?,
            tool_router: Self::tool_router(),
        })
    }
}

#[tool_router]
impl PrometheusTool {
    #[tool(name = "get_cpu_bytes", description = "CPU bytes allocation per event")]
    pub async fn get_cpu_bytes(
        &self,
        Parameters(params): Parameters<Params>,
    ) -> Result<String, ()> {
        // >>> instruction-file update: no panic path; return MCP-style error propagation
        match self
            .prometheus
            .query_get_cpu_bytes(&params.container_name, &params.start, &params.end, &params.step)
            .await
        {
            Ok(res) => Ok(res),
            Err(_) => Err(()),
        }
    }

    #[tool(
        name = "get_memory_allocated_bytes",
        description = "Bytes requested via mmap syscalls"
    )]
    pub async fn get_memory_allocated_bytes(
        &self,
        Parameters(params): Parameters<Params>,
    ) -> Result<String, ()> {
        // >>> instruction-file update: no panic path; return MCP-style error propagation
        match self
            .prometheus
            .query_get_memory_allocated_bytes(&params.container_name, &params.start, &params.end, &params.step)
            .await
        {
            Ok(res) => Ok(res),
            Err(_) => Err(()),
        }
    }

    #[tool(
        name = "get_events",
        description = "Total number of eBPF events processed across all perf buffers"
    )]
    pub async fn get_events(
        &self,
        Parameters(params): Parameters<Params>,
    ) -> Result<String, ()> {
        // >>> instruction-file update: no panic path; return MCP-style error propagation
        match self
            .prometheus
            .query_get_events(&params.container_name, &params.start, &params.end, &params.step)
            .await
        {
            Ok(res) => Ok(res),
            Err(_) => Err(()),
        }
    }

    #[tool(
        name = "get_l4_events",
        description = "Total number of socket state events processed"
    )]
    pub async fn get_l4_events(
        &self,
        Parameters(params): Parameters<Params>,
    ) -> Result<String, ()> {
        // >>> instruction-file update: no panic path; return MCP-style error propagation
        match self
            .prometheus
            .query_get_l4_events(&params.container_name, &params.start, &params.end, &params.step)
            .await
        {
            Ok(res) => Ok(res),
            Err(_) => Err(()),
        }
    }

    #[tool(
        name = "get_ssl_write_events",
        description = "Total bytes requested by the ssl_write function"
    )]
    pub async fn get_ssl_write_events(
        &self,
        Parameters(params): Parameters<Params>,
    ) -> Result<String, ()> {
        // >>> instruction-file update: no panic path; return MCP-style error propagation
        match self
            .prometheus
            .query_get_ssl_write_events(&params.container_name, &params.start, &params.end, &params.step)
            .await
        {
            Ok(res) => Ok(res),
            Err(_) => Err(()),
        }
    }

    #[tool(
        name = "get_ssl_read_events",
        description = "Total bytes requested by the ssl_read function"
    )]
    pub async fn get_ssl_read_events(
        &self,
        Parameters(params): Parameters<Params>,
    ) -> Result<String, ()> {
        // >>> instruction-file update: no panic path; return MCP-style error propagation
        match self
            .prometheus
            .query_get_ssl_read_events(&params.container_name, &params.start, &params.end, &params.step)
            .await
        {
            Ok(res) => Ok(res),
            Err(_) => Err(()),
        }
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
