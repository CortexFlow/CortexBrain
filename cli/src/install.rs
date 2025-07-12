use std::process::{Command, exit};

use crate::essential::Environments;
use crate::essential::{create_config_file, create_configs, get_config_directory, read_configs};

use colored::Colorize;
use std::thread;
use std::time::Duration;
use tracing::debug;

/* components installation function */
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
            thread::sleep(Duration::from_secs(1));
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

/* main installation function */
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

/* Installation functions */
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
        "(1/5) Applying configmap.yaml"
    );
    thread::sleep(Duration::from_secs(2));
    Command::new(user_env)
        .args(["apply", "-f", "configmap.yaml", "-n", "cortexflow"])
        .output()
        .expect("error");
    println!(
        "{} {}",
        "=====>".blue().bold(),
        "(2/5) Applying configmap-role.yaml"
    );
    thread::sleep(Duration::from_secs(2));
    Command::new(user_env)
        .args(["apply", "-f", "configmap-role.yaml", "-n", "default"])
        .output()
        .expect("error");
    println!(
        "{} {}",
        "=====>".blue().bold(),
        "(3/5) Applying rolebinding.yaml"
    );
    thread::sleep(Duration::from_secs(2));
    Command::new(user_env)
        .args(["apply", "-f", "rolebinding.yaml", "-n", "kube-system"])
        .output()
        .expect("error");
    thread::sleep(Duration::from_secs(2));
    println!(
        "{} {}",
        "=====>".blue().bold(),
        "(4/5) Applying cortexflow-rolebinding.yaml"
    );
    Command::new(user_env)
        .args(["apply", "-f", "cortexflow-rolebinding.yaml"])
        .output()
        .expect("error");
    println!(
        "{} {}",
        "=====>".blue().bold(),
        "(5/5) Installing Identity service.yaml"
    );
    thread::sleep(Duration::from_secs(2));
    Command::new(user_env)
        .args(["apply", "-f", "identity.yaml"])
        .output()
        .expect("error");
    thread::sleep(Duration::from_secs(2));
}
fn copy_installation_files() {
    copy_file("../core/src/testing/configmap.yaml", "configmap.yaml");
    copy_file("../core/src/testing/configmap-role.yaml", "configmap-role.yaml");
    copy_file("../core/src/testing/rolebinding.yaml", "rolebinding.yaml");
    copy_file(
        "../core/src/testing/cortexflow-rolebinding.yaml",
        "cortexflow-rolebinding.yaml",
    );
    copy_file("../core/src/testing/identity.yaml", "identity.yaml");
    println!("\n");
}
fn rm_installation_files() {
    println!(
        "{} {}",
        "=====>".blue().bold(),
        "Removing temporary installation files".white()
    );
    rm_file("configmap.yaml");
    rm_file("configmap-role.yaml");
    rm_file("rolebinding.yaml");
    rm_file("cortexflow-rolebinding.yaml");
    rm_file("identity.yaml");
}


/* Auxiliary functions */
fn copy_file(src: &str, dest: &str) {
    let output = Command::new("cp")
        .args([src, dest])
        .output()
        .expect("cannot import config file");

    if !output.status.success() {
        eprintln!(
            "Error copying file: {} -> {}:\n{}",
            src,
            dest,
            String::from_utf8_lossy(&output.stderr)
        );
    } else {
        println!("✅ Copied file from {} → {}", src, dest);
    }

    thread::sleep(Duration::from_secs(2));
}
fn rm_file(file_to_remove: &str) {
    let output = Command::new("rm")
        .args(["-f", file_to_remove])
        .output()
        .expect("cannot remove temporary installation file");

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
}
