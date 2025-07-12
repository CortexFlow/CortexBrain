use std::process::{Command, exit};

use crate::essential::Environments;
use crate::essential::{create_config_file, create_configs, get_config_directory, read_configs};

use colored::Colorize;
use tracing::debug;

fn install_cluster_components(env: String) {
    let user_env = Environments::try_from(env.to_lowercase());
    match user_env {
        Ok(cluster_environment) => {
            let env = cluster_environment.base_command();
            println!(
                "{} {}",
                "=====>".blue().bold(),
                "Copying installation files".white()
            );
            copy_installation_files();

            println!(
                "{} {}",
                "=====>".blue().bold(),
                "Creating cortexflow namespace".white()
            );
            Command::new(env)
                .args(["create", "namespace", "cortexflow"])
                .output()
                .expect("Failed to create cortexflow namespace");

            install_components(env.to_string());
            println!("\n");
            rm_installation_files();
            println!(
                "{} {}",
                "=====>".blue().bold(),
                "installation completed".white()
            );
        }
        Err(e) => {
            eprintln!(
                "An error occured while installing cortexflow components: {:?}",
                e
            );
            exit(1)
        }
    }
}

pub fn install_cortexflow() {
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
    let metadata_configs = create_configs();
    create_config_file(metadata_configs);

    let file_path = get_config_directory().unwrap().1;

    let env = read_configs(file_path);
    install_cluster_components(env);
}

fn install_components(env: String) {
    println!(
        "{} {}",
        "=====>".blue().bold(),
        "Installing cortexflow components".white()
    );
    let user_env = env.as_str();
    debug!("Debugging env var in install components {:?}", user_env);
    println!(
        "{} {}",
        "=====>".blue().bold(),
        "(1/4) Applying configmap.yaml"
    );
    Command::new(user_env)
        .args(["apply", "-f", "configmap.yaml", "-n", "cortexflow"])
        .output()
        .expect("error");
    println!(
        "{} {}",
        "=====>".blue().bold(),
        "(2/4) Applying configmap-role.yaml"
    );
    Command::new(user_env)
        .args(["apply", "-f", "configmap-role.yaml", "-n", "default"])
        .output()
        .expect("error");
    println!(
        "{} {}",
        "=====>".blue().bold(),
        "(3/4) Applying rolebinding.yaml"
    );
    Command::new(user_env)
        .args(["apply", "-f", "rolebinding.yaml", "-n", "kube-system"])
        .output()
        .expect("error");
    println!(
        "{} {}",
        "=====>".blue().bold(),
        "(4/4) Applying cortexflow-rolebinding.yaml"
    );
    Command::new(user_env)
        .args(["apply", "-f", "cortexflow-rolebinding.yaml"])
        .output()
        .expect("error");
}
fn copy_installation_files() {
    Command::new("cp")
        .args(["../../core/src/testing/configmap.yaml", "configmap.yaml"])
        .output()
        .expect("cannot import configmap installation file");
    Command::new("cp")
        .args([
            "../../core/src/testing/configmap-role.yaml",
            "configmap-role.yaml",
        ])
        .output()
        .expect("cannot import configmap-role installation file");
    Command::new("cp")
        .args([
            "../../core/src/testing/rolebinding.yaml",
            "rolebinding.yaml",
        ])
        .output()
        .expect("cannot import rolebinding installation file");
    Command::new("cp")
        .args([
            "../../core/src/testing/cortexflow-rolebinding.yaml",
            "cortexflow-rolebinding.yaml",
        ])
        .output()
        .expect("cannot import rolebinding installation file");
    Command::new("cp")
        .args(["../../core/src/testing/identity.yaml", "identity.yaml"])
        .output()
        .expect("cannot import identity installation file");
}
fn rm_installation_files() {
    println!(
        "{} {}",
        "=====>".blue().bold(),
        "Removing temporary installation files".white()
    );
    Command::new("rm")
        .args(["-rf", "configmap.yaml"])
        .output()
        .expect("cannot remove configmap installation file");
    Command::new("rm")
        .args(["-rf", "configmap-role.yaml"])
        .output()
        .expect("cannot remove configmap-role installation file");
    Command::new("rm")
        .args(["-rf", "rolebinding.yaml"])
        .output()
        .expect("cannot remove rolebinding installation file");
    Command::new("rm")
        .args(["-rf", "cortexflow-rolebinding.yaml"])
        .output()
        .expect("cannot remove cortexflow-rolebinding installation file");
    Command::new("rm")
        .args(["-rf", "identity.yaml"])
        .output()
        .expect("cannot remove identity installation file");
}
