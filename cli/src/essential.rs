use std::{fs, io::stdin, path::PathBuf, process::exit};

use directories::ProjectDirs;
use serde::Serialize;
use std::fs::OpenOptions;

use colored::Colorize;
use std::time::Duration;
use std::thread;

pub struct GeneralData {
    env: String,
}
#[derive(Serialize)]
pub struct MetadataConfigFile {
    env: String,
}
#[derive(Debug)]
pub enum Environments {
    Kubernetes,
}
impl TryFrom<&str> for Environments {
    type Error = String;

    fn try_from(environment: &str) -> Result<Self, Self::Error> {
        match environment {
            "kubernetes" | "k8s" => Ok(Environments::Kubernetes),
            _ => Err(format!(
                "Environment '{}' not supported. Please insert a supported value: Kubernetes, K8s",
                environment
            )),
        }
    }
}

//for owned types
impl TryFrom<String> for Environments {
    type Error = String;

    fn try_from(environment: String) -> Result<Self, Self::Error> {
        Environments::try_from(environment.as_str())
    }
}

impl Environments {
    pub fn base_command(&self) -> &'static str {
        match self {
            Environments::Kubernetes => "kubectl",
        }
    }
}

impl GeneralData {
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");
    pub const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
    pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

    pub fn new(env: String) -> Self {
        GeneralData {
            env: env.to_string(),
        }
    }
    pub fn set_env(mut self, env: String) {
        self.env = env;
    }
    pub fn get_env(self) -> String {
        self.env
    }
    pub fn get_env_output(self) {
        println!("{:?}", self.env)
    }
}

pub fn update_cli() {
    println!("Updating CortexFlow CLI");
    println!("Looking for a newer version");
}
pub fn info(general_data: GeneralData) {
    println!("{} {} {}","=====>".blue().bold(),"Version:", GeneralData::VERSION);
    println!("{} {} {}","=====>".blue().bold(),"Author:", GeneralData::AUTHOR);
    println!("{} {} {}","=====>".blue().bold(),"Description:", GeneralData::DESCRIPTION);
    println!("{} {} {}","=====>".blue().bold(),"Environment:", general_data.get_env());
}

fn is_supported_env(env: &str) -> bool {
    matches!(env.to_lowercase().trim(), "kubernetes" | "k8s")
}

pub fn create_configs() -> MetadataConfigFile {
    let mut user_input: String = String::new();
    println!(
        "{} {}",
        "=====>".blue().bold(),
        "Insert your cluster environment (e.g. Kubernetes)".white()
    );
    stdin().read_line(&mut user_input).unwrap();
    let cluster_environment = user_input.trim().to_string();

    if !is_supported_env(&cluster_environment) {
        eprintln!(
            "Cannot save cluster environment data. Installation aborted. Please insert supported environment"
        );
        exit(1);
    }

    let configs = MetadataConfigFile {
        env: cluster_environment,
    };
    configs
}
pub fn read_configs(config_path: PathBuf) -> String {
    let config = fs::File::open(config_path).unwrap();
    let parsed_config: Result<serde_yaml::Value, serde_yaml::Error> =
        serde_yaml::from_reader(config);

    match parsed_config {
        Ok(cfg) => {
            let env = &cfg["env"].as_str().unwrap().to_string();
            thread::sleep(Duration::from_secs(1));
            println!(
                "{} {} {:?}",
                "[SYSTEM]".blue().bold(),
                "Readed configs for env variable:".white(),
                env
            );
            return env.to_string();
        }
        Err(e) => {
            eprintln!("An error occured while reading the config file: {:?}", e);
            exit(1)
        }
    }
}

pub fn create_config_file(config_struct: MetadataConfigFile) {
    let dirs = ProjectDirs::from("org", "cortexflow", "cfcli")
        .expect("Cannot determine the config directory");
    let config_dir = dirs.config_dir().to_path_buf();
    let config_save_path = config_dir.join("config.yaml");

    //create directory
    fs::create_dir_all(&config_dir).expect("Cannot create directories");

    let configs = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&config_save_path)
        .expect("Cannot open config file");

    match serde_yaml::to_writer(configs, &config_struct) {
        Ok(_) => {
            println!("\n");
            thread::sleep(Duration::from_secs(1));
            println!(
                "{} {}{:?}",
                "[SYSTEM]".blue().bold(),
                "Configuration files saved in path :".white(),
                &config_save_path.display()
            );
            println!("\n");
        }
        Err(e) => eprintln!(
            "An error occured during the creation of the config files. {:?}",
            e
        ),
    }
}
pub fn get_config_directory() -> Result<(PathBuf, PathBuf), ()> {
    let dirs = ProjectDirs::from("org", "cortexflow", "cfcli")
        .expect("Cannot determine the config directory");
    let config_dir = dirs.config_dir().to_path_buf();
    let file_path = config_dir.join("config.yaml");

    Ok((config_dir, file_path))
}

pub fn get_startup_config_dir() -> bool {
    ProjectDirs::from("org", "cortexflow", "cfcli")
        .map(|dirs| {
            let path = dirs.config_dir();
            path.exists()
        })
        .unwrap_or(false)
}
