!!! note 
    CortexBrain is currently in active development, so occasional bugs may occur. Your contributions and feedback are invaluable in refining and enhancing the project! 🚀  

**CortexBrain** is an ambitious open-source project aimed at creating an intelligent, lightweight, and efficient architecture to seamlessly connect cloud and edge devices.  

## Current Development Stage

You can see the development stage of every component here:

| **Component**       | **Stage**                | **Latest Commit** | **Referring Branch**  |
|---------------------|--------------------------|-------------------|-----------------------|
| **Dashboard**       | 👨🏻‍🔬 Under development  | [![GitHub last commit](https://img.shields.io/github/last-commit/CortexFlow/CortexBrain?style=for-the-badge&logo=github&color=success)](https://github.com/CortexFlow/CortexBrain/commits/feature/frontend)                | `feature/frontend`    |
| **Client**          | 👨🏻‍🔬 Under development  | [![GitHub last commit](https://img.shields.io/github/last-commit/CortexFlow/CortexBrain?style=for-the-badge&logo=github&color=success)](https://github.com/CortexFlow/CortexBrain/commits/core)                 | `feature/core`        |
| **Proxy**           | 👨🏻‍🔬 Under development  | [![GitHub last commit](https://img.shields.io/github/last-commit/CortexFlow/CortexBrain?style=for-the-badge&logo=github&color=success)](https://github.com/CortexFlow/CortexBrain/commits/core)                 | `feature/core`        |
| **Controller**      | 👨🏻‍🔬 Under development  |   [![GitHub last commit](https://img.shields.io/github/last-commit/CortexFlow/CortexBrain?style=for-the-badge&logo=github&color=success)](https://github.com/CortexFlow/CortexBrain/commits/core)               | `feature/core`        |
| **CLI**             | ❌ Not started yet       | ❌                | `feature/cli`         |
| **Identity Service**             | ❌ Not started yet       | ❌                | ❌         |

## An Introduction to Service Mesh

A **service mesh** is a specialized infrastructure layer embedded within a software application that manages communication between services. It handles critical functions such as traffic routing, security, observability, and resiliency, while shielding individual services from these complexities.

In modern applications, functionality is often divided into a network of specialized services, each performing a specific task. To fulfill its role, a service may need to request data from multiple other services. However, issues arise when certain services, like a retailer’s inventory database, become overwhelmed with requests. This is where a service mesh proves invaluable—it orchestrates and optimizes communication between services, ensuring all components work seamlessly together.
## Architecture

The Cortexflow architecture is designed to ensure a robust, scalable, and fault-tolerant system that can operate seamlessly without interruptions. It is composed of several key components that work together to provide a continuous and reliable infrastructure. These components are orchestrated within a Kubernetes cluster, ensuring efficient resource management, high availability, and scalability. Below is a GIF that visually represents the architecture and illustrates how the components interact within the cluster.

![Architecture](architecture.gif "Cortexflow architecture")

The architecture is divided into two main planes: the **Control Plane** and the **Data Plane**. The Control Plane is responsible for managing and orchestrating the system, while the Data Plane handles the actual data processing and traffic routing. This separation of concerns allows Cortexflow to maintain a high level of performance and reliability.



# Control Plane

The **Control Plane** is the core of the Cortexflow architecture. It is responsible for managing the overall system, including service discovery, configuration management, and monitoring. The Control Plane consists of a collection of services that run within a dedicated Kubernetes namespace named `cortexflow`. These services work together to ensure the system operates smoothly and can dynamically adapt to changes in the environment.

### Key Components of the Control Plane:

1. **Proxy Injector**:  
   The Proxy Injector is a Kubernetes admission controller that plays a critical role in the Cortexflow architecture. It listens for webhook requests triggered whenever a new pod is created in the cluster. Upon receiving a request, the Proxy Injector automatically injects a sidecar proxy into the pod. This proxy is responsible for handling network traffic, enforcing security policies, and collecting metrics. The injection process is seamless and ensures that all pods within the cluster are automatically equipped with the necessary components to integrate with Cortexflow.

2. **Monitoring System**:  
   Cortexflow leverages **Prometheus** as its primary monitoring system. Prometheus is a powerful open-source tool designed for real-time monitoring and alerting. It collects and stores time-series data, enabling Cortexflow to capture a wide range of metrics that are critical for system health and performance.  

   Prometheus is configured to scrape metrics from various components within the cluster, including the injected proxies, Kubernetes nodes, and other services. These metrics are then made available for querying and visualization. By exposing the `9090` TCP port, users can directly access Prometheus to query metrics using its built-in query language (PromQL). Additionally, Cortexflow provides a user-friendly dashboard that aggregates and visualizes these metrics, making it easier for users to monitor the system's health and performance.

#### Detected Metrics:  
!!! note  
    The list of detected metrics is currently limited, but the Cortexflow team is actively working on expanding the monitoring capabilities to include additional metrics and features. This will provide users with even greater visibility into the system's performance and health.
Currently, Cortexflow collects a limited set of metrics, but the team is actively working on expanding the monitoring capabilities to include more features and metrics. Below is the list of metrics currently being collected:  

1. **Total DNS Requests**:  
    This metric measures the total number of DNS requests that the proxy is handling. It provides insights into the DNS query load and helps identify potential bottlenecks or anomalies in DNS resolution.

2. **DNS Response Time**:  
    This metric tracks the time taken for DNS queries to be resolved. It is a critical indicator of the performance of the DNS resolution process and helps ensure that the system is meeting its latency requirements.



The Cortexflow architecture is designed to be modular and extensible, allowing new components and features to be added as the system evolves. By leveraging Kubernetes and modern monitoring tools like Prometheus, Cortexflow ensures that the system remains resilient, scalable, and easy to manage. This architecture provides a solid foundation for building and deploying distributed systems that can handle high traffic loads while maintaining high availability and performance.
# Data Plane
The data plane handles the traffic between services. When a service wishes to communicate with another service, the sidecar proxy performs the following actions:

1. The sidecar intercepts the request
2. Encapsulates the request in a separate network connection
3. Establishes a secure, encrypted channel between the source and destination proxies

Sidecar proxies handle low-level messaging between services. They also implement features, such as circuit interruption and request retries, to improve resiliency and prevent service degradation. Service mesh features, such as load balancing, service discovery, and traffic routing, are implemented in the data plane.

## Cortexflow Proxy
The Cortexflow proxy is an essential part of the Cortexbrain service mesh. It's responsible for the communication between your application and the cluster. Cortexflow proxy acts as an intermediary for inbound and outbound traffic of the associated service without letting requiring application-level modifications. Up to now cortexflow proxy support:

- UDP/TCP traffic interception: cortexflow proxy service is able to intercept udp and tcp messages and forwards them to the kubernetes dns service. The default port for the udp traffic is the 5053 and the default part for the tcp traffic is the 5054
- Observability and Logging: cortexflow sidecar proxy collects metrics and errors and sends them to Prometheus to let the user see real time metrics

Using a more technical language we can say that the cortexflow proxy take the ingress traffic and allows the application in your cluster to seamlessly connect together using different protocols. To summerize, this is what the proxy mesh looks like:
![ProxySidecarTopology](assets/cf_sidecar_proxy_topology.gif "Cortexflow Proxy Sidecar Topology")
