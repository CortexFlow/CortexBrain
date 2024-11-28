/* The corefile.go file in Kubernetes is part of the source code for CoreDNS, which is the default DNS server used in Kubernetes clusters for service name resolution and other internal DNS operations. This file defines the structure and functionality associated with CoreDNS configuration, specifically the object called Corefile.

Corefile.go's main functionality
CoreDNS configuration parsing:

Handles the logic for reading, parsing, and interpreting the CoreDNS configuration file, which is usually called Corefile. This file specifies how CoreDNS should behave, which plugins to use, and how to handle DNS queries.
Configuration validation:

Verifies that the CoreDNS configuration is valid. For example, it checks that the configuration blocks are correct and that the defined plugins are supported.
Configuration Manipulation:

Allows programmatic changes to the Corefile configuration. For example, if the cluster requires an update of DNS zones or the addition of a new plugin, this file defines the structures and functions to make those changes.
Useful Data Structures:

Defines data structures to represent the Corefile, with each block and directive described in a structured way for programmable management.
Interface with Kubernetes:

Provides functionality to integrate CoreDNS with Kubernetes clusters, such as configuring internal DNS services to resolve names such as my-service.my-n

*/
use crate::client::client::Client;
use crate::utilities::utilities;

use anyhow::{anyhow, Error, Result};
use kube::api::{Api, ApiResource, DynamicObject, ListParams};
use kube::{core::DynamicObject as CoreDynamicObject, discovery};
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::net::IpAddr;
use tracing::{error, info};

/* template block */

const STUB_DOMAIN_BLOCK: &str = r#"{{domain_name}}::{{port}}{
    bind {{local_ip}}
    cache {{cache_ttl}}
    errors
    forward . {{upstream_servers}}{
        force_tcp
    }
    {{kubernetes_plugin}}
    log
    loop
    reload
}"#;

const KUBERNETES_PLUGIN_BLOCK: &str = r#"Kubernetes cluster.local in-addr.arpa ip6.arpa{
    {{api_server}}
    pods insecure
    fallthrough in-addr.arpa ip6.arpa
    ttl {{ttl}}
}"#;

/* constants */
const DEFAULT_TTL: u32 = 30;
const DEFAULT_UPSTREAM_SERVER: &str = "/etc/resolv.conf";

/* parameters */
#[derive(Serialize)]
pub struct StubDomainInfo {
    domain_name: String,
    local_ip: String,
    port: String,
    cache_ttl: u32,
    upstream_servers: String,
    kubernetes_plugin: String,
}

#[derive(Serialize)]
pub struct KubernetesPluginInfo {
    api_server: String,
    ttl: u32,
}

fn generate_stub_domain_block(config: StubDomainInfo) -> Result<String, Error> {
    let template = STUB_DOMAIN_BLOCK.to_string();
    let mut rendered = template;

    rendered = rendered.replace("{{domain_name}}", &config.domain_name);
    rendered = rendered.replace("{{port}}", &config.port.to_string());
    rendered = rendered.replace("{{local_ip}}", &config.local_ip.to_string());
    rendered = rendered.replace("{{cache_ttl}}", &config.cache_ttl.to_string());
    rendered = rendered.replace("{{upstream_servers}}", &config.upstream_servers);
    rendered = rendered.replace("{{kubernetes_plugin}}", &config.kubernetes_plugin);

    Ok(rendered)
}

fn generate_kubernetes_plugin_block(config: KubernetesPluginInfo) -> Result<String, Error> {
    let template = KUBERNETES_PLUGIN_BLOCK.to_string();
    let mut rendered = template;

    rendered = rendered.replace("{{api_server}}", &config.api_server);
    rendered = rendered.replace("{{ttl}}", &config.ttl.to_string());

    Ok(rendered)
}

pub async fn detect_cluster_dns(client: Client) -> Vec<String> {
    let namespace = "kube-system";
    let mut servers = HashSet::new();

    // Scoperta delle risorse per oggetti dinamici
    let discovery = discovery::Discovery::new(client.get_client().clone());

    // Crea un GroupVersionKind per "Service"
    let gvk = kube::api::GroupVersionKind {
        group: "".to_string(),
        version: "v1".to_string(),
        kind: "Service".to_string(),
    };

    // Risolve la risorsa "Service"
    let service_resource = match discovery.resolve_gvk(&gvk){
        Some((resource, _capabilities)) => resource,  // Estrai solo ApiResource
        None => {
            error!("Failed to resolve Service resource: Service resource not found");
            return Vec::new(); // Nessuna risorsa trovata
        }
    };
    
    let services: Api<DynamicObject> = Api::namespaced_with(client.get_client().clone(), namespace, &service_resource);

    if let std::result::Result::Ok(coredns) = services.get("coredns").await {
        if let Some(cluster_ip) = coredns.data["spec"]["clusterIP"].as_str() {
            if cluster_ip != "None" {
                servers.insert(cluster_ip.to_string());
            }
        }
    }

    if let std::result::Result::Ok(kubedns) = services.get("kube-dns").await {
        if let Some(cluster_ip) = kubedns.data["spec"]["clusterIP"].as_str() {
            if cluster_ip != "None" {
                servers.insert(cluster_ip.to_string());
            }
        }
    }

    let label_selector = ListParams::default().labels("k8s-app=kube-dns");
    if let std::result::Result::Ok(kube_dns_list) = services.list(&label_selector).await {
        for item in kube_dns_list.items {
            if let Some(cluster_ip) = item.data["spec"]["clusterIP"].as_str() {
                if cluster_ip != "None" {
                    servers.insert(cluster_ip.to_string());
                }
            }
        }
    }

    let mut servers: Vec<String> = servers.into_iter().collect();
    servers = utilities::remove_duplicates(servers);

    if servers.is_empty() {
        error!("Unable to automatically detect cluster DNS. Do you have CoreDNS or kube-dns installed in your cluster?");
    } else {
        info!("Automatically detected cluster DNS: {:?}", servers);
    }

    return servers
}

/*
// Funzione per ottenere l'indirizzo IP dell'interfaccia di rete (simile a netutil.GetInterfaceIP)
fn get_interface_ip(interface: &str) -> Result<IpAddr, Error> {
    let interfaces = pnet::datalink::interfaces();
    for iface in interfaces {
        if iface.name == interface {
            for ip in iface.ips {
                if let IpAddr::V4(ipv4) = ip.ip() {
                    return Ok(IpAddr::V4(ipv4));
                }
            }
        }
    }
    Err(anyhow!("Failed to find interface with name: {}", interface))
}

// Funzione principale che aggiorna il Corefile
pub async fn update_corefile(cfg: &super::kernel::EdgeDNSConfig, kube_client: Client) -> Result<(), Error> {
    // Ottieni l'indirizzo IP dell'interfaccia di rete
    let listen_ip = get_interface_ip(&cfg.listen_interface)?;
    
    // Imposta i valori predefiniti per cacheTTL e upstreamServers
    let mut cache_ttl = DEFAULT_TTL;
    let mut upstream_servers = vec![DEFAULT_UPSTREAM_SERVER.to_string()];

    // Ottieni la stringa di configurazione del plugin Kubernetes
    let kubernetes_plugin = get_kubernetes_plugin_str(cfg)?;

    // Se il caching DNS è abilitato
    if cfg.cache_dns.enable {
        upstream_servers.clear();

        // Se l'auto-rilevamento è abilitato, aggiungi gli upstream server rilevati dal cluster
        if cfg.cache_dns.auto_detect {
            upstream_servers.extend(detect_cluster_dns(kube_client).await);
        }

        // Aggiungi gli upstream servers specificati nella configurazione, ignorando le righe vuote
        for server in &cfg.cache_dns.upstream_servers {
            let server = server.trim();
            if server.is_empty() {
                continue;
            }
            if is_valid_address(server) {
                upstream_servers.push(server.to_string());
            } else {
                error!("Invalid address: {}", server);
            }
        }

        // Rimuovi eventuali duplicati dagli upstream servers
        upstream_servers = utilities::remove_duplicates(upstream_servers);

        if upstream_servers.is_empty() {
            return Err(anyhow!("Failed to get node local dns upstream servers"));
        }

        // Aggiorna il TTL della cache e disabilita il plugin Kubernetes di CoreDNS
        cache_ttl = cfg.cache_dns.cache_ttl;
    }

    // Crea la mappa per i domini stub
    let mut stub_domain_map = std::collections::HashMap::new();
    stub_domain_map.insert(".".to_string(), upstream_servers);

    // Crea la stringa di configurazione per il dominio stub
    let stub_domain_str = generate_stub_domain_str(
        stub_domain_map,
        &StubDomainInfo {
            domain_name: ".".to_string(),
            local_ip: listen_ip.to_string(),
            port: cfg.listen_port.to_string(),
            cache_ttl,
            upstream_servers: upstream_servers.join(", "),
            kubernetes_plugin,
        },
    )?;

    // Scrivi la nuova configurazione nel file temporaneo
    let temp_corefile_path = "/tmp/Corefile"; // Puoi usare il path predefinito o configurabile
    fs::write(temp_corefile_path, stub_domain_str)?;

    info!("Corefile updated successfully at {}", temp_corefile_path);

    Ok(())
}

// Helper per ottenere la configurazione del plugin Kubernetes
fn get_kubernetes_plugin_str(cfg: &super::kernel::EdgeDNSConfig) -> Result<String, Error> {
    // Logica per generare la stringa di configurazione del plugin Kubernetes
    if cfg.kubernetes_plugin_enable {
        let plugin_config = KubernetesPluginInfo {
            api_server: cfg.kubernetes_api_server.clone(),
            ttl: cfg.kubernetes_ttl.unwrap_or(DEFAULT_TTL),
        };
        generate_kubernetes_plugin_block(plugin_config)
    } else {
        Ok("".to_string()) // Nessun plugin Kubernetes se non abilitato
    }
} */

// Helper per validare gli indirizzi
fn is_valid_address(address: &str) -> bool {
    address.parse::<IpAddr>().is_ok()
}
