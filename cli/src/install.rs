use std::process::{ Command, exit };

use crate::essential::Environments;
use crate::essential::{ create_config_file, create_configs, get_config_directory, read_configs };

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
            println!("{} {}", "=====>".blue().bold(), "Copying installation files".white());
            copy_installation_files();
            thread::sleep(Duration::from_secs(1));
            println!("{} {}", "=====>".blue().bold(), "Creating cortexflow namespace".white());
            Command::new(env)
                .args(["create", "namespace", "cortexflow"])
                .output()
                .expect("Failed to create cortexflow namespace");

            install_components(env.to_string());
            println!("\n");
            rm_installation_files();
            println!("{} {}", "=====>".blue().bold(), "installation completed".white());
        }
        Err(e) => {
            eprintln!("An error occured while installing cortexflow components: {:?}", e);
            exit(1)
        }
    }
}

/* example installation function */
fn install_simple_example_component(env: String) {
    let user_env = Environments::try_from(env.to_lowercase());
    match user_env {
        Ok(cluster_environment) => {
            let env = cluster_environment.base_command();
            println!("{} {}", "=====>".blue().bold(), "Copying installation files".white());
            copy_example_installation_file();
            thread::sleep(Duration::from_secs(1));
            install_example(env.to_string());
            println!("\n");
            rm_example_installation_file();
            println!("{} {}", "=====>".blue().bold(), "installation completed".white());
        }
        Err(e) => {
            eprintln!("An error occured while installing cortexflow components: {:?}", e);
            exit(1)
        }
    }
}

/* main installation function */
pub fn install_cortexflow() {
    println!("{} {}", "=====>".blue().bold(), "Preparing cortexflow installation".white());
    println!("{} {}", "=====>".blue().bold(), "Creating the config files".white());
    let metadata_configs = create_configs();
    create_config_file(metadata_configs);

    let file_path = get_config_directory().unwrap().1;

    let env = read_configs(file_path);
    install_cluster_components(env);
}
/* install simple example */
pub fn install_simple_example() {
    println!("{} {}", "=====>".blue().bold(), "Installing simple example".white());

    let file_path = get_config_directory().unwrap().1;

    let env = read_configs(file_path);
    install_simple_example_component(env);
}


/* install example component */
fn install_example(env: String) {
    let files_to_install = vec!["deploy-test-pod.yaml"];
    let tot_files = files_to_install.len();

    println!("{} {}", "=====>".blue().bold(), "Installing cortexflow components".white());
    let user_env = env.as_str();
    debug!("Debugging env var in install components {:?}", user_env);

    let mut i = 1;

    for component in files_to_install {
        println!(
            "{} {}{}{}{} {} {} {}",
            "=====>".blue().bold(),
            "(",
            i,
            "/",
            tot_files,
            ")",
            "Applying ",
            component
        );
        apply_component(component, user_env);
        i = i + 1;
    }
}

/* Installation functions */
fn install_components(env: String) {
    let files_to_install = vec![
        "configmap.yaml",
        "configmap-role.yaml",
        "rolebinding.yaml",
        "cortexflow-rolebinding.yaml",
        "identity.yaml",
        "agent.yaml"
    ];
    let tot_files = files_to_install.len();

    println!("{} {}", "=====>".blue().bold(), "Installing cortexflow components".white());
    let user_env = env.as_str();
    debug!("Debugging env var in install components {:?}", user_env);

    let mut i = 1;

    for component in files_to_install {
        println!(
            "{} {}{}{}{} {} {} {}",
            "=====>".blue().bold(),
            "(",
            i,
            "/",
            tot_files,
            ")",
            "Applying ",
            component
        );
        apply_component(component, user_env);
        i = i + 1;
    }
}

fn apply_component(file: &str, env: &str) {
    let output = Command::new(env)
        .args(["apply", "-f", file])
        .output()
        .expect("cannot install component from file");

    if !output.status.success() {
        eprintln!("Error installing file: {}:\n{}", file, String::from_utf8_lossy(&output.stderr));
    } else {
        println!("✅ Applied {}", file);
    }

    thread::sleep(Duration::from_secs(2));
}

fn copy_installation_files() {
    download_file(
        "https://raw.githubusercontent.com/CortexFlow/CortexBrain/refs/heads/main/core/src/testing/configmap.yaml"
    );
    download_file(
        "https://raw.githubusercontent.com/CortexFlow/CortexBrain/refs/heads/main/core/src/testing/configmap-role.yaml"
    );
    download_file(
        "https://raw.githubusercontent.com/CortexFlow/CortexBrain/refs/heads/main/core/src/testing/rolebinding.yaml"
    );
    download_file(
        "https://raw.githubusercontent.com/CortexFlow/CortexBrain/refs/heads/main/core/src/testing/cortexflow-rolebinding.yaml"
    );
    download_file(
        "https://raw.githubusercontent.com/CortexFlow/CortexBrain/refs/heads/main/core/src/testing/identity.yaml"
    );
    download_file(
        "https://raw.githubusercontent.com/CortexFlow/CortexBrain/refs/heads/feature/ebpf-core/core/src/testing/agent.yaml"
    );
    println!("\n");
}
fn copy_example_installation_file() {
    download_file(
        "https://raw.githubusercontent.com/CortexFlow/CortexBrain/refs/heads/feature/ebpf-core/core/src/testing/deploy-test-pod.yaml"
    );
    println!("\n");
}
fn rm_installation_files() {
    println!("{} {}", "=====>".blue().bold(), "Removing temporary installation files".white());
    rm_file("configmap.yaml");
    rm_file("configmap-role.yaml");
    rm_file("rolebinding.yaml");
    rm_file("cortexflow-rolebinding.yaml");
    rm_file("identity.yaml");
    rm_file("agent.yaml");
}
fn rm_example_installation_file() {
    println!("{} {}", "=====>".blue().bold(), "Removing temporary installation files".white());
    rm_file("deploy-test-pod.yaml");
}

/* Auxiliary functions */
fn download_file(src: &str) {
    let output = Command::new("wget").args([src]).output().expect("cannot import config file");

    if !output.status.success() {
        eprintln!("Error copying file: {}.\n{}", src, String::from_utf8_lossy(&output.stderr));
    } else {
        println!("✅ Copied file from {} ", src);
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
