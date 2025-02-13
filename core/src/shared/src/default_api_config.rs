/*!
    This module defines the components and parameters for the default API configuration.

    Key functionalities:
    - **API Configuration (`ApiConfig`)**:
        Provides the primary structure for storing API-related settings, including:
        - Base directory and configuration file paths.
        - Module names for EdgeMesh Agent, Gateway, DNS, Proxy, Tunnel, and CNI.
        - Metadata server settings and device configurations.
        - Modes for service filtering, load balancing, and discovery types.
        - Other operational modes like edge, cloud, and manual modes.

    - **Configuration Loading**:
        Implements methods to load configurations from YAML files for:
        - Default settings.
        - Version-specific settings (e.g., `V1`).

    - **EdgeCNI and EdgeDNS Configurations**:
        Handles parsing of specialized sections (`edgeCNI`, `edge_dns`, `cache_dns`, `kubeapi`)
        within the configuration file for fine-grained control.

    Features:
    - **Error Handling**:
        Uses the `anyhow` crate for detailed context in error reporting.
    - **Serialization and Deserialization**:
        Supports `Serialize` and `Deserialize` traits for seamless integration with YAML.
    - **Configurable Defaults**:
        Offers predefined defaults for critical parameters such as `ServiceFilterMode`,
        `LoadBalancerCaller`, and `DiscoveryType`.

    This module is essential for initializing and managing configurations in
    distributed systems with complex edge and cloud operations.
*/
use anyhow::anyhow;
#[allow(unused_imports)]
use anyhow::{Context, Result};
use k8s_openapi::api::core::v1::ConfigMap;
use kube::Api;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs::File;
use tracing_subscriber::fmt::format;

use crate::apiconfig::{
    AgentModules, CommonConfig, EdgeCNIConfig, EdgeDNSConfig, EdgeMeshAgentConfig,
};
use crate::params::{DiscoveryType, LoadBalancerCaller, ServiceFilterMode};

#[derive(Debug)]
pub enum ConfigType {
    Default,
    V1,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ApiConfig {
    pub base_dir: String,
    pub config_file: String,
    pub edgemesh_agent_config_name: String,
    pub edgemesh_gateway_config_name: String,
    pub edgemesh_proxy_module_name: String,
    pub edgemesh_tunnel_module_name: String,
    pub edgemesh_cni_module_name: String,
    pub bridge_device_name: String,
    pub bridge_device_ip: String,
    pub tun_device_name: String,
    pub temp_kube_config_path: String,
    pub temp_core_file_path: String,
    pub meta_server_address: String,
    pub meta_server_cert_dir: String,
    pub meta_server_ca_file: String,
    pub meta_server_cert_file: String,
    pub meta_server_key_file: String,
    pub cloud_mode: String,
    pub manual_mode: String,
    pub empty_node_name: String,
    pub empty_pod_name: String,
    pub service_filter_mode: Option<ServiceFilterMode>,
    pub loadbalancer_caller: Option<LoadBalancerCaller>,
    pub discovery_type: Option<DiscoveryType>,
}

impl ApiConfig {
    pub fn load_from_file<P: AsRef<std::path::Path>>(
        path: P,
        config_type: ConfigType,
    ) -> Result<Self> {
        let cur_path = std::env::current_dir()?;
        println!("The current directory is {}", cur_path.display());
        println!(
            "Trying to load configuration from path: {}",
            path.as_ref().display()
        );
        let cfg_file = File::open(&path).with_context(|| {
            format!(
                "Problem opening config file in path {}",
                path.as_ref().display()
            )
        })?;
        let config_map: serde_yaml::Value =
            serde_yaml::from_reader(cfg_file).context("Failed to parse YAML")?;

        let config_section = match config_type {
            ConfigType::Default => &config_map["default"],
            ConfigType::V1 => &config_map["v1"],
        };

        let config: ApiConfig = serde_yaml::from_value(config_section.clone())
            .context("Failed to extract config section")?;

        Ok(ApiConfig {
            service_filter_mode: Some(ServiceFilterMode {
                filter_if_label_exists_mode: String::from(
                    ServiceFilterMode::filter_if_label_exists_mode(),
                ),
                filter_if_label_doesn_not_exists_mode: String::from(
                    ServiceFilterMode::filter_if_label_doesn_not_exists_mode(),
                ),
            }),
            loadbalancer_caller: Some(LoadBalancerCaller {
                proxy_caller: String::from(LoadBalancerCaller::proxy_caller()),
                gateway_caller: String::from(LoadBalancerCaller::gateway_caller()),
            }),
            discovery_type: Some(DiscoveryType {
                mdns_discovery: String::from(DiscoveryType::mdns_discovery()),
                dht_discovery: String::from(DiscoveryType::dht_discovery()),
            }),
            ..config
        })
    }
}
impl EdgeCNIConfig {
    pub fn load_from_file<P: AsRef<std::path::Path>>(
        path: P,
        config_type: ConfigType,
    ) -> Result<Self> {
        let cfg_file = File::open(path).context("Errore nell'aprire il file di configurazione")?;

        // Analizza il file YAML
        let config_map: serde_yaml::Value =
            serde_yaml::from_reader(cfg_file).context("Errore nella lettura del file YAML")?;

        // Seleziona la sezione corretta del file di configurazione
        let config_section = match config_type {
            ConfigType::Default => &config_map["default"],
            ConfigType::V1 => &config_map["v1"],
        };

        // EdgeCNI section
        let edgecni_section = config_section.get("edgeCNI").ok_or_else(|| {
            anyhow::anyhow!("'edgeCNI' section doesn not exists in the config file")
        })?;

        let edgecni_config: EdgeCNIConfig = serde_yaml::from_value(edgecni_section.clone())
            .context("Error parsing 'edgeCNI' section")?;

        Ok(EdgeCNIConfig { ..edgecni_config })
    }
}
impl EdgeDNSConfig {
    pub async fn load_from_configmap(configmap: Api<ConfigMap>, config_type: ConfigType) -> Result<Self> {
        // Get the content of config.yaml from Kubernetes ConfigMap
        let cm = configmap.get("cortexbrain-client-config").await.context("Failed to get ConfigMap")?;
        
        let config_data = cm.data.ok_or_else(|| anyhow::anyhow!("No data in ConfigMap"))?;

        let config_yaml = config_data.get("config.yaml")
            .ok_or_else(|| anyhow::anyhow!("Missing 'config.yaml' in ConfigMap data"))?
            .clone();

        // Now parse the YAML content
        let config_map: serde_yaml::Value = serde_yaml::from_str(&config_yaml)
            .context("Error reading the yaml file from Kubernetes")?;

        // Extract the relevant config section
        let configs_yaml = config_map
            .get("data")
            .and_then(|data| data.get("config.yaml"))
            .and_then(|values| values.as_str())
            .ok_or_else(|| anyhow::anyhow!("Error reading data.config.yaml from the configmap file"))?;

        let configs: serde_yaml::Value = serde_yaml::from_str(configs_yaml).context("Error parsing 'config.yaml' content")?;

        // Select the correct version
        let config_section = match config_type {
            ConfigType::Default => &configs["default"],
            ConfigType::V1 => &configs["v1"],
        };

        // Edge DNS Section
        let edge_dns_section = config_section.get("edge_dns").ok_or_else(|| {
            anyhow::anyhow!("'edge_dns' section does not exist in the config file")
        })?;

        let edge_dns_config: EdgeDNSConfig = serde_yaml::from_value(edge_dns_section.clone())
            .context("Error parsing 'edge_dns' section")?;

        // Cache DNS section
        let cache_dns_section = config_section.get("cache_dns");

        let cache_dns_config = if let Some(cache_dns_section) = cache_dns_section {
            Some(
                serde_yaml::from_value(cache_dns_section.clone())
                    .context("Error parsing 'cache dns' section")?,
            )
        } else {
            None
        };

        // KubeAPI section
        let kubeapi_section = config_section.get("kubeapi");

        let kubeapi_config = if let Some(kubeapi_section) = kubeapi_section {
            Some(
                serde_yaml::from_value(kubeapi_section.clone())
                    .context("Error parsing 'kubeapi' section")?,
            )
        } else {
            None
        };

        // Return the EdgeDNS configuration
        Ok(EdgeDNSConfig {
            cache_dns: cache_dns_config,
            kube_api_config: kubeapi_config,
            ..edge_dns_config
        })
    }
}
impl CommonConfig {
    pub fn load_from_file<P: AsRef<std::path::Path>>(
        path: P,
        config_type: ConfigType,
    ) -> Result<Self> {
        let cfg_file = File::open(path);

        let file = match cfg_file {
            Ok(file) => file,
            Err(error) => panic!("Problem opening the file: {error:?}"),
        };

        let config_map: serde_yaml::Value =
            serde_yaml::from_reader(file).context("Failed to parse YAML")?;

        let config_section = match config_type {
            ConfigType::Default => &config_map["default"],
            ConfigType::V1 => &config_map["v1"],
        };

        let common_config: CommonConfig = serde_yaml::from_value(config_section.clone())
            .context("Failed to extract config section")?;

        // Return the CommonConfig configuration
        Ok(CommonConfig {
            bridge_device_name: common_config.bridge_device_name,
            bridge_device_ip: common_config.bridge_device_ip,
        })
    }
}
impl EdgeMeshAgentConfig {}
impl AgentModules {}
