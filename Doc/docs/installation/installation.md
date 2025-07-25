!!! warning
    CortexFlow is currently under active development and, therefore, is not yet available to the public for production use. If you are a developer and would like to contribute or try out CortexFlow, feel free to reach out to us.

###

# **Guided Installation**

CortexFlow provides a command line interface (CLI) to dynamically interact with all the CortexBrain services.

## **CLI Installation**
CortexFlow CLI is hosted using cargo package manager. The current vesion is **0.1.2**. The installation comes very easy thanks to cargo install

```bash
   cargo install cortexflow-cli
```

### **Components Installation**
Once you have installed the CLI you can run the following command to install **all** the CortexBrain components  
``` bash
   cfcli install
```
cfcli stores relevant information to interoperates with your cluster environment, up to now these information are limited to the cluster environment. 

During the installation you must enter your cluster environment, right now **Kubernetes** is the only supported container orchestrator:

!!! note
      CortexFlow aims to expand the current CortexBrain stack to support other container 
      orchestrator in the future

For a more comprehensive guide and internals you can read the [command list]() in the [CLI section]()


# **Manual Installation**
The manual installation allows users to install the CortexBrain components directly from the manifests' files. If you are a newbie this is not recommended. For newbies and users the recommended installation is the **Guided Installation** :

**Clone** the repository to your local machine.

```bash
   git clone https://github.com/CortexFlow/CortexBrain.git
```
**Install required packages**:

   | **Feature**              | **Requirements**                                                                 |
   | ------------------------- | -------------------------------------------------------------------------------- |
   | **CortexBrain Core**      | - Kubernetes or Minikube v1.34.0  <br> - Linux system with kernel version >= 5.15 (mandatory for core development)  <br> - Rust programming language (rustc >= 1.85.0) preferably a **nightly** version |
   | **CortexBrain Dashboard** | - npm v10.7.0  <br> - React v18.2.0  <br> - Electron v33.2.0                      |

## Getting Started for developers
Actually CortexBrain encorporates two major projects the **Core** and the **Dashboard**. Being open-source empowers CortexFlow ecosystem to naturally evolve and adapt to the latest challenges. If you are interested in mantaining the CortexFlow ecosystem or collaborate with the team, below you can find a detailed guide to get started.

### **Example of core development with Minikube**  

### **Environment Preparatives: Installing system requirements and dependencies**
CortexBrain contains the core CortexFlow functionalities and it's developed around the Linux kernel thanks to the eBPF technology. The get started with the core development you must install the following linux packages:

- bpftool
- bcc
- clang
- llvm
- libbpf-dev

For Debian/Ubuntu based distributions you can copy and paste the following command in your terminal:
```bash
   sudo apt install bpftool bcc clang llvm libbpf-dev
```

For RedHat/CentOS/Fedora based distributions you can copy and paste the following command in your terminal:
```bash
   sudo dnf install bpftool bcc clang llvm libbpf-dev
```


### **Minikube setup and core installation**
- Install Rust using RustUp tools : 
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```  
- Install [Docker](https://www.docker.com/get-started/)  
- Install [Minikube](https://minikube.sigs.k8s.io/docs/start/?arch=%2Fwindows%2Fx86-64%2Fstable%2F.exe+download)  
- Run minikube
   ```bash
   minikube start
   ```
- Install the core components using cfcli:
   ```bash
      cfcli install 
   ```

### **Dashboard Development**  

1.  Install [Node.js](https://nodejs.org/en/download)
2.  Open the dashboard folder and install the required packages 
   ```bash
      cd dashboard
      npm install 
   ```  
3.  Run the local development server
   ```bash
      npm start 
   ```
