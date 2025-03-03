//TODO: basic proxy functionalities
//TODO: add integration with prometheus loggin system
//TODO: add load balancer between dns servers


mod proxy;
mod vars;

use kube::{api::Api,Client};
use k8s_openapi::api::core::v1::ConfigMap;
use shared::{apiconfig::EdgeProxyConfig, default_api_config::ConfigType};
use proxy::Proxy;


#[tokio::main]
async fn main()-> Result<(),anyhow::Error> {

    let client = Client::try_default().await?;
    let configmap: Api<ConfigMap> = Api::namespaced(client.clone(), "cortexflow");

    let proxycfg= EdgeProxyConfig::load_from_configmap(configmap,ConfigType::Default).await?;
    let proxy = Proxy::new(proxycfg,client.clone()).await?;

    proxy.start().await;
    proxy.get_info().await;

    Ok(())
}