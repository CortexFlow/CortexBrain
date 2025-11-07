use std::{ collections::BTreeMap, fmt, process::Command, result::Result::Ok };

use kube::core::ErrorResponse;
use serde::Serialize;
use colored::Colorize;

use k8s_openapi::api::core::v1::ConfigMap;
use k8s_openapi::serde_json::json;
use kube::{ Config, Error };
use kube::api::{ Api, ObjectMeta, Patch, PatchParams, PostParams };
use kube::client::Client;

pub static BASE_COMMAND: &str = "kubectl"; // docs: Kubernetes base command

// docs:
//
// Custom enum definition to group all the installation error for cortexflow
//

pub enum CliError {
    InstallerError {
        reason: String,
    },
    ClientError(kube::Error),
    UninstallError {
        reason: String,
    },
}
impl From<kube::Error> for CliError {
    fn from(e: Error) -> Self {
        CliError::ClientError(e)
    }
}

// docs:
//
// Custom error definition
// InstallerError:
//      - used for general installation errors occured during the installation of cortexflow components. Can be used for:
//          - Return downloading errors
//          - Return unsuccessful file removal
//
//
// implements fmt::Display for user-friendly error messages
//

#[derive(Debug, Clone)]
pub struct InstallerError {
    pub(crate) reason: String,
}

impl fmt::Display for InstallerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "An error occured while installing cortexflow components. Reason: {}",
            self.reason
        );
        Ok(())
    }
}

#[derive(Serialize)]
pub struct MetadataConfigFile {
    blocklist: Vec<String>,
}

// docs:
//
// This is a wrapper functions used to create a kubernetes client session
// Used in modules:
//      - essentials
//      - install
//      - logs
//      - service
//      - status
//      - uninstall
//
//
// Returns a Result with the client an a kube::Error

pub async fn connect_to_client() -> Result<Client, kube::Error> {
    let client = Client::try_default().await;
    client
}

// docs:
//
// This is an function used to update the cli
//
// Steps:
//      - Checks the current CLI version
//      - if the version matches the current latest version doesn't do anything
//      - else runs the cargo update command
//
// Returns an error if the command fails

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

// docs:
//
// This is a function to display the CLI Version,Author and Description using a fancy output style

pub fn info() {
    println!("{} {} {}", "=====>".blue().bold(), "Version:", env!("CARGO_PKG_VERSION"));
    println!("{} {} {}", "=====>".blue().bold(), "Author:", env!("CARGO_PKG_AUTHORS"));
    println!("{} {} {}", "=====>".blue().bold(), "Description:", env!("CARGO_PKG_DESCRIPTION"));
}

// docs:
//
// This is a wrapper function to create the MetadataConfigFile structure

pub fn create_configs() -> MetadataConfigFile {
    let mut blocklist: Vec<String> = Vec::new();
    blocklist.push("".to_string());

    let configs = MetadataConfigFile { blocklist };
    configs
}

// docs:
//
// This is an helper functions used to read the configs from a kubernetes configmap
//
// Steps:
//      - connects to kubernetes client
//      - read the configmap from the kubernetes API. Needed: namespace_name , configmap_name
//      - returns the given configmap blocklist data in a Vec<String> type
//
// Returns an error if something fails

pub async fn read_configs() -> Result<Vec<String>, CliError> {
    match connect_to_client().await {
        Ok(client) => {
            let namespace = "cortexflow";
            let configmap = "cortexbrain-client-config";
            let api: Api<ConfigMap> = Api::namespaced(client, namespace);

            let cm = api.get(configmap).await?;

            if let Some(data) = cm.data {
                if let Some(blocklist_raw) = data.get("blocklist") {
                    let lines: Vec<String> = blocklist_raw
                        .lines()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty()) // ignore empty lines
                        .collect();

                    return Ok(lines);
                }
            }

            Ok(Vec::new()) //in case the key fails
        }
        Err(_) => {
            Err(
                CliError::ClientError(
                    Error::Api(ErrorResponse {
                        status: "failed".to_string(),
                        message: "Failed to connect to kubernetes client".to_string(),
                        reason: "Your cluster is probably disconnected".to_string(),
                        code: 404,
                    })
                )
            )
        }
    }
}

// docs:
//
// This is a function used to create a configmap file
//
// With the version 0.1.4 cortexflow introduced a configmap file to store the relevant cortexflow metadata
// Up to now the metadata includes:
//      - blocked ip addresses passed using the CLI
//
// Steps:
//      - connects to kubernetes client
//      - creates a configmap named "cortexbrain-client-config" stored in the cortexflow namespace
//      - the blocklist field is initialized with zero blocked addresses
//
// Returns an error if something fails

pub async fn create_config_file(config_struct: MetadataConfigFile) -> Result<(), CliError> {
    match connect_to_client().await {
        Ok(client) => {
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
            }
            Ok(())
        }
        Err(_) => {
            Err(
                CliError::ClientError(
                    Error::Api(ErrorResponse {
                        status: "failed".to_string(),
                        message: "Failed to connect to kubernetes client".to_string(),
                        reason: "Your cluster is probably disconnected".to_string(),
                        code: 404,
                    })
                )
            )
        }
    }
}

// docs:
//
// This is a function used to update a configmap file. Takes an input and an action
//
// Input: an ip (&str type)
// Actions:
//      - Add: add the ip to the blocklist metadata
//      - Delete: remove the ip from the blocklist metadata
//
// Steps:
//      - connects to kubernetes client
//      - reads the existing configmap
//      - creates a temporary vector with the old addresses and the new address
//      - creates a patch by calling the update_configamp file
//
// Returns an error if something fails

pub async fn update_config_metadata(input: &str, action: &str) -> Result<(), CliError> {
    if action == "add" {
        //retrieve current blocked ips list
        let mut ips = read_configs().await?;
        println!("Readed current blocked ips: {:?}", ips);

        //create a temporary vector of ips
        ips.push(input.to_string());

        // override blocklist parameters
        let new_configs = MetadataConfigFile { blocklist: ips };
        //create a new config
        update_configmap(new_configs).await?;
    } else if action == "delete" {
        let mut ips = read_configs().await?;
        if let Some(index) = ips.iter().position(|target| target == &input.to_string()) {
            ips.remove(index);
        } else {
            eprintln!("Index of element not found");
        }

        // override blocklist parameters
        let new_configs = MetadataConfigFile { blocklist: ips };
        //create a new config
        update_configmap(new_configs).await?;
    }
    Ok(())
}

// docs:
//
// This is a function used to create a patch to update a configmap
//
// Steps:
//      - connects to kubernetes client
//      - creates a patch using the config_struct data
//      - pushes the patch to the kubernetes API
//
// Returns an error if something fails

pub async fn update_configmap(config_struct: MetadataConfigFile) -> Result<(), CliError> {
    match connect_to_client().await {
        Ok(client) => {
            let namespace = "cortexflow";
            let name = "cortexbrain-client-config";
            let api: Api<ConfigMap> = Api::namespaced(client, namespace);

            let blocklist_yaml = config_struct.blocklist
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<String>>()
                .join("\n");

            let patch = Patch::Apply(
                json!({
                    "apiVersion": "v1",
                    "kind": "ConfigMap",
                    "data": {
                        "blocklist": blocklist_yaml
                    }
                })
            );

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
        Err(_) => {
            Err(
                CliError::ClientError(
                    Error::Api(ErrorResponse {
                        status: "failed".to_string(),
                        message: "Failed to connect to kubernetes client".to_string(),
                        reason: "Your cluster is probably disconnected".to_string(),
                        code: 404,
                    })
                )
            )
        }
    }
}
