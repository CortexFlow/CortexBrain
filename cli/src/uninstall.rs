use crate::essential::{Environments, get_config_directory, read_configs};
use colored::Colorize;
use std::io::stdin;
use std::process::Command;
use tracing::debug;

use std::thread;
use std::time::Duration;

pub fn uninstall() {
    //let file_path = get_config_directory().unwrap().1;
    //let dir_config_path = get_config_directory().unwrap().0;
    //debug!("file_path variable:{:?}", dir_config_path);
    //let env_from_file = read_configs(file_path.clone());
    //let user_env = Environments::try_from(env_from_file.to_lowercase());

    //match user_env {
    //    Ok(cluster_environment) => {
    let env = "kubectl".to_string();
    println!(
        "{} {}",
        "=====>".blue().bold(),
        "Uninstalling cortexflow..."
    );
    let mut userinput: String = String::new();
    println!("{} {}", "=====>".blue().bold(), "Select one option:");
    display_uninstall_options();
    stdin()
        .read_line(&mut userinput)
        .expect("Error reading user input");

    let trimmed_input = userinput.trim();
    if trimmed_input == "1" {
        uninstall_all(&env);
        println!(
            "{} {}",
            "=====>".blue().bold(),
            "Do you want to remove the command line metadata? [y/n]"
        );

        //clear the user input before assigning a new value
        userinput.clear();
        stdin()
            .read_line(&mut userinput)
            .expect("Error reading user input");

        if userinput.trim() == "y" {
            println!(
                "{} {}",
                "=====>".blue().bold(),
                "Deleting metadata config files"
            );
            //println!(
            //    "{} {}: {:?}",
            //    "=====>".blue().bold(),
            //    "Removing",
            //    dir_config_path.clone()
            //);
            //rm_dir(dir_config_path.as_os_str().to_str().unwrap());
        } else if userinput.trim() == "n" {
            println!(
                "{} {}",
                "=====>".blue().bold(),
                "Skipping metadata config files deletion"
            );
        }
    } else if trimmed_input == "2" {
        uninstall_component("deployment", "cortexflow-identity", &env.to_owned());
    }
}
// Err(e) => println!("An error occured while reading the config files: {}", e),
//}
//}

fn display_uninstall_options() {
    println!("{} {}", "=====>".blue().bold(), "1 > all");
    println!("{} {}", "=====>".blue().bold(), "2 > identity-service");
}

fn uninstall_all(env: &str) {
    println!(
        "{} {}",
        "=====>".blue().bold(),
        "Deleting cortexflow components".red()
    );
    //uninstall_component("namespace", "cortexflow", env);
    let output = Command::new(env)
        .args(["delete", "namespace", "cortexflow"])
        .output()
        .expect("Error deleting cortexflow namespace");

    if !output.status.success() {
        eprintln!(
            "Error deleting cortexflow namespace:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    } else {
        println!("✅ Removed cortexflow namespace");
    }
}

fn uninstall_component(component_type: &str, component: &str, env: &str) {
    println!(
        "{} {} {}",
        "=====>".blue().bold(),
        "Deleting service",
        component
    );
    let output = Command::new(env)
        .args(["delete", component_type, component, "-n", "cortexflow"])
        .output()
        .expect("Error deleting cortexflow-identity");

    if !output.status.success() {
        eprintln!(
            "Error deleting: {}:\n{}",
            component,
            String::from_utf8_lossy(&output.stderr)
        );
    } else {
        println!("✅ Removed component {}", component);
    }
}

fn rm_dir(directory_to_remove: &str) {
    let output = Command::new("rm")
        .args(["-rf", directory_to_remove])
        .output()
        .expect("cannot remove directory");

    if !output.status.success() {
        eprintln!(
            "Error removing directory: {}:\n{}",
            directory_to_remove,
            String::from_utf8_lossy(&output.stderr)
        );
    } else {
        println!("✅ Removed directory {}", directory_to_remove);
    }

    thread::sleep(Duration::from_secs(2));
}
