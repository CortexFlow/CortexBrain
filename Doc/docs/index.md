!!! note 
    CortexBrain is currently in active development so that occasional bugs may occur. Your contributions and feedback are foundamental for refining and enhancing the project! 🚀  

**CortexBrain** is an ambitious open-source project aimed at creating an intelligent, lightweight, and efficient architecture to seamlessly connect cloud and edge devices.  
## **Current Development Stage**

You can see the development stage of every component here:

| **Component**       | **Stage**                | **Latest Commit** | **Referring Branch**  |**Related Milestone**
|---------------------|--------------------------|-------------------|-----------------------|-----------------------|
| **Dashboard**       | Under development  | [![GitHub last commit](https://img.shields.io/github/last-commit/CortexFlow/CortexBrain?style=flat-square&logo=github&color=success)](https://github.com/CortexFlow/CortexBrain/commits/feature/frontend)                | `feature/frontend`    | CortexBrain v 0.1.0 Launch |
| **Identity Service**             | Under development       | [![GitHub last commit](https://img.shields.io/github/last-commit/CortexFlow/CortexBrain?style=flat-square&logo=github&color=success)](https://github.com/CortexFlow/CortexBrain/commits/feature/ebpf-core)                 | `feature/ebpf-core`         | CortexBrain core v 0.1.0 |
| **Agent**           |  Under development  | [![GitHub last commit](https://img.shields.io/github/last-commit/CortexFlow/CortexBrain?style=flat-square&logo=github&color=success)](https://github.com/CortexFlow/CortexBrain/commits/core)                 | `feature/core`        | CortexBrain core v 0.1.0 |
| **CLI**             | Under development       |[![GitHub last commit](https://img.shields.io/github/last-commit/CortexFlow/CortexBrain?style=flat-square&logo=github&color=success)](https://github.com/CortexFlow/CortexBrain/commits/feature/cli)              | `feature/cli`         | CortexBrain CLI v .0.1 |

# **An Introduction to Service Mesh**

A **service mesh** is a specialized infrastructure layer embedded within a software application that manages communication between services. It handles critical functions such as traffic routing, security, observability, and resiliency, while shielding individual services from these complexities.

In modern applications, functionality is often divided into a network of specialized services, each performing a specific task. To fulfill its role, a service may need to request data from multiple other services. However, issues arise when certain services, like a retailer’s inventory database, become overwhelmed with requests. This is where a service mesh proves invaluable it orchestrates and optimizes communication between services, ensuring all components work seamlessly together.

# **Architecture**
The CortexFlow architecture is designed to ensure a robust, scalable, and fault-tolerant system that can operate seamlessly without interruptions. It is composed of several key components that work together to provide a continuous and reliable infrastructure. These components are orchestrated within a Kubernetes cluster, ensuring efficient resource management, high availability, and scalability. Below is a GIF that visually represents the architecture and illustrates how the components interact within the cluster.

![Architecture](./cf_architecture.svg "Cortexflow architecture")

## **User space vs kernel space**
In the Linux kernel the **user space** is the environment where user-facing applications run. This includes applications such as web servers, Chrome, text editors, and command utilities. User space applications are also known as userland applications.

User space applications cannot directly access the system’s hardware resources. They must make system calls to the kernel to request access to these resources.

Kernel space is where the core of the operating system, the kernel, operates. The kernel is responsible for managing the system’s resources, such as the CPU, memory, and storage. It also provides system calls, which are interfaces that allow userspace applications to interact with the kernel. The kernel has unrestricted access to the system’s hardware resources. This is necessary for the kernel to perform its essential tasks, such as scheduling processes, managing memory, and handling interrupts.


The CortexFlow architecture is built upon the Linux kernel, the core or the brain, referred as CortexBrain, interacts directly with the Linux kernel entities and, thanks to eBPF (Extended Berkley Packet Filter) CortexBrain can trace and extract relevant cluster insights before exiting the kernel space, this results in exact cluster metrics data without modifying its source code. eBPF is a virtual machine designed to run with the Linux kernel. eBPF programs are written in a C-like language, such as C itself or Rust. The code is compiled into bytecode and then checked by the BPF verifier, which analyzes the bytecode before it is executed in the kernel. The verifier scans how the program manipulates memory addresses and returns errors, not allowing the kernel to execute the program if some operation is considered suspicious. The verifier must track the range of all possible values in each register and so in each stack slot, for example, if a programs execute a function that assigns a memory address in a way that the verifier cannot prove its safety, the entire program will never be accepted as a valid program and so will never be executed. 

The execution flow of a BPF program can be resumed in four key steps:

- Program Loading: An application in the user space loads the compiled BPF program bytecode into the kernel. Typically, the program can be attached to an interface, a Kprobe, or a system call (syscall)
- Program verification: The verifier scans and analyzes the program to ensure its validity and safety
- Compilation: The JIT compiler translates the bytecode into native machine code
- Execution: The compiled code is executed by the kernel

This rigorous process ensures safety and maximum performance by molding itself with an event-driven approach, and so can be used as a foundation of an efficient monitoring system (Metrics and Observability in the graph above) and a fast networking plugin that we refer to in the illustration above as Identity Service.


# **CortexBrain components**
Cortexflow core components, also referred to as CortexBrain components, are composed of a list of services and utilities that empower users to efficiently observe networking events and resource usage. It also includes a command line interface (CLI) referred to as cfcli (cortexflow-cli) and a dashboard. Every component is carefully and detaily documented below.

## **CLI**
The command line interface, also known as CLI, is an essential part of the CortexFlow User Experience. It allows users and developers to interact with all the core components without directly managing the manifests' YAML files. The CLI stores the relevant information, such as the underlying cluster environment (e.g., Kubernetes, Docker Swarm, etc), to support multiple environments without changing the user experience. Right now, the CLI only supports **Kubernetes** as an orchestrator.

 The CLI is available to install with the cargo package manager; we have carefully documented the installation in this [page](./tools/tools.md).

!!! warning
    Right now, the identity service, metrics, and dashboard are under development until 2026. We will release the first documentation snippet soon