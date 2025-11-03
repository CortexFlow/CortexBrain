# CortexBrain © 2025 

<p align="center">
  <a href="https://github.com/CortexFlow/CortexBrain/releases">
    <img src="https://img.shields.io/badge/Release-Currently%20under%20development-red?style=flat-square&logo=github" alt="Release">
  </a>
  <a href="./LICENSE">
    <img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg?style=flat-square&logo=open-source-initiative&logoColor=white" alt="License">
  </a>
  <a href="https://www.cortexflow.org/doc/">
    <img src="https://img.shields.io/badge/Documentation-Available-brightgreen?style=flat-square&logo=readthedocs&logoColor=white" alt="Documentation">
  </a>
  <a href="https://www.docker.com">
    <img src="https://img.shields.io/badge/Docker-Containerized-%232496ED.svg?style=flat-square&logo=docker&logoColor=white" alt="Docker">
  </a>
  <a href="https://kubernetes.io">
    <img src="https://img.shields.io/badge/Kubernetes-Orchestrator-%23326CE5.svg?style=flat-square&logo=Kubernetes&logoColor=white" alt="Kubernetes">
  </a>
  <a href="https://github.com/CortexFlow/CortexBrain/discussions">
    <img src="https://img.shields.io/github/discussions/CortexFlow/CortexBrain?style=flat-square&logo=github-discussions&logoColor=white" alt="Discussions">
  </a>
  <a href="https://github.com/CortexFlow/CortexBrain#contributing">
    <img src="https://img.shields.io/badge/Contributors-Welcome-brightgreen?style=flat-square&logo=github&logoColor=white" alt="Contributors">
  </a>
  <a href="https://dev.to/cortexflow">
    <img src="https://img.shields.io/badge/dev.to-Community-%23326CE5.svg?style=flat-square&logo=Dev.To&logoColor=white" alt="Dev.To">
  </a>
</p>



![alt text](https://www.cortexflow.org/app.png)

## 📬Contacts

- **Tettamanti Lorenzo**  [📧 lorenzotettamanti5@gmail.com](mailto:lorenzotettamanti5@gmail.com)

- **Lorenzo Bradanini**  [📧 lorenzolollobrada@gmail.com](mailto:lorenzolollobrada@gmail.com)

# ℹ️ What is CortexBrain?
**CortexBrain** is an ambitious open-source project aimed at creating an intelligent, lightweight, and efficient architecture to connect cloud and edge devices seamlessly. 
By leveraging the power of eBPF, CortexBrain can successfully manage **networking** and **observability** in a distributed cluster, limiting resource waste and improving overall performance. 

The linux in-kernel verifier (BPF Verifier) ensures that all the programs are safe to run and a JIT compiler converts the bytecode to CPU architecture specific for native execution efficiency. CortexBrain can observe and trace network events such as **ingress** (incoming) and **egress** (outgoing) connections, apply policies, and distribute traffic among different backends directly at **kernel level** by attaching the programs in different hooks such as TC (traffic control) and XDP hooks. All the intercepted events are successfully notified in the user space thanks to BPF maps. 

Comprehensive information about CortexBrain’s core architecture, installation, and practical applications is available in the [Official Documentation](https://docs.cortexflow.org/) and on the [CortexFlow blog](https://blog.cortexflow.org/).

# **🧑🏻‍🔬 Current Development Focus**  

Our current development efforts are dedicated to the following key improvements:  

- 🔧 **System Architecture:** Optimizing the actual [architecture](https://www.cortexflow.org/doc/#architecture) by eliminating sidecar proxies while maintaining scalability
- 🌐 **Kernel-Level Observability with eBPF:** Enhancing system observability by leveraging eBPF to collect logs,metrics at the kernel level
- 📊 **Expanded Metrics:** Extending the current [metrics](https://www.cortexflow.org/doc/#detected-metrics) landscape to provide deeper understanding  
- 🖥️ **Integrating a Command Line Interface:** We're actually working on a CLI to let users interact with the cluster without any stress or frustation. You can track the progress of the CLI tool by referring to the [[CLI Milestone]](https://github.com/CortexFlow/CortexBrain/milestone/3)
- 📡 **Cloud-Edge Multi-Cluster Integration:** Enabling orchestration between cloud environments and edge devices

![Architecture](Doc/docs/cf_architecture.svg "Cortexflow architecture")

# 🤖 Getting Started

> <p align="center"> ⚠️ CortexBrain is still in its development stages, so you can expect some bugs. Contributions and feedback are highly appreciated to help improve the project! 
</p>

## 🥷 Installation (for users)
CortexBrain provides a simple installation for users thanks to his command line interface

- Install the CLI using cargo 
   ```bash
   cargo install cortexflow-cli
   ```
   You can find the installation guide in the [official documentation](https://docs.cortexflow.org)
- Start your local cluster
- Install CortexBrain components:
   ```bash
   cfcli install
   ```
- List all the installed services:
   ```bash
   cfcli service list
   ```
   


## 🥷 Installation (for developers and contributors)

- **Clone the Repository**:  First, clone the repository to your local machine.

   ```bash
   git clone https://github.com/CortexFlow/CortexBrain.git
    ```

- **Install required packages**:

   | **Feature**              | **Requirements**                                                                 |
   | ------------------------- | -------------------------------------------------------------------------------- |
   | **CortexBrain Core**      | - Kubernetes or Minikube v1.34.0  <br> - Linux system with kernel version >= 5.15 (mandatory for core development)  <br> - Rust programming language (rustc >= 1.85.0) preferably a **nightly** version |
   | **CortexBrain Dashboard** | - npm v10.7.0  <br> - React v18.2.0  <br> - Electron v33.2.0                      |

## **⚛️ Core Development:**  
   - Install Rust using RustUp tools : 
      ```bash
      curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
      ```  
   - Install [Docker](https://www.docker.com/get-started/)
   - Install [Minikube](https://minikube.sigs.k8s.io/docs/start/?arch=%2Fwindows%2Fx86-64%2Fstable%2F.exe+download) or any other local kubernetes environment 
   - Run minikube (or your local environment)
      ```bash
         minikube start
      ```
   - Install the CLI:
      ```bash
         cargo install cortexflow-cli
      ```
   - Install the core components:
      ```bash
         cfcli install
      ```



- ## **📈 Dashboard Development:**  
   - Install [Node.js](https://nodejs.org/en/download)
   - Open the dashboard folder and install the required packages 
      ```bash
         cd dashboard
         npm install 
      ```  
   - Run the local development server
      ```bash
         npm start 
      ```


# 💪🏻 Contributing
Do you think the project is missing something? Contributing is the best way to show your skills and leave your mark on a project.
If you know DevOps/Kubernetes, networking, security, or you enjoy maintaining a repository, please write an email to lorenzotettamanti5@gmail.com
   | **Role**              | **Skills** | **Tasks** | **Related Issues and Milestones** |
   | ------------------------- | ------------------------------------------------------------------------- | --------------- |--------|
   | **CortexBrain Core Developer**      | - Kubernetes  <br> - Networks  <br> - Rust programming language | - Work alongside us to build and optimize the core functionalities (Client,DNS,Proxy,Telemetry,etc..) <br>                | - [Rust](https://github.com/CortexFlow/CortexBrain/labels/rust) <br> - [Core](https://github.com/CortexFlow/CortexBrain/milestone/1) <br> - [eBPF](https://github.com/CortexFlow/CortexBrain/labels/ebpf)
   | **CortexBrain Dashboard Developer** | - React  <br> - Frontend Development <br> - Javascript/TypeScript | - Work alongside us to design and improve the dashboard  <br>            | [Javascript](https://github.com/CortexFlow/CortexBrain/labels/javascript)
   | **General Mantainers** | - Github  <br> - Practical organition  <br> - Documentation                   | - Keep the repository organized and clean <br> - Write/Update documentation <br> - Spot typos in the repository     | - [Documentation](https://github.com/CortexFlow/CortexBrain/labels/documentation) <br> - [question](https://github.com/CortexFlow/CortexBrain/labels/question)
   | **Code Reviewers/Testers** | - Rust  <br> - Javascript/TypeScript  <br> - Kubernetes <br> - Docker    | - Review code and suggest changes/optimizations <br> - Write tests for CI/CD  | [Code refactoring](https://github.com/CortexFlow/CortexBrain/labels/code%20refactoring)
 

## 🤖 How to Contribute?
We welcome contributions from the community! To contribute to the project, please follow these steps:

- Fork the repository.
- Check out [Contributing Best Practices](https://github.com/CortexFlow/CortexBrain/blob/main/CONTRIBUTING.md) 
- Create a new branch for your feature (`git checkout -b feature/feature-name`).
- Submit a Pull Request with a detailed explanation of your changes.

## 🙋**Proposing New Features**

If you would like to contribute a new feature, we ask you to open a discussion before submitting a PR. This is to ensure that all new features align with the project's goals and to avoid overlapping work or conflicting views.

Please initiate a discussion in the [GitHub Discussions](https://github.com/CortexFlow/CortexBrain/discussions) section where we can collectively review, refine, and approve your idea before you begin implementation. Pull Requests for new features that have not been discussed beforehand may be declined to maintain project coherence and ensure alignment with the broader roadmap.

By collaborating in this manner, we can maintain clarity and consistency, ensuring that all contributors are working towards the same objectives. Thank you for your understanding and contributions!

## 🐐 Top contributors
[![Top contributors](https://images.repography.com/54717595/CortexFlow/CortexBrain/top-contributors/bRL3WTk3lP0LlkiA2QM-GAH_NLqgBwcXYg8aH_s_9Fg/_YHQeQ-ptyH2aRy6rfxNfiMSSDWLoxKWQgKovd2sKJM_table.svg)](https://github.com/CortexFlow/CortexBrain/graphs/contributors)
