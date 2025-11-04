use std::collections::BTreeMap;
use std::ptr::read;
//TODO: Check if is possible to use the get_config_path function. Check for reusable components
use std::{fs, io::stdin, path::PathBuf, process::exit};

use directories::ProjectDirs;
use k8s_openapi::api::core::v1::ConfigMap;
use k8s_openapi::serde_json::json;
use kube::Config;
use prost_types::MethodDescriptorProto;
use serde::Serialize;
use std::fs::{Metadata, OpenOptions};
use std::result::Result::Ok;

use colored::Colorize;
use std::thread;
use std::time::Duration;

use std::process::Command;

use kube::api::{Api, ObjectMeta, Patch, PatchParams, PostParams};
use kube::client::Client;

//pub struct GeneralData {
//    env: String,
//}
#[derive(Serialize)]
pub struct MetadataConfigFile {
    blocklist: Vec<String>,
}

//FIXME: remove this part
//#[derive(Debug)]
//pub enum Environments {
//    Kubernetes,
//}
//impl TryFrom<&str> for Environments {
//    type Error = String;
//
//    fn try_from(environment: &str) -> Result<Self, Self::Error> {
//        match environment {
//            "kubernetes" | "k8s" => Ok(Environments::Kubernetes),
//            _ =>
//                Err(
//                    format!("Environment '{}' not supported. Please insert a supported value: Kubernetes, K8s", environment)
//                ),
//        }
//    }
//}

//for owned types
//impl TryFrom<String> for Environments {
//    type Error = String;
//
//    fn try_from(environment: String) -> Result<Self, Self::Error> {
//        Environments::try_from(environment.as_str())
//    }
//}

//impl Environments {
//    pub fn base_command(&self) -> &'static str {
//        match self {
//            Environments::Kubernetes => "kubectl",
//        }
//    }
//}

//impl GeneralData {
    //pub const VERSION: &str = env!("CARGO_PKG_VERSION");
    //pub const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
    //pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

    //pub fn new(env: String) -> Self {
    //    GeneralData {
    //        env: env.to_string(), // FIXME: remove this field
    //    }
    //}
    //pub fn set_env(mut self, env: String) {
    //    self.env = env;
    //}
    //pub fn get_env(self) -> String {
    //    self.env
    //}
    //pub fn get_env_output(self) {
    //   println!("{:?}", self.env)
    //}
//}

pub async fn connect_to_client() -> Result<Client, kube::Error> {
    let client = Client::try_default().await;
    client
}


pub fn update_cli() {
    println!("{} {}", "=====>".blue().bold(), "Updating CortexFlow CLI");
    println!("{} {}", "=====>".blue().bold(), "Looking for a newer version");

    let output = Command::new("cargo").args(["update", "cortexflow-cli"]).output().expect("error");

    if !output.status.success() {
        eprintln!("Error updating CLI : {}", String::from_utf8_lossy(&output.stderr));
    } else {
        println!("✅ Updated CLI");
    }
}
//pub fn info(general_data: GeneralData) {
//    println!("{} {} {}", "=====>".blue().bold(), "Version:", GeneralData::VERSION);
//    println!("{} {} {}", "=====>".blue().bold(), "Author:", GeneralData::AUTHOR);
//    println!("{} {} {}", "=====>".blue().bold(), "Description:", GeneralData::DESCRIPTION);
//      println!("{} {} {}", "=====>".blue().bold(), "Environment:", general_data.get_env()); // FIXME: remove this field
//}

pub fn create_configs() -> MetadataConfigFile {
    let mut blocklist: Vec<String> = Vec::new();
    blocklist.push("".to_string());

    let configs = MetadataConfigFile { blocklist };
    configs
}
pub async fn read_configs() -> Result<Vec<String>, anyhow::Error> {
    let client = Client::try_default().await?;
    let namespace = "cortexflow";
    let configmap = "cortexbrain-client-config";
    let api: Api<ConfigMap> = Api::namespaced(client, namespace);

    let cm = api.get(configmap).await?;

    if let Some(data) = cm.data {
        if let Some(blocklist_raw) = data.get("blocklist") {
            let lines: Vec<String> = blocklist_raw
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()) // ignora righe vuote
                .collect();

            return Ok(lines);
        }
    }

    Ok(Vec::new()) //in case the key fails
}
pub async fn create_config_file(config_struct: MetadataConfigFile) -> Result<(), anyhow::Error> {
    let client = Client::try_default().await?;
    let namespace = "cortexflow";
    let configmap = "cortexbrain-client-config";

    let api: Api<ConfigMap> = Api::namespaced(client, namespace);

    // create configmap
    let mut data = BTreeMap::new();
    for x in config_struct.blocklist {
        data.insert("blocklist".to_string(), x);
    }
    let cm = ConfigMap {
        metadata: ObjectMeta {
            name: Some("cortexbrain-client-config".to_string()),
            ..Default::default()
        }, // type ObjectMeta
        data: Some(data), //type Option<BTreeMap<String, String, Global>>
        ..Default::default()
    };
    match api.create(&PostParams::default(), &cm).await {
        Ok(_) => {
            println!("Configmap created successfully");
        }
        Err(e) => {
            eprintln!("An error occured: {}", e);
        }
    };
    Ok(())
}

pub async fn update_config_metadata(input: &str, action: &str) {
    if action == "add" {
        //retrieve current blocked ips list
        let mut ips = read_configs().await.unwrap();
        println!("Readed current blocked ips: {:?}", ips);

        //create a temporary vector of ips
        ips.push(input.to_string());

        // override blocklist parameters
        let new_configs = MetadataConfigFile { blocklist: ips };
        //create a new config
        update_configmap(new_configs).await;
    } else if action == "delete" {
        let mut ips = read_configs().await.unwrap();
        if let Some(index) = ips.iter().position(|target| target == &input.to_string()) {
            ips.remove(index);
        } else {
            eprintln!("Index of element not found");
        }

        // override blocklist parameters
        let new_configs = MetadataConfigFile { blocklist: ips };
        //create a new config
        update_configmap(new_configs).await;
    }
}

pub async fn update_configmap(config_struct: MetadataConfigFile) -> Result<(), anyhow::Error> {
    let client = Client::try_default().await?;
    let namespace = "cortexflow";
    let name = "cortexbrain-client-config";
    let api: Api<ConfigMap> = Api::namespaced(client, namespace);

    let blocklist_yaml = config_struct
        .blocklist
        .iter()
        .map(|x| format!("{}", x))
        .collect::<Vec<String>>()
        .join("\n");

    let patch = Patch::Apply(json!({
        "apiVersion": "v1",
        "kind": "ConfigMap",
        "data": {
            "blocklist": blocklist_yaml
        }
    }));

    let patch_params = PatchParams::apply("cortexbrain").force();
    match api.patch(name, &patch_params, &patch).await {
        Ok(_) => {
            println!("Map updated successfully");
        }
        Err(e) => {
            eprintln!("An error occured during the patching process: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
