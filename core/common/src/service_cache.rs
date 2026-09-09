use anyhow::Error;
use k8s_openapi::api::core::v1::Pod;
use kube::api::ObjectList;
use kube::{Api, Client};
use std::collections::HashMap;
use tracing::debug;

#[derive(Clone, Debug)]
pub struct ServiceCache {
    // service_map should maitain the Option Type. Since Option type can be also None
    // it's used to deal all the cases
    pub service_map: Option<HashMap<String, String>>,
}
impl ServiceCache {
    pub fn init(&mut self) {
        self.service_map = Some(HashMap::<String, String>::new());
    }

    /// Helper function to query all pods form the Kubernetes API
    pub async fn query_all_pods_from_kubeapi(&mut self) -> Result<ObjectList<Pod>, Error> {
        debug!("Connecting to kubernetes client");
        let client = Client::try_default().await?;
        debug!("Querying all the pods");
        let pods: Api<Pod> = Api::all(client);
        let lp = kube::api::ListParams::default(); // default list params
        let pod_list = pods.list(&lp).await?;

        Ok(pod_list)
    }

    /// Helper function to populate and mantain a cache service map
    pub async fn populate_map_with_pod_info(&mut self) -> Result<(), Error> {
        if Client::try_default().await.is_ok() {
            let all_pods = self.query_all_pods_from_kubeapi().await?;

            debug!("Querying and updating all the pods");
            for pod in all_pods {
                if let (Some(name), Some(uid), Some(annotations)) = (
                    pod.metadata.name,
                    pod.metadata.uid,
                    pod.metadata.annotations,
                ) {
                    // docs from kubernetes
                    // Annotations is an unstructured key value map stored with a resource that
                    // may be set by external tools to store and retrieve arbitrary metadata.
                    // They are not queryable and should be preserved when modifying objects.
                    //More info: https://kubernetes.io/docs/concepts/overview/working-with-objects/annotations

                    if let Some(map) = self.service_map.as_mut() {

                        // static pods are managed by the kubelet itself instead all the other pods 
                        // are managed by controllers without the need of the API server
                        // the kubelet mirrors the static pods but the UID does not match the real signature of the pod (verified)
                        // so the algorithm check if the pod is static by checking if the annotations contains the word 'file' or 'http' 
                        // reference for the labels: https://kubernetes.io/docs/reference/labels-annotations-taints/
                        // and copies the config.hash (aka the signature of the static pod) into the service cache instead of the UID

                        // more about static pods: https://medium.com/@iamsteffinissac/mastering-static-pods-in-kubernetes-bootstrapping-troubleshooting-mirror-pods-advanced-update-a35a5f95e329


                        let is_static_pod = annotations
                            .get("kubernetes.io/config.source")
                            .is_some_and(|a| a == "file" || a== "http");

                        let key = if is_static_pod {
                            annotations
                                .get("kubernetes.io/config.hash")
                                .cloned()
                                .unwrap_or(uid)
                        } else {
                            uid
                        };
                        map.insert(key, name);
                    }
                }
            } // insert the pod name and uid from the KubeAPI
        } else {
            debug!("Kubernetes environment not available");
        }

        Ok(())
    }
    pub async fn get_from_cache(&self, container_id: &str) -> Option<String> {
        // pass a container_id and returns the object meta infos:
        /* pub struct ObjectMeta {
            pub annotations: Option<BTreeMap<String, String>>,
            pub creation_timestamp: Option<Time>,
            pub deletion_grace_period_seconds: Option<i64>,
            pub deletion_timestamp: Option<Time>,
            pub finalizers: Option<Vec<String>>,
            pub generate_name: Option<String>,
            pub generation: Option<i64>,
            pub labels: Option<BTreeMap<String, String>>,
            pub managed_fields: Option<Vec<ManagedFieldsEntry>>,
            pub name: Option<String>,
            pub namespace: Option<String>,
            pub owner_references: Option<Vec<OwnerReference>>,
            pub resource_version: Option<String>,
            pub self_link: Option<String>,
            pub uid: Option<String>,
        } */

        if let Some(map) = self.service_map.as_ref() {
            map.get(container_id).cloned() // cache hit 
        } else {
            None
        }
    }
}
