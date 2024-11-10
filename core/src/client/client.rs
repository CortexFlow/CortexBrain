use kube::{Client as KubeClient, Api};
use kube::api::ListParams;
use k8s_openapi::api::core::v1::Pod;
use std::error::Error;
pub struct Client {
    config: String,  
    pub kube_client: KubeClient, 
}

impl Client {
    // create a new client istance-->act as a constructor
    pub async fn new_client(config: &str) -> Result<Self, Box<dyn Error>> {
        // Crea un client di Kubernetes
        let kube_client = KubeClient::try_default().await?;
        Ok(Client {
            config:config.to_string(),
            kube_client,
        })
    }

    // return the list of pods 
    pub async fn list_pods(&self, namespace: &str) -> Result<Vec<Pod>, Box<dyn Error>> {
        let pods: Api<Pod> = Api::namespaced(self.kube_client.clone(), namespace);
        let lp = ListParams::default();
        
        let pod_list = pods.list(&lp).await?;
        if pod_list.items.is_empty(){
            return Err(format!("No pods found").into());
        }
        Ok(pod_list.items)
    }
}

