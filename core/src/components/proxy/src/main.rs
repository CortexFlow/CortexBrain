//TODO: basic proxy functionalities
//TODO: add integration with prometheus logging system
//TODO: add load balancer between dns servers

mod discovery;
mod proxy;
mod vars;

use k8s_openapi::api::core::v1::ConfigMap;
use kube::{Client, api::Api};
use proxy::Proxy;
use shared::{apiconfig::EdgeProxyConfig, default_api_config::ConfigType};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format::FmtSpan;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_level(true)
        .with_span_events(FmtSpan::NONE)
        .without_time()
        .with_file(false)
        .pretty()
        .with_env_filter(EnvFilter::new("info"))
        .with_line_number(false)
        .init();

    let client = Client::try_default().await?;
    let configmap: Api<ConfigMap> = Api::namespaced(client.clone(), "cortexflow");

    let proxycfg = EdgeProxyConfig::load_from_configmap(configmap, ConfigType::Default).await?;
    let proxy = Proxy::new(proxycfg).await?;

    proxy.start().await?;
    proxy.get_info().await;

    Ok(())
}
