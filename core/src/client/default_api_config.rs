/* 

    Components of the default Api config. 
    Contains the Default and V1 parameters and implementation  

*/


use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs::File;

/* ###################################################################################
##################################  DEFAULT ######################################
################################################################################### */


#[derive(Debug)]
pub enum ConfigType {
    Default,
    V1,
}

#[derive(Debug, Deserialize, Clone,Serialize)]
pub struct ServiceFilterMode {
    pub filter_if_label_exists_mode: String,
    pub filter_if_label_doesn_not_exists_mode: String,
}
impl ServiceFilterMode {
    pub fn filter_if_label_exists_mode() -> &'static str {
        "FilterIfLabelExists"
    }
    pub fn filter_if_label_doesn_not_exists_mode() -> &'static str {
        "FilterIfLabelDoesNotExists"
    }
}
#[derive(Debug, Deserialize, Clone,Serialize)]
pub struct LoadBalancerCaller {
    pub proxy_caller: String,
    pub gateway_caller: String,
}
impl LoadBalancerCaller {
    pub fn proxy_caller() -> &'static str {
        "ProxyCaller"
    }
    pub fn gateway_caller() -> &'static str {
        "GatewayCaller"
    }
}
#[derive(Debug, Deserialize, Clone,Serialize)]
pub struct DiscoveryType {
    pub mdns_discovery: String,
    pub dht_discovery: String,
}
impl DiscoveryType {
    pub fn mdns_discovery() -> &'static str {
        "MDNS"
    }
    pub fn dht_discovery() -> &'static str {
        "DHT"
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub base_dir: String,
    pub config_file: String,
    pub edgemesh_agent_config_name: String,
    pub edgemesh_gateway_config_name: String,
    pub edgemesh_dns_module_name: String,
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
    pub edge_mode: String,
    pub edge_mode_enable: bool,
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
