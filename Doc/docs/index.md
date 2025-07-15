!!! note 
    CortexBrain is currently in active development so that occasional bugs may occur. Your contributions and feedback are invaluable in refining and enhancing the project! 🚀  

**CortexBrain** is an ambitious open-source project aimed at creating an intelligent, lightweight, and efficient architecture to seamlessly connect cloud and edge devices.  

## Current Development Stage

You can see the development stage of every component here:

| **Component**       | **Stage**                | **Latest Commit** | **Referring Branch**  |**Related Milestone**
|---------------------|--------------------------|-------------------|-----------------------|-----------------------|
| **Dashboard**       | Under development  | [![GitHub last commit](https://img.shields.io/github/last-commit/CortexFlow/CortexBrain?style=flat-square&logo=github&color=success)](https://github.com/CortexFlow/CortexBrain/commits/feature/frontend)                | `feature/frontend`    | CortexBrain v 0.1.0 Launch |
| **Identity Service**             | Under development       | [![GitHub last commit](https://img.shields.io/github/last-commit/CortexFlow/CortexBrain?style=flat-square&logo=github&color=success)](https://github.com/CortexFlow/CortexBrain/commits/feature/ebpf-core)                 | `feature/ebpf-core`         | CortexBrain core v 0.1.0 |
| **Agent**           |  Under development  | [![GitHub last commit](https://img.shields.io/github/last-commit/CortexFlow/CortexBrain?style=flat-square&logo=github&color=success)](https://github.com/CortexFlow/CortexBrain/commits/core)                 | `feature/core`        | CortexBrain core v 0.1.0 |
| **CLI**             | Under development       |[![GitHub last commit](https://img.shields.io/github/last-commit/CortexFlow/CortexBrain?style=flat-square&logo=github&color=success)](https://github.com/CortexFlow/CortexBrain/commits/feature/cli)              | `feature/cli`         | CortexBrain CLI v .0.1 |

## An Introduction to Service Mesh

A **service mesh** is a specialized infrastructure layer embedded within a software application that manages communication between services. It handles critical functions such as traffic routing, security, observability, and resiliency, while shielding individual services from these complexities.

In modern applications, functionality is often divided into a network of specialized services, each performing a specific task. To fulfill its role, a service may need to request data from multiple other services. However, issues arise when certain services, like a retailer’s inventory database, become overwhelmed with requests. This is where a service mesh proves invaluable—it orchestrates and optimizes communication between services, ensuring all components work seamlessly together.
## Architecture

The CortexFlow architecture is designed to ensure a robust, scalable, and fault-tolerant system that can operate seamlessly without interruptions. It is composed of several key components that work together to provide a continuous and reliable infrastructure. These components are orchestrated within a Kubernetes cluster, ensuring efficient resource management, high availability, and scalability. Below is a GIF that visually represents the architecture and illustrates how the components interact within the cluster.

![Architecture](./cf_architecture.svg "Cortexflow architecture")

!!! warning
      Right now, the documentation is incomplete and is being updated. We will release an updated version in the next few days