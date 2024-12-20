/*!
    This module defines structures and parameters for key functionalities
    in service filtering, load balancing, and service discovery.

    Key components:
    - **Service Filter Mode**:
        Encapsulates the filtering logic for services based on label existence,
        managed through the `ServiceFilterMode` structure.

    - **Load Balancer Caller**:
        Defines the calling logic for proxies and gateways using the
        `LoadBalancerCaller` structure.

    - **Service Discovery Type**:
        Specifies discovery mechanisms, including mDNS and DHT, via
        the `DiscoveryType` structure.

    Features:
    - **Serialization and Deserialization**:
        Structures implement `Serialize` and `Deserialize` traits for compatibility
        with formats such as JSON and YAML.
    - **Modularity**:
        Clearly separated components allow for scalability and reuse.
    - **Convenient Defaults**:
        Static methods provide default values for common use cases, such as
        `"FilterIfLabelExists"` or `"ProxyCaller"`.

    This module is intended for use in distributed systems requiring flexible
    configurations for service filtering, load balancing, and discovery protocols.
*/

use serde::{Deserialize, Serialize};

/* ServiceFilter Mode */
#[derive(Debug, Deserialize, Clone, Serialize)]
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

/* LoadBalancer Caller */
#[derive(Debug, Deserialize, Clone, Serialize)]
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

//Discovery Type
#[derive(Debug, Deserialize, Clone, Serialize)]
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
