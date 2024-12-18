impl EdgeDNS {
    // Function to create a new instance of EdgeDNS based on the provided configuration
    pub fn new(c: Arc<EdgeDNSConfig>, clients: &Clients) -> Result<Self> {
        if !c.enable {
            return Ok(EdgeDNS { config: c });
        }

        // Update the Corefile (DNS) for the local node
        EdgeDNS::update_corefile(&c, clients)?;

        Ok(EdgeDNS { config: c })
    }

    // Update Corefile (DNS) in the Kubernetes cluster
    fn update_corefile(config: &EdgeDNSConfig, clients: &Clients) -> Result<()> {
        let kube_client = clients.get_kube_client();
        let namespace = &config.kube_namespace;

        // Simulating updating the ConfigMap for CoreDNS in Kubernetes
        let config_map_name = "coredns-config";
        let cm_api: kube::Api<kube::api::ConfigMap> = kube::Api::namespaced(kube_client.clone(), namespace);

        let patch = serde_json::json!({
            "data": {
                "Corefile": "updated Corefile contents here"
            }
        });

        cm_api.patch(config_map_name, &PatchParams::apply("edgedns"), &serde_json::to_vec(&patch)?)
            .map_err(|e| anyhow!("Failed to update Corefile ConfigMap: {}", e))?;

        info!("Corefile updated successfully in namespace {}", namespace);
        Ok(())
    }
}
