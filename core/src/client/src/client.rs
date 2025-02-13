/* Contains the Client configuration  */
#[allow(unused_imports)]

use anyhow::{anyhow, Error, Result};
use k8s_openapi::api::core::v1::Pod;
use kube::api::ListParams;
use kube::{Api, Client as KubeClient};
use shared::default_api_config::{ApiConfig,ConfigType};


use tracing::info;

#[derive(Clone)]
pub struct Client {
    config: ApiConfig,
    pub kube_client: KubeClient,
}

impl Client {
    //default config_type (ConfigType::Default)
    pub async fn new_client(config_path: &str,config_type: Option<ConfigType>) -> Result<Self, Error> {
        let config_type = config_type.unwrap_or(ConfigType::Default); //use default if config_type ==none
        //let config_path = "./src/client/config.yaml";
        let api_config = ApiConfig::load_from_file(config_path, config_type)?;
        let kube_client = KubeClient::try_default().await?;
        Ok(Client {
            config: api_config,
            kube_client,
        })
    }

    pub fn get_client(&self) -> &KubeClient {
        &self.kube_client
    }

    pub async fn list_pods(&self, namespace: &str) -> Result<Vec<Pod>, Error> {
        let pods: Api<Pod> = Api::namespaced(self.kube_client.clone(), namespace);
        let lp = ListParams::default();
        let pod_list = pods.list(&lp).await?;
        if pod_list.items.is_empty() {
            return Err(anyhow!("No pods found"));
        }
        Ok(pod_list.items)
    }

    pub fn print_config(&self) {
        info!("------- E D G E M E S H  N E T W O R K -------\n");
        info!("Base dir: {}", self.config.base_dir);
        info!("Config File: {}", self.config.config_file);
        info!(
            "Edgemesh Agent config name: {}",
            self.config.edgemesh_agent_config_name
        );
        info!(
            "Edgemesh Gateway config name: {}",
            self.config.edgemesh_gateway_config_name
        );
        info!(
            "Edgemesh Proxy Module Name: {}",
            self.config.edgemesh_proxy_module_name
        );
        info!(
            "Edgemesh Tunnel Module Name: {}",
            self.config.edgemesh_tunnel_module_name
        );
        info!(
            "Edgemesh CNI Module Name: {}",
            self.config.edgemesh_cni_module_name
        );
        info!("Bridge Device: {}", self.config.bridge_device_name);
        info!("Bridge Device IP: {}", self.config.bridge_device_ip);
        info!("TUN Device Name: {}", self.config.tun_device_name);
        info!(
            "Temp Kube Config Path: {}",
            self.config.temp_kube_config_path
        );
        info!("Temp Core File Path: {}", self.config.temp_core_file_path);
        info!("Meta Server Address: {}", self.config.meta_server_address);
        info!("Meta Server Cert Dir: {}", self.config.meta_server_cert_dir);
        info!("Meta Server CA File: {}", self.config.meta_server_ca_file);
        info!(
            "Meta Server Cert File: {}",
            self.config.meta_server_cert_file
        );
        info!("Meta Server Key File: {}", self.config.meta_server_key_file);
        info!("Cloud Mode: {}", self.config.cloud_mode);
        info!("Manual Mode: {}", self.config.manual_mode);
        info!("Empty Node Name: {}", self.config.empty_node_name);
        info!("Empty Pod Name: {}", self.config.empty_pod_name);
    }
}
