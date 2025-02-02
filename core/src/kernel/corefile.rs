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
#[allow(unused_imports)]
use crate::client::apiconfig::EdgeDNSConfig;
use crate::client::client::Client;
use crate::kernel::utilities::{get_interfaces, is_valid_ip, remove_duplicates};
use anyhow::{anyhow, Error, Result};
use k8s_openapi::api::core::v1::{Capabilities, ConfigMap};
use kube::api::{Patch, PatchParams};
use kube::api::{Api, DynamicObject, ListParams};
use kube::discovery;
use k8s_openapi::api::core::v1::Service; 
use serde::Serialize;
use serde_json::json;
use std::collections::HashSet;
use std::fs;
use std::net::IpAddr;
use tracing::{error, info,instrument, warn};

/* template block */

const STUB_DOMAIN_BLOCK: &str = r#"{{domain_name}}:{{port}} {
    bind {{local_ip}}
    log
    errors
    forward . {{upstream_servers}} {
        force_tcp
    }
    cache {{cache_ttl}}
    loop
    reload
}"#;



//TODO: add certificate to protect the route
//TODO: auto deduct the port 
const KUBERNETES_PLUGIN_BLOCK: &str = r#"kubernetes cluster.local in-addr.arpa ip6.arpa{
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

#[instrument(skip(client))]
pub async fn detect_cluster_dns(client: Client) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let namespace = "kube-system";
    let mut servers = HashSet::new();

    info!("Running DNS service detection...");

    let services: Api<Service> = Api::namespaced(client.get_client().clone(), namespace);
    info!("Initialized API for services in namespace: {}", namespace);

    let label_selector = ListParams::default().labels("k8s-app=kube-dns");
    info!("Using label selector: k8s-app=kube-dns");

    let service_list = match services.list(&label_selector).await {
        Ok(list) => {
            info!("Successfully retrieved list of services with label k8s-app=kube-dns");
            list
        }
        Err(e) => {
            error!("Failed to retrieve services: {}", e);
            return Err(e.into());
        }
    };

    info!("Processing {} services...", service_list.items.len());
    for service in service_list.items {
        let service_name = service.metadata.name.clone().unwrap_or_else(|| "unnamed".to_string());
        info!("Processing service: {}", service_name);

        if let Some(spec) = service.spec {
            if let Some(cluster_ip) = spec.cluster_ip {
                if cluster_ip != "None" {
                    info!("Found valid ClusterIP: {} for service: {}", cluster_ip, service_name);
                    servers.insert(cluster_ip);
                } else {
                    info!("Service {} has ClusterIP set to 'None', skipping", service_name);
                }
            } else {
                info!("Service {} has no ClusterIP, skipping", service_name);
            }
        } else {
            info!("Service {} has no spec, skipping", service_name);
        }
    }

    let servers: Vec<String> = servers.into_iter().collect();
    info!("Detected unique DNS servers: {:?}", servers);

    if servers.is_empty() {
        error!("Unable to automatically detect cluster DNS. Do you have CoreDNS or kube-dns installed in your cluster?");
        Err("No DNS service detected".into())
    } else {
        info!("Automatically detected cluster DNS: {:?}", servers);
        Ok(servers)
    }
}
// return the interface ip
fn get_interface_ip(interface: &str) -> Result<IpAddr, Error> {
    /*
    Lib reference: pnet:
        https://crates.io/crates/pnet
     */
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
    get_interfaces();
    Err(anyhow!(
        "Failed to find interface with name: {:?}",
        interface
    ))
}

//update corefile function
#[instrument(skip(kube_client))]
pub async fn update_corefile(cfg: EdgeDNSConfig, kube_client: &Client) -> Result<(), Error> {
    info!("Updating the EdgeDNS corefile configuration\n\n");
    info!("Retrieving the corefile current configuration");
    let configmaps: Api<ConfigMap> =
        Api::namespaced(kube_client.get_client().clone(), "kube-system");
    let mut corefile_configmap = configmaps.get("coredns").await?;
    info!("{:?}\n\n", corefile_configmap);

    //TODO: inject in the kubernetes corefile the modified parameters

    // obtain the interface ip address
    let listen_ip = get_interface_ip(&cfg.listen_interface)?;

    info!("listener ip {}", listen_ip);

    // Set default values for cacheTTL and upstreamServers
    let mut cache_ttl = DEFAULT_TTL;
    let mut upstream_servers = vec![DEFAULT_UPSTREAM_SERVER.to_string()];

    info!("Cache ttl {}", cache_ttl);

    info!("upstream server {:?}", upstream_servers);

    // Get the Kubernetes plugin configuration string
    let kubernetes_plugin = get_kubernetes_plugin_str(cfg.clone())?;

    info!("kubernetes plugin string: {}", kubernetes_plugin);

    if let Some(cache_dns_config) = cfg.cache_dns {
        if cache_dns_config.enable {
            upstream_servers.clear();

            // Automatic detection of upstream servers from the cluster
            if cache_dns_config.auto_detect {
                info!("\nAuto detecting servers");
                match detect_cluster_dns(kube_client.clone()).await {
                    Ok(detected_servers) => {
                        // Aggiungi i server rilevati alla lista upstream_servers
                        upstream_servers.extend(detected_servers);
                        info!("Auto detected servers: {:?}\n", upstream_servers);
                    }
                    Err(e) => {
                        // Gestisci l'errore se il rilevamento fallisce
                        error!("Failed to auto-detect servers: {}", e);
                    }
                }
            }
            
            // Aggiungi gli upstream servers configurati
            for server in &cache_dns_config.upstream_servers {
                let server = server.trim();
                if !server.is_empty() {
                    if is_valid_ip(server) {
                        upstream_servers.push(server.to_string());
                    } else {
                        error!("Invalid address: {}", server);
                    }
                }
            }

            // Rimuovi duplicati dagli upstream servers
            upstream_servers = remove_duplicates(upstream_servers);

            if upstream_servers.is_empty() {
                return Err(anyhow!("No valid upstream servers detected"));
            }

            // Aggiorna il TTL della cache
            cache_ttl = cache_dns_config.cache_ttl;
        }
    }

    // Crea la stringa di configurazione per il dominio stub
    let stub_domain_str = generate_stub_domain_block(StubDomainInfo {
        domain_name: "cortexflow-edge.dns".to_string(),
        local_ip: listen_ip.to_string(),
        port: cfg.listen_port.to_string(),
        cache_ttl,
        upstream_servers: upstream_servers.join(", "),
        kubernetes_plugin,
    })?;

    let stub_domain_str_copy = stub_domain_str.clone();

    // Scrivi la nuova configurazione nel file temporaneo
    let temp_corefile_path = "/tmp/Corefile";
    fs::write(temp_corefile_path, stub_domain_str)?;

    //Create a full patched file to check before submission in the k8s coredns file in corefile

    if let Some(coredns_data) = corefile_configmap.data.as_mut() {
        //search for corefile in data map
        if let Some(corefile) = coredns_data.get_mut("Corefile") {
            let mut corefile_copy = corefile.clone();

            //new config to add--> stub_domain_str
            corefile_copy.push_str(&stub_domain_str_copy); //copy trait implementare

            let patched_data = json!({
                "data":{
                    "Corefile": corefile_copy
                }
            });

            let temp_coredns_patch = "/tmp/PatchedCoreDns";
            fs::write(
                temp_coredns_patch,
                serde_json::to_string_pretty(&patched_data)?,
            )?;

            info!("CoreDNS updated successfully at {}", temp_coredns_patch);

            //apply the corefile patched file after user decision
            if corefile.contains("cortexflow-edge.dns:53") {
                error!("Configuration block already present, skipping update.");
            } else {
                warn!("Do you want to patch the coredns configuration? Yes[Y] No[N]");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if input.trim().eq_ignore_ascii_case("Y") {
                    info!("\nInserting patch:");
                    info!("{:?}\n",stub_domain_str_copy);
                    *corefile = format!("{}{}", corefile, stub_domain_str_copy);
                    

                    //send the patch to the cluster
                    let patch_data = json!({
                        "data": {
                            "Corefile": corefile.clone()
                        }
                    });
                    
                    let patch_new = Patch::Merge(patch_data);
                    
                    configmaps
                        .patch("coredns", &PatchParams::default(), &patch_new)
                        .await?;
                    

                    //TODO: add error handler

                    //logging
                    info!("Patched corefile successfully:\n");
                    info!("{:?}", corefile);
                } else {
                    //logging
                    error!("Corefile not patched");
                }
            }
        }
    }

    Ok(())
}

// Helper per ottenere la configurazione del plugin Kubernetes
fn get_kubernetes_plugin_str(cfg: EdgeDNSConfig) -> Result<String, Error> {
    // Logica per generare la stringa di configurazione del plugin Kubernetes
    if cfg.enable {
        let plugin_config = KubernetesPluginInfo {
            api_server: cfg
                .kube_api_config
                .as_ref()
                .and_then(|server| server.master.clone())
                .unwrap_or(" ".to_owned()),

            ttl: cfg
                .cache_dns
                .as_ref().map(|cache| cache.cache_ttl)
                .unwrap_or(DEFAULT_TTL),
        };
        generate_kubernetes_plugin_block(plugin_config)
    } else {
        Ok("".to_string()) // Nessun  plugin Kubernetes se non abilitato
    }
}
