/* Contains the Config structures used in ApiConfig */

use anyhow::{Context,Result};
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs::File;

use default_api_config::ConfigType;

use super::default_api_config;

/* ###################################################################################
################################# CONFIG ##########################################
################################################################################### */

/* Defines the API config for the Kubernetes Plugin */
pub struct EdgeMeshAgentConfig {}
pub struct AgentModules {
    pub edge_dns_config: Option<EdgeDNSConfig>,
    pub edge_proxy_config: Option<EdgeProxyConfig>,
    pub edge_cni_config: Option<EdgeCNIConfig>,
}
pub struct EdgeMeshGatewayConfig {}
pub struct GatewayModules {
    pub edge_gateway_config: Option<EdgeGatewayConfig>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct KubeApiConfig {
    pub master: Option<String>,
    pub content_type: Option<String>,
    pub qps: i32,
    pub burst: i32,
    pub kube_config: Option<String>,
    pub meta_server: Option<String>,
    pub delete_kube_config: bool,
}

pub struct MetaServer {
    pub server: String,
    pub security: Option<MetaServerSecurity>,
}
pub struct MetaServerSecurity {}

pub struct CommonConfig {
    pub bridge_device_name: String,
    pub bridge_device_ip: String,
}
pub struct EdgeProxyConfig {
    pub enable: bool,
    pub listen_interface: String,
    pub loadbalancer: Option<LoadBalancer>,
    pub socks5proxy: Option<Socks5Proxy>,
    pub service_filter_mode: String,
}
pub struct Socks5Proxy {
    pub enable: bool,
    pub listen_port: i32,
    pub nodename: String,
    pub namespace: String,
}
#[derive(Serialize,Deserialize)]
pub struct EdgeCNIConfig {
    pub enable: bool,
    pub encap_ip: String,
    pub tun_mode: i32,
    pub mesh_cidr_config: Option<MeshCIDRConfig>,
}
impl EdgeCNIConfig{
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
#[derive(Serialize,Deserialize)]
pub struct MeshCIDRConfig {
    pub cloud_cidr: Vec<String>,
    pub edge_cidr: Vec<String>,
}
pub struct EdgeGatewayConfig {
    pub enable: bool,
    pub nic: String,
    pub include_ip: String,
    pub exclude_ip: String,
    pub loadbalancer: Option<LoadBalancer>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EdgeDNSConfig {
    pub enable: bool,
    pub listen_interface: String,
    pub listen_port: i32,
    pub kube_api_config: Option<KubeApiConfig>,
    pub cache_dns: Option<CacheDNS>,}
impl EdgeDNSConfig {
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

        // Edge DNS Section
        let edge_dns_section = config_section.get("edge_dns").ok_or_else(|| {
            anyhow::anyhow!("'edge_dns' section doesn not exists in the config file")
        })?;

        let edge_dns_config: EdgeDNSConfig = serde_yaml::from_value(edge_dns_section.clone())
            .context("Error parsing 'edge_dns' section")?;

        // Cache DNS section
        let cache_dns_section = config_section.get("cache_dns");

        let cache_dns_config = if let Some(cache_dns_section) = cache_dns_section {
            Some(
                serde_yaml::from_value(cache_dns_section.clone())
                    .context("Error parsing 'edge_dns' section")?,
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

#[derive(Clone, Serialize, Deserialize)]
pub struct CacheDNS {
    pub enable: bool,
    pub auto_detect: bool,
    pub upstream_servers: Vec<String>,
    pub cache_ttl: u32,
}
pub struct LoadBalancer {
    pub caller: String,
    pub nodename: String,
    //add consistent hash
}
