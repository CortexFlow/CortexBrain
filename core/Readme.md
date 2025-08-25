# Experimental Core Build in Rust

This project is an experimental core build developed using the Rust programming language. The core functions are currently in an early development stage, aiming to enhance the current system capabilities to run in limited-resource environments such as edge devices.

>⚠️ Note:  
A pre-alpha version is expected to be released by the **end of 2025**.

The core leverages the power of EBPF to directly interact with the Linux kernel. This is extremely important to reduce overhead and latency, especially in limited-resource environments. In an EBPF-based architecture, many tasks can be done in the **Kernel Space**, such as Networking, Security, and Observability, leaving the **User Space** to only load and manage the BPF programs.

Since EBPF is a linux-native framework to interact with the Linux kernel, programs are typically written in C. We decided to use the Rust programming language to add additional memory safety features that Rust offers. The use of the Rust language is possible nowadays thanks to the Aya project (@aya-rs) that provides a solid library to interact with the linux kernel.

Below you will find a detailed project map to help new users and developers deep dive into the main core components.

If you're interested or have any questions, feel free to contact:  
📧 **lorenzotettamanti5@gmail.com**

## Core Project Map
   | **Component**              | **Description** | **Related Issues or Milestones** |
   | ------------------------- |--------------------------------------------------- | --------------- |
   | **Conntracker**      |   Kernel space component that tracks connections inside the cluster using TC hook and classifier. Intercepts Ingress and Egress connections by registering a PerfEventArray with five parameters (Source Ip, Source Port, Destination Ip, Destination Port, Hash ID). The hash ID is generated to uniquely track the connection based on the previous 4 parameters          | - [[92]](https://github.com/CortexFlow/CortexBrain/issues/92) <br> - [Core](https://github.com/CortexFlow/CortexBrain/milestone/1)
   | **Identity**      |    User Space program that uses Conntracker component and displays active connections in the cluster           | -  [[92]](https://github.com/CortexFlow/CortexBrain/issues/92) <br> - [Core](https://github.com/CortexFlow/CortexBrain/milestone/1)
   | **Metrics_tracer**      |  Kernel Space program that collects the main CortexBrain metrics collectors | - [[91]](https://github.com/CortexFlow/CortexBrain/issues/78) <br> - [Core](https://github.com/CortexFlow/CortexBrain/milestone/1)
   | **Metrics**      |  User Space implementation of the metrics_tracer BPF scripts. The metrics crate also aggregates,  stores, and hosts the main data processing functions | - [[91]](https://github.com/CortexFlow/CortexBrain/issues/78) <br> - [Core](https://github.com/CortexFlow/CortexBrain/milestone/1)
   