use std::borrow::Cow;
use std::thread;
use std::time::Duration;
use std::{collections::BTreeMap, fmt, process::Command, result::Result::Ok};

use anyhow::Error;
use colored::Colorize;
use kube::core::ErrorResponse;
use serde::Serialize;

use k8s_openapi::api::core::v1::ConfigMap;
use k8s_openapi::serde_json::json;
use kube::api::{Api, ObjectMeta, Patch, PatchParams, PostParams};
use kube::client::Client;

pub static BASE_COMMAND: &str = "kubectl"; // docs: Kubernetes base command

// docs:
//
// CliError enum to group all the errors
//
// Custom error definition
// InstallerError:
//      - used for general installation errors occured during the installation of cortexflow components. Can be used for:
//          - Return downloading errors
//          - Return unsuccessful file removal during installation
//
// ClientError:
//      - used for Kubernetes client errors. Can be used for:
//          - Return client connection errors
//
// UninstallError:
//      - used for general installation errors occured during the uninstall for cortexflow components. Can be used for:
//          -  Return components removal errors
//
// AgentError:
//      - used for cortexflow agent errors. Can be used for:
//          - return errors from the reflection server
//          - return unavailable agent errors (404)
//
// MonitoringError:
//      - used for general monitoring errors. TODO: currently under implementation
//
// implements fmt::Display for user friendly error messages

#[derive(Debug)]
pub enum CliError {
    InstallerError { reason: String },
    ClientError(kube::Error),
    UninstallError { reason: String },
    AgentError(tonic_reflection::server::Error),
    MonitoringError { reason: String },
}
// docs:
// error type conversions

impl From<kube::Error> for CliError {
    fn from(e: kube::Error) -> Self {
        CliError::ClientError(e)
    }
}
impl From<anyhow::Error> for CliError {
    fn from(e: anyhow::Error) -> Self {
        CliError::MonitoringError {
            reason: format!("{}", e),
        }
    }
}
impl From<()> for CliError {
    fn from(e: ()) -> Self {
        return ().into();
    }
}
impl From<prost::DecodeError> for CliError {
    fn from(e: prost::DecodeError) -> Self {
        todo!()
    }
}
impl From<tonic::Status> for CliError {
    fn from(e: tonic::Status) -> Self {
        todo!()
    }
}

// docs:
// fmt::Display implementation for CliError type. Creates a user friendly message error message.
// TODO: implement colored messages using the colorize crate for better output display

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::InstallerError { reason } => {
                write!(
                    f,
                    "{} {} {}",
                    "=====>".blue().bold(),
                    "An error occured while installing cortexflow components. Reason:"
                        .bold()
                        .red(),
                    reason
                )
            }
            CliError::UninstallError { reason } => {
                write!(
                    f,
                    "An error occured while installing cortexflow components. Reason: {}",
                    reason
                )
            }
            CliError::MonitoringError { reason } => {
                write!(
                    f,
                    "An error occured while installing cortexflow components. Reason: {}",
                    reason
                )
            }
            CliError::ClientError(e) => write!(f, "Client Error: {}", e),
            CliError::AgentError(e) => {
                write!(
                    f,
                    "{} {} {}",
                    "=====>".bold().blue(),
                    "Agent Error:".bold().red(),
                    e
                )
            }
        }
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
    let latest_version = get_latest_cfcli_version().expect("Can't get the latest version");
    println!("{} {}", "=====>".blue().bold(), "Updating CortexFlow CLI");
    println!(
        "{} {}",
        "=====>".blue().bold(),
        "Looking for a newer version \n "
    );

    let output = Command::new("cfcli")
        .args(["--version"])
        .output()
        .expect("error");

    if !output.status.success() {
        eprintln!(
            "Error extracting the version : {}",
            String::from_utf8_lossy(&output.stderr)
        );
    } else {
        // extract the cli version:
        let version = String::from_utf8_lossy(&output.stdout)
            .split_whitespace()
            .last()
            .expect("An error occured during the version extraction")
            .to_string();

        if version == latest_version {
            println!(
                "{} {} {} {} {} {}{}",
                "=====>".blue().bold(),
                "Your version".green().bold(),
                (&version.to_string()).green().bold(),
                "is already up to date".green().bold(),
                "(latest:".green().bold(),
                (&latest_version).green().bold(),
                ")\n".green().bold()
            );
        } else {
            println!(
                "{} {} {} {} {} {}{}",
                "=====>".blue().bold(),
                "Your version".red().bold(),
                (&version.to_string()).red().bold(),
                "needs to be updated".red().bold(),
                "(latest:".red().bold(),
                (&latest_version).red().bold(),
                ")\n".red().bold()
            );
            thread::sleep(Duration::from_secs(1));
            println!("{} {}", "=====>".blue().bold(), "Updating the CLI...");
            let update_command = Command::new("cargo")
                .args(["install", "cortexflow-cli", "--force"])
                .output()
                .expect("error");
            if !update_command.status.success() {
                eprintln!(
                    "Error updating the CLI: {} ",
                    String::from_utf8_lossy(&update_command.stderr)
                );
            } else {
                println!(
                    "{} {}",
                    "=====>".blue().bold(),
                    "CLI updated".green().bold()
                )
            }
        }
    }
}

// docs:
//
// This function returns the latest version of the CLI from the crates.io registry
// TODO: implement CliError here
pub fn get_latest_cfcli_version() -> Result<String, CliError> {
    let output = Command::new("cargo")
        .args(["search", "cortexflow-cli", "--limit", "1"])
        .output()
        .expect("Error");

    if !output.status.success() {
        return Err(CliError::InstallerError {
            reason: "Cannot extract the latest version".to_string(),
        });
    } else {
        let command_stdout = String::from_utf8_lossy(&output.stdout);

        // here the data output have this structure:
        // cortexflow-cli = "0.1.4"    # CortexFlow command line interface made to interact with the CortexBrain core components
        // ... and 3 crates more (use --limit N to see more)

        // i need to extract only the version tag
        let version = extract_version_from_output(command_stdout);

        Ok(version)
    }
}

// docs:
// this is an helper function used in a unit test
//
// Takes a Clone-On-Write (Cow) smart pointer (the same type returned by the String::from_utf8_lossy(&output.stdout) code )
// and returns a String that contains the cfcli version

fn extract_version_from_output(command_stdout: Cow<'_, str>) -> String {
    let version = command_stdout.split('"').nth(1).unwrap().to_string();
    version
}

// docs:
//
// This is a function to display the CLI Version,Author and Description using a fancy output style

pub fn info() {
    println!(
        "{} {} {}",
        "=====>".blue().bold(),
        "Version:",
        env!("CARGO_PKG_VERSION")
    );
    println!(
        "{} {} {}",
        "=====>".blue().bold(),
        "Author:",
        env!("CARGO_PKG_AUTHORS")
    );
    println!(
        "{} {} {}",
        "=====>".blue().bold(),
        "Description:",
        env!("CARGO_PKG_DESCRIPTION")
    );
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
        Err(e) => Err(CliError::ClientError(kube::Error::Api(ErrorResponse {
            status: "failed".to_string(),
            message: "Failed to connect to kubernetes client".to_string(),
            reason: e.to_string(),
            code: 404,
        }))),
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
            //let configmap = "cortexbrain-client-config";

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
        Err(e) => Err(CliError::ClientError(kube::Error::Api(ErrorResponse {
            status: "failed".to_string(),
            message: "Failed to connect to kubernetes client".to_string(),
            reason: e.to_string(),
            code: 404,
        }))),
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
        Err(e) => Err(CliError::ClientError(kube::Error::Api(ErrorResponse {
            status: "failed".to_string(),
            message: "Failed to connect to kubernetes client".to_string(),
            reason: e.to_string(),
            code: 404,
        }))),
    }
}

#[cfg(test)]
mod tests {
    use crate::essential::extract_version_from_output;

    #[test]
    fn test_version_extraction() {
        let command_stdout = String::from(
            r#"cortexflow-cli = "0.1.4-test_123"    
            # CortexFlow command line interface made to interact with the CortexBrain core components...
             and 3 crates more (use --limit N to see more)"#,
        );

        let extracted_command = extract_version_from_output(command_stdout.into());
        assert_eq!(extracted_command, "0.1.4-test_123");
    }
}
