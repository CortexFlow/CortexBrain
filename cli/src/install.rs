use crate::essential::{
    BASE_COMMAND, CliError, connect_to_client, create_config_file, create_configs,
};
use clap::{Args, Subcommand, command};
use colored::Colorize;
use kube::Error;
use kube::core::ErrorResponse;
use std::{process::Command, thread, time::Duration};

// docs:
//
// Custom enum definition:
// InstallationType:
//      - used to pass installation files. Can be used for:
//          - Install components by passing a Vec<String> containing the components urls
//          - Install a simple-example by passing the component url (String)
//
//

enum InstallationType {
    Components(Vec<String>),
    SimpleExample(String),
}

// docs:
//
// main cortexflow installation function to install all the cortexflow components:
// This function creates the cortexflow namespace, manages the metadata file creation and removes the temporary installation files

#[derive(Subcommand, Debug, Clone)]
pub enum InstallCommands {
    #[command(
        name = "cortexflow",
        about = "Install all the CortexBrain core components"
    )]
    All,
    #[command(
        name = "simple-example",
        about = "Deploys a simple example contained in deploy-test-pod.yaml"
    )]
    TestPods,
}

//install args
#[derive(Args, Debug, Clone)]
pub struct InstallArgs {
    #[command(subcommand)]
    pub install_cmd: InstallCommands,
}

// docs:
//
// main cortexflow installation function to install all the cortexflow components:
// This function creates the cortexflow namespace, manages the metadata file creation and removes the temporary installation files

pub async fn install_cortexflow() -> Result<(), CliError> {
    println!(
        "{} {}",
        "=====>".blue().bold(),
        "Preparing cortexflow installation".white()
    );
    println!(
        "{} {}",
        "=====>".blue().bold(),
        "Creating the config files".white()
    );
    println!(
        "{} {}",
        "=====>".blue().bold(),
        "Creating cortexflow namespace".white()
    );
    Command::new("kubectl")
        .args(["create", "namespace", "cortexflow"])
        .output()
        .expect("Failed to create cortexflow namespace");

    let metadata_configs = create_configs();
    create_config_file(metadata_configs).await?;
    install_cluster_components().await?;
    Ok(())
}

// docs:
//
// main cortexflow installation function to install the examples:
// This function installs the demostration examples

pub async fn install_simple_example() -> Result<(), CliError> {
    println!(
        "{} {}",
        "=====>".blue().bold(),
        "Installing simple example".white()
    );
    install_simple_example_component().await?;
    Ok(())
}

//docs:
//
// This function manages the installation of the cortexflow cluster components
// Steps:
//      - Connects to kubernetes client
//      - Copies installation files from the offcial github repository
//      - Executes the install_components function
//      - Executes the rm_installation_files to remove the temporary installation files
//
// Returns an CliError if something fails

async fn install_cluster_components() -> Result<(), CliError> {
    match connect_to_client().await {
        Ok(_) => {
            println!(
                "{} {}",
                "=====>".blue().bold(),
                "Copying installation files".white()
            );
            download_installation_files(
                InstallationType::Components(
                    vec![
                        "https://raw.githubusercontent.com/CortexFlow/CortexBrain/refs/heads/main/core/src/testing/configmap-role.yaml".to_string(),
                        "https://raw.githubusercontent.com/CortexFlow/CortexBrain/refs/heads/main/core/src/testing/rolebinding.yaml".to_string(),
                        "https://raw.githubusercontent.com/CortexFlow/CortexBrain/refs/heads/main/core/src/testing/cortexflow-rolebinding.yaml".to_string(),
                        "https://raw.githubusercontent.com/CortexFlow/CortexBrain/refs/heads/main/core/src/testing/identity.yaml".to_string(),
                        "https://raw.githubusercontent.com/CortexFlow/CortexBrain/refs/heads/main/core/src/testing/agent.yaml".to_string(),
                        "https://raw.githubusercontent.com/CortexFlow/CortexBrain/refs/heads/main/core/src/testing/metrics.yaml".to_string()
                    ]
                )
            )?;
            thread::sleep(Duration::from_secs(1));
            install_components("cortexbrain")?;
            println!("\n");
            rm_installation_files(InstallationType::Components(vec![
                "configmap-role.yaml".to_string(),
                "rolebinding.yaml".to_string(),
                "cortexflow-rolebinding.yaml".to_string(),
                "identity.yaml".to_string(),
                "metrics.yaml".to_string(),
                "agent.yaml".to_string(),
            ]))?;
            println!(
                "{} {}",
                "=====>".blue().bold(),
                "installation completed".white()
            );
            Ok(())
        }
        Err(e) => Err(CliError::ClientError(Error::Api(ErrorResponse {
            status: "failed".to_string(),
            message: "Failed to connect to kubernetes client".to_string(),
            reason: "Your cluster is probably disconnected".to_string(),
            code: 404,
        }))),
    }
}

//docs:
//
// This function manages the installation of the examples
// Steps:
//      - Connects to kubernetes client
//      - Copies examples files from the offcial github repository
//      - Executes the install_example function
//      - Executes the rm_example_installation_file to remove the temporary installation files
//
// Returns an CliError if something fails

async fn install_simple_example_component() -> Result<(), CliError> {
    match connect_to_client().await {
        Ok(_) => {
            println!(
                "{} {}",
                "=====>".blue().bold(),
                "Copying installation files".white()
            );
            download_installation_files(
                InstallationType::SimpleExample(
                    "https://raw.githubusercontent.com/CortexFlow/CortexBrain/refs/heads/feature/ebpf-core/core/src/testing/deploy-test-pod.yaml".to_string()
                )
            )?;
            thread::sleep(Duration::from_secs(1));
            install_components("simple-example")?;
            println!("\n");
            rm_installation_files(InstallationType::SimpleExample(
                "deploy-test-pod.yaml".to_string(),
            ))?;
            println!(
                "{} {}",
                "=====>".blue().bold(),
                "installation completed".white()
            );
            Ok(())
        }
        Err(e) => Err(CliError::ClientError(Error::Api(ErrorResponse {
            status: "failed".to_string(),
            message: "Failed to connect to kubernetes client".to_string(),
            reason: "Your cluster is probably disconnected".to_string(),
            code: 404,
        }))),
    }
}

//docs:
//
// This is an auxiliary function to help manage the cortexflow components during the installation
// Steps:
//      - Read the Vec<&str> with the list of components to install
//      - Executes the apply_component function
//

fn install_components(components_type: &str) -> Result<(), CliError> {
    if components_type == "cortexbrain" {
        let files_to_install = vec![
            "configmap-role.yaml",
            "rolebinding.yaml",
            "cortexflow-rolebinding.yaml",
            "identity.yaml",
            "metrics.yaml",
            "agent.yaml",
        ];
        let tot_files = files_to_install.len();

        println!(
            "{} {}",
            "=====>".blue().bold(),
            "Installing cortexflow components".white()
        );
        let mut i = 1;

        for component in files_to_install {
            println!(
                "{} {}{}{}{}{} {} {}",
                "=====>".blue().bold(),
                "(",
                i,
                "/",
                tot_files,
                ")",
                "Applying",
                component.to_string().green().bold()
            );
            apply_component(component);
            i = i + 1;
        }
    } else if components_type == "simple-example" {
        let files_to_install = vec!["deploy-test-pod.yaml"];
        let tot_files = files_to_install.len();
        let mut i = 1;

        for component in files_to_install {
            println!(
                "{} {}{}{}{}{} {} {}",
                "=====>".blue().bold(),
                "(",
                i,
                "/",
                tot_files,
                ")",
                "Applying",
                component.to_string().green().bold()
            );
            apply_component(component);
            i = i + 1;
        }
    } else {
        return Err(CliError::InstallerError {
            reason: "No installation type selected".to_string(),
        });
    }
    Ok(())
}

//docs:
//
// This is an auxiliary function to help manage the cortexflow components during the installation
// Steps:
//      - Read the file name of a kubernetes manifest (e.g agent.yaml)
//      - Applies the manifest using the command kubectl apply -f <filename>
//
// Returns an CliError if something fails

fn apply_component(file: &str) -> Result<(), CliError> {
    let output = Command::new(BASE_COMMAND)
        .args(["apply", "-f", file])
        .output()
        .map_err(|_| CliError::InstallerError {
            reason: "Can't install component from file".to_string(),
        })?;

    if !output.status.success() {
        eprintln!(
            "Error installing file: {}:\n{}",
            file,
            String::from_utf8_lossy(&output.stderr)
        );
    } else {
        println!("✅ Applied {}", file);
    }
    thread::sleep(Duration::from_secs(2));
    Ok(())
}

//docs:
//
// This is an auxiliary function to download all the installation files
// Steps:
//      - Read the Vec<String> containing the file names of the installation files from the InstallationType enum
//      - Download the corresponding installation files from the github repository
//
// Returns an CliError if something fails

fn download_installation_files(installation_files: InstallationType) -> Result<(), CliError> {
    match installation_files {
        InstallationType::Components(files) => {
            for src in files.iter() {
                download_file(&src)?;
            }
        }
        InstallationType::SimpleExample(file) => {
            download_file(&file)?;
        }
    }
    println!("\n");
    Ok(())
}

//docs:
//
// This is an auxiliary function to specifically remove the installation files after the installation
// Steps:
//      - Read the Vec<String> containing the file names of the installation files from the InstallationType enum
//      - Executes the rm_file function for each installation file
//
// Returns an CliError if something fails

fn rm_installation_files(file_to_remove: InstallationType) -> Result<(), CliError> {
    println!(
        "{} {}",
        "=====>".blue().bold(),
        "Removing temporary installation files".white()
    );
    match file_to_remove {
        InstallationType::Components(files) => {
            for src in files.iter() {
                rm_file(&src)?;
            }
        }
        InstallationType::SimpleExample(file) => {
            rm_file(&file)?;
        }
    }

    Ok(())
}

//docs:
//
// This is an auxiliary function to help manage the cortexflow components during the installation
// Steps:
//      - Read the url name of a kubernetes manifest
//      - Download the manifest file from the cortexflow repository
//
// Returns a CliError if something fails

fn download_file(src: &str) -> Result<(), CliError> {
    let output =
        Command::new("wget")
            .args([src])
            .output()
            .map_err(|_| CliError::InstallerError {
                reason: "An error occured: component download failed".to_string(),
            })?;

    if !output.status.success() {
        eprintln!(
            "Error copying file: {}.\n{}",
            src,
            String::from_utf8_lossy(&output.stderr)
        );
    } else {
        println!("✅ Copied file from {} ", src);
    }
    thread::sleep(Duration::from_secs(2));
    Ok(())
}

//docs:
//
// This is an auxiliary function to help manage the cortexflow components during the installation
// Steps:
//      - Read the file name
//      - Removes the file using the rm -f <filename>
//
// Returns an CliError if something fails

fn rm_file(file_to_remove: &str) -> Result<(), CliError> {
    let output = Command::new("rm")
        .args(["-f", file_to_remove])
        .output()
        .map_err(|_| CliError::InstallerError {
            reason: "cannot remove temporary installation file".to_string(),
        })?;

    if !output.status.success() {
        eprintln!(
            "Error removing file: {}:\n{}",
            file_to_remove,
            String::from_utf8_lossy(&output.stderr)
        );
    } else {
        println!("✅ Removed file {}", file_to_remove);
    }

    thread::sleep(Duration::from_secs(2));
    Ok(())
}
