# CortexBrain © 2025

**Enabling developers to effectively monitor and analyze distributed applications for rapid and efficient problem solving**

_Written in Rust and eBPF for minimum overhead and maximum performance_

<div align="center">
  <a href="https://github.com/CortexFlow/CortexBrain/releases">
    <img src="https://img.shields.io/badge/Release-Currently%20under%20development-red?style=for-the-badge&logo=github" alt="Release">
  </a>
  <a href="./LICENSE">
    <img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg?style=for-the-badge&logo=open-source-initiative&logoColor=white" alt="License">
  </a>
  <a href="https://docs.cortexflow.org/">
    <img src="https://img.shields.io/badge/Documentation-Available-brightgreen?style=for-the-badge&logo=readthedocs&logoColor=white" alt="Documentation">
  </a>
</div>

## 📬Contacts

- **Tettamanti Lorenzo** [📧 lorenzotettamanti5@gmail.com](mailto:lorenzotettamanti5@gmail.com)

- **Lorenzo Bradanini** [📧 lorenzolollobrada@gmail.com](mailto:lorenzolollobrada@gmail.com)

## ℹ️ What is CortexBrain ?

**CortexBrain** is an ambitious open-source project designed to build an intelligent, lightweight, and highly efficient monitoring platform for distributed cloud and hybrid (cloud–edge) workflows.
By leveraging the power of eBPF, CortexBrain can successfully manage **networking** and **observability** in a distributed cluster, limiting resource waste and improving overall performance.

Comprehensive information about CortexBrain’s core architecture, installation, and practical applications is available in the [Official Documentation](https://docs.cortexflow.org/) and on the [CortexFlow blog](https://blog.cortexflow.org/).

## ⚡ Why CortexBrain ?

- **🔎 Deeper Insights**: CortexBrain integrates eBPF with the KubeAPI to produce deeper kernel-level insights of your system

- **🚁 No sidecar overhead:** Sidecarless architecture that eliminates memory waste and processing overhead

- **🔒 Safety:** The linux **BPF Verifier** ensures that all the programs are safe to run.A **JIT** compiler converts bytecode into native CPU instructions for optimal execution efficiency. CortexBrain can trace network traffic such as **ingress** (incoming) and **egress** (outgoing) TCP/UDP connections and apply policies directly at **kernel level** by attaching the programs in different hooks such as TC (traffic control) and XDP hooks. All the intercepted events are successfully propagated in the **user space** thanks to BPF maps.

## **🧑🏻‍🔬 Current Development Focus**
Our current development efforts are dedicated to the following key features:

- **🌐 Open Telemetry Integration:** Integrating the open telemetry stack to our metrics to produce industry standard metrics formats
- **📈 Metrics enhancement:** Expanding the current [metrics](https://docs.cortexflow.org/cfcli/overview/#monitoring-and-status-commands) landscape to provide deeper system understanding
- **📊 DashBoard Integration:** Delivering beautiful _user centered_ data visualization from the collected metrics
- **📡 [Experimental] Cloud-Edge Multi-Cluster Integration:** Enabling orchestration between hybrid cloud and edge environments
- **🧪[Experimental] GPU Observability**: Introducing GPU tracing and monitoring capabilities to efficiently support AI/ML applications

![Architecture](Doc/docs/cf_architecture.svg "Cortexflow architecture")

## 🤖 Getting Started

> <p align="center"> ⚠️ CortexBrain is still in its development stages, so you can expect some bugs. Contributions and feedback are highly appreciated to help improve the project!

</p>

## 🥷 Installation

CortexBrain provides a simple installation for users thanks to his command line interface. You can find the installation guide in the [official documentation](https://docs.cortexflow.org)

### _Install the CLI using cargo_
```bash
cargo install cortexflow-cli
```
### _Start your local cluster_
### _Install CortexBrain components_
```bash
cfcli install cortexflow
```
### _List all the installed services_
```bash
cfcli service list
```

## 💪🏻 Contributing

Do you think the project is missing something? Contributing is the best way to show your skills and leave your mark on a project.
If you know DevOps/Kubernetes, networking, security, or you enjoy maintaining a repository, please write an email to lorenzotettamanti5@gmail.com
| **Role** | **Skills** | **Tasks** | **Related Issues and Milestones** |
| ------------------------- | ------------------------------------------------------------------------- | --------------- |--------|
| **CortexBrain Core Developer** | - Kubernetes <br> - Networks <br> - Rust programming language | - Work alongside us to build and optimize the core functionalities (Client,DNS,Proxy,Telemetry,etc..) <br> | - [Rust](https://github.com/CortexFlow/CortexBrain/labels/rust) <br> - [Core](https://github.com/CortexFlow/CortexBrain/milestone/1) <br> - [eBPF](https://github.com/CortexFlow/CortexBrain/labels/ebpf)
| **CortexBrain Dashboard Developer** | - React <br> - Frontend Development <br> - Javascript/TypeScript | - Work alongside us to design and improve the dashboard <br> | [Javascript](https://github.com/CortexFlow/CortexBrain/labels/javascript)
| **General Mantainers** | - Github <br> - Practical organition <br> - Documentation | - Keep the repository organized and clean <br> - Write/Update documentation <br> - Spot typos in the repository | - [Documentation](https://github.com/CortexFlow/CortexBrain/labels/documentation) <br> - [question](https://github.com/CortexFlow/CortexBrain/labels/question)
| **Code Reviewers/Testers** | - Rust <br> - Javascript/TypeScript <br> - Kubernetes <br> - Docker | - Review code and suggest changes/optimizations <br> - Write tests for CI/CD | [Code refactoring](https://github.com/CortexFlow/CortexBrain/labels/code%20refactoring)

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
