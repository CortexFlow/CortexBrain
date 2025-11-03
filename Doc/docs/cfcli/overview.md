# CortexFlow CLI

CortexFlow provides a command-line interface to interact with the **CortexBrain** core components via a gRPC API.  

The tool is called **`cfcli`**.  

This document describes the available commands and provides a quick reference table.


## 📦 Setup Commands

- **`cfcli install cortexflow`**  
  Installs **all** CortexBrain core components.  

- **`cfcli install simple-example`**  
  Installs a demo example defined in [deploy-test-pod.yaml](https://github.com/CortexFlow/CortexBrain/blob/main/core/src/testing/deploy-test-pod.yaml).  

- **`cfcli uninstall`** 
  Uninstalls **all** CortexBrain components.  


## ⚙️ CLI Management Commands

- **`cfcli update`**  
  Checks if the current `cfcli` version is up to date.  

- **`cfcli info`**  
  Displays CLI metadata, including:  
    - version  
    - authors  
    - description  
    - installation environment (Kubernetes, Docker, etc.)  


## 📝 Logging Commands

- **`cfcli logs`**  
  Retrieves logs for a specified pod.  


## 📊 Monitoring and Status Commands

- **`cfcli status`**  
  Performs a health check of the CortexBrain core:  
    - Validates if the `cortexflow` namespace exists.  
    - Returns the status of all core components.  

- **`cfcli monitoring list`**  
  Lists available CortexFlow agent endpoints.  
    - Useful for checking supported agent API functionalities.  
    - Returns an error if the agent is not running.  

- **`cfcli monitoring connections`**  
  Displays currently logged connections from the **Identity** service.  
    - Reads data from `events_map`.  
    - Shows the most recent detected events.  


## 📑 Command Reference Table

| Command                        | Category             | Description                                                                 |
|--------------------------------|----------------------|-----------------------------------------------------------------------------|
| `cfcli install cortexflow`     | Installation         | Installs all CortexBrain core components                                    |
| `cfcli install simple-example` | Installation         | Installs a demo example from `deploy-test-pod.yaml`                         |
| `cfcli uninstall`              | Installation         | Uninstalls all CortexBrain components                                       |
| `cfcli update`                 | CLI Management       | Checks if the CLI version is up to date                                     |
| `cfcli info`                   | CLI Management       | Displays version, authors, description, and environment metadata            |
| `cfcli logs`                   | Logging              | Retrieves logs for a specified pod                                          |
| `cfcli status`                 | Monitoring / Status  | Runs a health check and validates the `cortexflow` namespace                |
| `cfcli monitoring list`        | Monitoring / Status  | Lists CortexFlow agent endpoints                                            |
| `cfcli monitoring connections` | Monitoring / Status  | Displays logged connections from the Identity service                       |

---
