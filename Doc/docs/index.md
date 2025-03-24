!!! note 
    CortexBrain is currently in active development so that occasional bugs may occur. Your contributions and feedback are invaluable in refining and enhancing the project! 🚀  

**CortexBrain** is an ambitious open-source project aimed at creating an intelligent, lightweight, and efficient architecture to seamlessly connect cloud and edge devices.  

## Current Development Stage

You can see the development stage of every component here:

| **Component**       | **Stage**                | **Latest Commit** | **Referring Branch**  |**Related Milestone**
|---------------------|--------------------------|-------------------|-----------------------|-----------------------|
| **Dashboard**       | Under development  | [![GitHub last commit](https://img.shields.io/github/last-commit/CortexFlow/CortexBrain?style=flat-square&logo=github&color=success)](https://github.com/CortexFlow/CortexBrain/commits/feature/frontend)                | `feature frontend`    | CortexBrain v 0.1.0 Launch |
| **Client**          |  Under development  | [![GitHub last commit](https://img.shields.io/github/last-commit/CortexFlow/CortexBrain?style=flat-square&logo=github&color=success)](https://github.com/CortexFlow/CortexBrain/commits/core)                 | `feature/core`        | CortexBrain core v 0.1.0 |
| **Proxy**           |  Under development  | [![GitHub last commit](https://img.shields.io/github/last-commit/CortexFlow/CortexBrain?style=flat-square&logo=github&color=success)](https://github.com/CortexFlow/CortexBrain/commits/core)                 | `feature/core`        | CortexBrain core v 0.1.0 |
| **Controller**      |  Under development  |   [![GitHub last commit](https://img.shields.io/github/last-commit/CortexFlow/CortexBrain?style=flat-square&logo=github&color=success)](https://github.com/CortexFlow/CortexBrain/commits/core)               | `feature/core`   | CortexBrain core v 0.1.0 |
| **CLI**             | ❌ Not started yet       | ❌                | `feature/cli`         | CortexBrain CLI v .0.1 |
| **Identity Service**             | ❌ Not started yet       | ❌                | ❌         | CortexBrain core v 0.1.0 |

## An Introduction to Service Mesh

A **service mesh** is a specialized infrastructure layer embedded within a software application that manages communication between services. It handles critical functions such as traffic routing, security, observability, and resiliency, while shielding individual services from these complexities.

In modern applications, functionality is often divided into a network of specialized services, each performing a specific task. To fulfill its role, a service may need to request data from multiple other services. However, issues arise when certain services, like a retailer’s inventory database, become overwhelmed with requests. This is where a service mesh proves invaluable—it orchestrates and optimizes communication between services, ensuring all components work seamlessly together.
## Architecture

The CortexFlow architecture is designed to ensure a robust, scalable, and fault-tolerant system that can operate seamlessly without interruptions. It is composed of several key components that work together to provide a continuous and reliable infrastructure. These components are orchestrated within a Kubernetes cluster, ensuring efficient resource management, high availability, and scalability. Below is a GIF that visually represents the architecture and illustrates how the components interact within the cluster.

![Architecture](architecture.gif "CortexFlow architecture")

The architecture is divided into two main planes: the **Control Plane** and the **Data Plane**. The Control Plane is responsible for managing and orchestrating the system, while the Data Plane handles the actual data processing and traffic routing. This separation of concerns allows CortexFlow to maintain a high level of performance and reliability.

---

# Control Plane

The **Control Plane** is the core of the CortexFlow architecture. It is responsible for managing the overall system, including service discovery, configuration management, and monitoring. The Control Plane consists of a collection of services that run within a dedicated Kubernetes namespace named `CortexFlow`. These services work together to ensure the system operates smoothly and can dynamically adapt to changes in the environment. Below you can see the key components of the control plane.

## Proxy Injector:  


The **Proxy Injector** is a Kubernetes admission controller that plays a critical role in the CortexFlow architecture. It listens for webhook requests triggered whenever a new pod is created in the cluster. Once a new pod is created, a mutating admission controller is triggered. This controller runs on an **HTTPS server with TLS encryption** and exposes a `/mutate` endpoint.
This summarize the inject logic:

   1. A new pod is created in the cluster (e.g., a [test pod](https://github.com/CortexFlow/CortexBrain/blob/feature/core/core/src/testing/deploy-test-pod.yaml)).
   2. The **mutating webhook** intercepts the `"CREATE"` request from the Kubernetes API server.
   3. The **proxy-injector service** processes the request.
   4. The **proxy-injector** use the `check_and_validate_pod` function to determine if the pod is eligible for injection.
   5. If the validation succeeds, the **mutating webhook injects** the CortexFlow proxy as a sidecar by applying a **JSON patch encoded in Base64**.

This proxy is responsible for handling network traffic, enforcing security policies, and collecting metrics. The injection process is seamless and ensures that all pods within the cluster are automatically equipped with the necessary components to integrate with CortexFlow.

### Security and Deployment

- The **admission controller is secured via TLS**.
- The webhook server **listens on port 9443** and serves requests over HTTPS.
- The entire injection process is **seamless**, ensuring that all eligible pods within the cluster are automatically equipped with the necessary components to integrate with CortexFlow.

## Monitoring System:  

CortexFlow leverages **Prometheus** as its primary monitoring system. Prometheus is a powerful open-source tool designed for real-time monitoring and alerting. It collects and stores time-series data, enabling CortexFlow to capture a wide range of metrics that are critical for system health and performance.  

Prometheus is configured to scrape metrics from various components within the cluster, including the injected proxies, Kubernetes nodes, and other services. These metrics are then made available for querying and visualization. By exposing the `9090` TCP port, users can directly access Prometheus to query metrics using its built-in query language (PromQL). Additionally, CortexFlow provides a user-friendly dashboard that aggregates and visualizes these metrics, making it easier for users to monitor the system's health and performance.

### Detected Metrics:  
Currently, CortexFlow collects a limited set of metrics, but the team is actively working on expanding the monitoring capabilities to include more features and metrics. Below is the list of metrics currently being collected:  

1. **Total DNS Requests**:  
    This metric measures the total number of DNS requests that the proxy is handling. It provides insights into the DNS query load and helps identify potential bottlenecks or anomalies in DNS resolution.

2. **DNS Response Time**:  
    This metric tracks the time taken for DNS queries to be resolved. It is a critical indicator of the performance of the DNS resolution process and helps ensure that the system is meeting its latency requirements.


The CortexFlow architecture is designed to be modular and extensible, allowing new components and features to be added as the system evolves. By leveraging Kubernetes and modern monitoring tools like Prometheus, CortexFlow ensures that the system remains resilient, scalable, and easy to manage.

---

# Data Plane

The **Data Plane** is the backbone of the CortexFlow service mesh, responsible for handling all traffic between services. It ensures secure, reliable, and efficient communication across the cluster. When a service needs to communicate with another service, the **sidecar proxy** plays a critical role in managing the interaction. Here's a detailed breakdown of how the Data Plane operates:

1. **Request Interception**:  
   The sidecar proxy intercepts all outgoing requests from the service. This interception happens transparently, without requiring any changes to the application code.

2. **Request Encapsulation**:  
   Once intercepted, the request is encapsulated into a separate network connection. This encapsulation ensures that the communication is isolated and secure.

3. **Secure Channel Establishment**:  
   The sidecar proxy establishes a secure, encrypted channel (e.g., using mTLS - mutual Transport Layer Security) between the source and destination proxies. This ensures that all communication is protected from eavesdropping or tampering.

!!! warning
    At the moment of publishing this documentation we are working on the implementation of the security feature.

### Key Responsibilities of the Data Plane

- **Low-Level Messaging**:  
  Sidecar proxies handle all low-level communication between services, abstracting away the complexity from the application.

- **Resiliency Features**:  
  The Data Plane implements advanced features such as **circuit breaking** and **request retries** to improve system resiliency. These features prevent cascading failures and ensure graceful degradation during high load or service failures.

- **Service Mesh Features**:  
  The Data Plane is responsible for implementing core service mesh functionalities, including:

      1. **Load Balancing**: Distributing traffic evenly across service instances to optimize resource utilization.
      2. **Service Discovery**: Automatically detecting and connecting to available services within the cluster.
      3. **Traffic Routing**: Enabling advanced traffic management, such as canary deployments, A/B testing, and blue-green deployments.

!!! warning
    At the moment of publishing this documentation we are working on the implementation of the load balancer. 


## CortexFlow Proxy

The **CortexFlow Proxy** is a critical component of the CortexFlow service mesh. It acts as a **sidecar proxy**, meaning it runs alongside each service instance in the cluster. The proxy is responsible for managing all inbound and outbound traffic for its associated service, ensuring secure and efficient communication without requiring any modifications to the application itself.

### Key Features of CortexFlow Proxy

1. **UDP/TCP Traffic Interception**:  
   The CortexFlow Proxy is capable of intercepting both **UDP** and **TCP** traffic. It forwards these messages to the Kubernetes DNS service, ensuring seamless communication between services.  

    - **Default UDP Port**: `5053`  
    - **Default TCP Port**: `5054`  

2. **Observability and Logging**:  
   The CortexFlow Proxy collects detailed metrics and error logs from the traffic it handles. These metrics are sent to **Prometheus**, CortexFlow's monitoring system, allowing users to visualize and analyze real-time performance data. This observability is crucial for debugging, performance tuning, and ensuring system health.

3. **Secure Communication**:  
   By default, the CortexFlow Proxy establishes secure, encrypted channels (e.g., using mTLS) for all communication. This ensures that sensitive data is protected from unauthorized access.

### How the CortexFlow Proxy Works

- **Ingress Traffic**:  
  The proxy intercepts incoming traffic (ingress) and routes it to the appropriate service within the cluster. It ensures that the traffic is validated, secure, and properly load-balanced.

- **Egress Traffic**:  
  For outgoing traffic (egress), the proxy encapsulates the request, establishes a secure connection, and forwards it to the destination service. It also handles retries and circuit breaking in case of failures.

- **Transparent Integration**:  
  The CortexFlow Proxy integrates seamlessly with your application, requiring no code changes. It operates at the network level, ensuring that your application remains unaware of the underlying service mesh.

### Visualizing the Proxy Topology

To better understand how the CortexFlow Proxy operates within the cluster, here's a visual representation of the sidecar proxy topology:

![ProxySidecarTopology](assets/cf_sidecar_proxy_topology.gif "CortexFlow Proxy Sidecar Topology")

### A Real Example

In this example, we have deployed two pods in our Kubernetes cluster using Minikube. Since pods live in an isolated environment (we call it a container), they are not able to communicate with each other simply.  

We noticed that immediately after we deployed the test pods, the proxy injector successfully intercepted our `CREATE` requests. The admission webhook deployed a container with the proxy sidecar in both pods. Now the pods can easily communicate using a TCP or UDP protocol.  

Let's test this by sending a simple TCP message `"Hello from proxy-sidecar"` from `test-proxy` to `test-proxy2` pod:

### Request format:
To communicate with the poe this JSON format is required:
```json
{
   "service": <destination-service.namespace>,
   "direction":"<direction>",
   "payload":{
      "<payload>": "<message>"
   }
}
```
Here's the explanation of each key:

   - **direction**: The direction of the message (Incoming). If there's no direction in the payload the system logs an error message and send a "Delivery failed" message
   - **service**: The name of the destination service. For example, if you want to send a message to the pod "test-proxy2" your destination is "test-proxy.cortexflow". In general, the default namespace is set to "cortexflow" as a fallback and can be not specified if your service lives in the cortexflow namespace,otherwise you must specify the namespace
   - **payload**: The message you want to send encoded in base64 format. You can easily encode a text message in base64 using this command:

   ```bash
   echo -n "Hello from proxy-sidecar" | base64
   ```
   Result:
   ```bash
   SGVsbG8gZnJvbSBwcm94eS1zaWRlY2Fy
   ```

If you try to send a message using a different format you'll get an error and a 'Delivery Failed' message.    
Now that all the assumptions have been made, we can try to send a message to test-proxy2 from the test-proxy pod using this command:
```bash
kubectl exec test-proxy -c proxy-sidecar -n cortexflow -- sh -c '
    echo "Test: Incoming Message ⏳"
    printf "{\"service\":\"test-proxy2.cortexflow\",\"direction\":\"Incoming\",\"payload\":\"eyJwYXlsb2FkIjogIkhlbGxvIGZyb20gcHJveHktc2lkZWNhciJ9\"}\n" | nc -w3 test-proxy2 5054 && echo "✅ Test completed"
```
We receive this response: 

```json
{
   "payload":"eyJzdGF0dXMiOiJyZWNlaXZlZCJ9",
   "service":"test-proxy2",
   "direction":"Outcoming"
}
```
At this point we have done! We received a success response from test-proxy2. We can decode the payload using this command:
```base
echo eyJzdGF0dXMiOiJyZWNlaXZlZCJ9 | base64 --decode
```
Resulting in:
```base
{"status":"received"}
```
In the next paragraph there's a detailed explanation of the response JSON.
### Response format:
Every time you send a message from one service to another service, if the message is successfully delivered you get a response like the one presented below:
```json
{
   "payload":<reponse-payload>,
   "service": <destination-service.namespace>,
   "direction":<Outcoming>
}
```
Here's a detailed explanation of every key of the response:

   - **payload**: The response message {"status":"received"} encoded in base64. This is the payload that the destination service has sent back.
   - **service**: The name of the service that has sent the response. For example, if you have successfully sent a
   message to the pod "test-proxy2.cortexflow" you will see "test-proxy2.cortexflow" in the response.
   - **direction**: The direction of the message (Outcoming).




### Summary

The **CortexFlow Proxy** is the workhorse of the CortexFlow service mesh, enabling secure, reliable, and efficient communication between services. By handling traffic interception, encapsulation, and secure channel establishment, it ensures that your applications can communicate seamlessly without requiring any modifications. Additionally, its support for observability and logging provides valuable insights into system performance, making it easier to monitor and troubleshoot your cluster.
