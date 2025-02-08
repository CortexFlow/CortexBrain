/*!
    This module defines a series of configuration structures for a
    distributed network infrastructure. It includes parameters for
    components such as Edge Mesh, Gateway, DNS, and CNI (Container Network Interface).

    Key functionalities:
    - **Kubernetes API Configuration**:
        Manages parameters for interacting with the Kubernetes API using the
        `KubeApiConfig` structure.

    - **Edge Mesh Configuration**:
        Configures agents (`EdgeMeshAgentConfig`) and gateways (`EdgeMeshGatewayConfig`)
        for the Edge Mesh network.

    - **Edge DNS**:
        Defines parameters for a custom DNS system, including interfaces,
        ports, and caching, through the `EdgeDNSConfig` structure.

    - **Edge Proxy**:
        Configures the Edge network proxy, with support for load balancing
        and Socks5 (`EdgeProxyConfig`).

    - **CNI Configuration**:
        Manages containerized network settings, such as tunneling and Mesh CIDR,
        using the `EdgeCNIConfig` structure.

    - **Gateway Components**:
        Configures specific gateway settings via `EdgeGatewayConfig`.

    Features:
    - **Serialization and Deserialization**:
        Uses `Serialize` and `Deserialize` traits to support formats like JSON and YAML.
    - **Modularity**:
        Independent configurations for clear and scalable management.
    - **Flexibility**:
        The use of `Option<T>` allows for partial and customizable configurations.

    This module is designed as an integral part of a distributed system's
    edge computing configuration.
*/



#[allow(unused_imports)]


use serde::{Deserialize, Serialize};

// ==================================================================
// ======================== Agent Section ===========================
// ==================================================================

pub struct EdgeMeshAgentConfig {
    pub kubeapi_config: Option<KubeApiConfig>,
    pub common_config: Option<CommonConfig>,
    pub modules : Option<AgentModules>,
}
pub struct AgentModules {
    pub edge_dns_config: Option<EdgeDNSConfig>,
    pub edge_proxy_config: Option<EdgeProxyConfig>,
    pub edge_cni_config: Option<EdgeCNIConfig>,
}


// ==================================================================
// ======================= Gateway Section ==========================
// ==================================================================

pub struct EdgeMeshGatewayConfig {}
pub struct GatewayModules {
    pub edge_gateway_config: Option<EdgeGatewayConfig>,
}
pub struct EdgeGatewayConfig {
    pub enable: bool,
    pub nic: String,
    pub include_ip: String,
    pub exclude_ip: String,
    pub loadbalancer: Option<LoadBalancer>,
}



// ==================================================================
// ======================= KubeAPI Section ==========================
// ==================================================================

#[derive(Clone, Serialize, Deserialize,Debug)]
pub struct KubeApiConfig {
    pub master: Option<String>,
    pub content_type: Option<String>,
    pub qps: i32,
    pub burst: i32,
    pub kube_config: Option<String>,
    pub meta_server: Option<String>,
    pub delete_kube_config: bool,
}
#[derive(Serialize,Deserialize,Clone)]
pub struct CommonConfig {
    pub bridge_device_name: String,
    pub bridge_device_ip: String,
    /* pub pprof : Option<PprofConfig> */
}
#[derive(Clone,Serialize,Deserialize)]
pub struct PprofConfig {}

// ==================================================================
// ======================= MetaServer Section =======================
// ==================================================================

pub struct MetaServer {
    pub server: String,
    pub security: Option<MetaServerSecurity>,
}
pub struct MetaServerSecurity {}


// ==================================================================
// ======================== Proxy Section ===========================
// ==================================================================

#[derive(Serialize,Deserialize,Clone)]
pub struct EdgeProxyConfig {
    pub enable: bool,
    pub listen_interface: String,
    pub loadbalancer: Option<LoadBalancer>,
    pub socks5proxy: Option<Socks5Proxy>,
    pub service_filter_mode: String,
}
#[derive(Serialize,Deserialize,Clone)]
pub struct Socks5Proxy {
    pub enable: bool,
    pub listen_port: i32,
    pub nodename: String,
    pub namespace: String,
}

// ==================================================================
// ========================= CNI Section ============================
// ==================================================================


#[derive(Serialize, Deserialize)]
pub struct EdgeCNIConfig {
    pub enable: bool,
    pub encap_ip: String,
    pub tun_mode: i32,
    pub mesh_cidr_config: Option<MeshCIDRConfig>,
}

#[derive(Serialize, Deserialize)]
pub struct MeshCIDRConfig {
    pub cloud_cidr: Vec<String>,
    pub edge_cidr: Vec<String>,
}

// ==================================================================
// ========================= DNS Section ============================
// ==================================================================

#[derive(Clone, Serialize, Deserialize,Debug)]
pub struct EdgeDNSConfig {
    pub enable: bool,
    pub listen_interface: String,
    pub listen_port: i32,
    pub kube_api_config: Option<KubeApiConfig>,
    pub cache_dns: Option<CacheDNS>,
}

#[derive(Clone, Serialize, Deserialize,Debug)]
pub struct CacheDNS {
    pub enable: bool,
    pub auto_detect: bool,
    pub upstream_servers: Vec<String>,
    pub cache_ttl: u32,
}

// ==================================================================
// ====================== LoadBalancer Section ======================
// ==================================================================

#[derive(Serialize,Deserialize,Clone)]
pub struct LoadBalancer {
    pub caller: String,
    pub nodename: String,
    //add consistent hash
}
