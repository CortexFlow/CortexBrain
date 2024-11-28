use anyhow::{Context, Error, Result};
use serde::Deserialize;
use serde_yaml;
use std::fs::File;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
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
    pub cloud_mode: String,
    pub manual_mode: String,
    pub empty_node_name: String,
    pub empty_pod_name: String,
}

#[derive(Debug)]
pub enum ConfigType {
    Default,
    V1,
}

#[derive(Clone)]
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
    pub cloud_mode: String,
    pub manual_mode: String,
    pub empty_node_name: String,
    pub empty_pod_name: String,
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

        let config: Config = serde_yaml::from_value(config_section.clone())
            .context("Failed to extract config section")?;

        Ok(ApiConfig {
            base_dir: config.base_dir,
            config_file: config.config_file,
            edgemesh_agent_config_name: config.edgemesh_agent_config_name,
            edgemesh_gateway_config_name: config.edgemesh_gateway_config_name,
            edgemesh_dns_module_name: config.edgemesh_dns_module_name,
            edgemesh_proxy_module_name: config.edgemesh_proxy_module_name,
            edgemesh_tunnel_module_name: config.edgemesh_tunnel_module_name,
            edgemesh_cni_module_name: config.edgemesh_cni_module_name,
            bridge_device_name: config.bridge_device_name,
            bridge_device_ip: config.bridge_device_ip,
            tun_device_name: config.tun_device_name,
            temp_kube_config_path: config.temp_kube_config_path,
            temp_core_file_path: config.temp_core_file_path,
            meta_server_address: config.meta_server_address,
            meta_server_cert_dir: config.meta_server_cert_dir,
            meta_server_ca_file: config.meta_server_ca_file,
            meta_server_cert_file: config.meta_server_cert_file,
            meta_server_key_file: config.meta_server_key_file,
            edge_mode: config.edge_mode,
            cloud_mode: config.cloud_mode,
            manual_mode: config.manual_mode,
            empty_node_name: config.empty_node_name,
            empty_pod_name: config.empty_pod_name,
        })
    }
}
