
## CortexFlow Agent: Quick Start Guide
!!! warning 
    Up to now the only supported cluster environment is Kubernetes

This section contains a quick tutorial that shows how to set up a local Kubernetes cluster using two popular tools and how to use the CortexFlow agent to return the latest detected TCP packages.  
The first tool is [minikube](https://minikube.sigs.k8s.io/docs/start/?arch=%2Flinux%2Fx86-64%2Fstable%2Fbinary+download), a popular tool for setting up a single-node local cluster. The second tool is [Kind](https://kind.sigs.k8s.io/), a tool for running local Kubernetes clusters using Docker containers.  

Since CortexFlow works in both environments, you can choose which one suits you best. In both examples, we will use Calico CNI.
## Cluster setup (with Calico)
!!! note 
    If you already have a working cluster you can skip this section

=== "Minikube " 
    To setup a single node minikube cluster, first you need to install minikube from the [Minikube Download Page](https://minikube.sigs.k8s.io/docs/start)

    Then you can create a cluster with Calico CNI using the command:
    ```bash  
      minikube start --cni=calico
    ```
    You can **verify** the Calico installation using this command:
    ```bash  
    watch kubectl get pods -l k8s-app=calico-node -A
    ```
    This is the expected output:
    ```bash  
    NAMESPACE     NAME                READY   STATUS    RESTARTS   AGE
    kube-system   calico-node-2xcf4   1/1     Running   0          57s
    kube-system   calico-node-gkqkg   1/1     Running   0          57s
    kube-system   calico-node-j44hp   1/1     Running   0          57s
    ```

=== "Kind "
    The first step to setup a cluster with Kind is to install Kind from the [Kind Repository](https://github.com/kubernetes-sigs/kind)
    If you have go and docker,podman or nerdctl you can install kind using the command:
    ```bash  
      go install sigs.k8s.io/kind@v0.30.0
    ```
    Then you can create a cluster with the [Calico CNI](https://github.com/projectcalico/calico):

    - Disable the default CNI to use Calico CNI by running the following command:

    ```bash  
      cat > values.yaml <<EOF
      kind: Cluster
      apiVersion: kind.x-k8s.io/v1alpha4
      nodes:
      - role: control-plane
      - role: worker
      - role: worker
      networking:
        disableDefaultCNI: true
        podSubnet: 192.168.0.0/16
      EOF
    ```

    Here we are setting a cluster with one master node and due worker nodes. If you need to set a cluster with only one master node and one worker node you can use this command:

      ```bash  
      cat > values.yaml <<EOF
      kind: Cluster
      apiVersion: kind.x-k8s.io/v1alpha4
      nodes:
      - role: control-plane
      - role: worker
      networking:
        disableDefaultCNI: true
        podSubnet: 192.168.0.0/16
      EOF
      ```


    - Start your Kind cluster with one control plane and two worker nodes by running the following command:
      ```bash  
      kind create cluster --config values.yaml --name dev
      ```

    - Confirm that you now have three nodes in your cluster by running the following command:
      ```bash  
      kubectl get nodes -o wide
      ```
      This is the expected output for a three nodes cluster (master-worker-worker):
      ```bash  
      NAME                STATUS      ROLES           AGE    VERSION   INTERNAL-IP   EXTERNAL-IP   OS-IMAGE             KERNEL-VERSION    CONTAINER-RUNTIME
      dev-control-plane   NotReady    control-plane   4m     v1.25.0   172.18.0.2    <none>        Ubuntu 22.04.1 LTS   5.10.0-17-amd64   containerd://1.6.7
      dev-worker          NotReady    <none>          4m     v1.25.0   172.18.0.4    <none>        Ubuntu 22.04.1 LTS   5.10.0-17-amd64   containerd://1.6.7
      dev-worker2         NotReady    <none>          4m     v1.25.0   172.18.0.3    <none>        Ubuntu 22.04.1 LTS   5.10.0-17-amd64   containerd://1.6.7
      ```

    - Now you have to install the calico operator: 
      ```bash  
      kubectl create -f https://raw.githubusercontent.com/projectcalico/calico/v3.30.3/manifests/operator-crds.yaml
      kubectl create -f https://raw.githubusercontent.com/projectcalico/calico/v3.30.3/manifests/tigera-operator.yaml
      ```
    - Now you need to setup the custom resources. You can deep dive into the configuration options using the [installation reference](https://docs.tigera.io/calico/latest/reference/installation/api) :
      ```bash  
      kubectl create -f https://raw.githubusercontent.com/projectcalico/calico/v3.30.3/manifests/custom-resources.yaml
      ```
    - The last step is to install Calico using the manifest file:
      ```bash  
      kubectl apply -f https://raw.githubusercontent.com/projectcalico/calico/v3.30.3/manifests/calico.yaml
      ```
    
    You can **verify** the Calico installation using this command:
    ```bash  
    watch kubectl get pods -l k8s-app=calico-node -A
    ```
    This is the expected output:
    ```bash  
    NAMESPACE     NAME                READY   STATUS    RESTARTS   AGE
    kube-system   calico-node-2xcf4   1/1     Running   0          57s
    kube-system   calico-node-gkqkg   1/1     Running   0          57s
    kube-system   calico-node-j44hp   1/1     Running   0          57s
    ```


### Common issues while using eBPF in a local setup:
Since CortexBrain uses BPF maps to manage monitoring data with the pinning operation ([What's Pinning?](https://docs.ebpf.io/linux/concepts/pinning/)) and minikube doesn't mount the the BPF file system automatically we need to do it manually:

- First enter into the minikube node with SSH:
  ```bash
  minikube ssh
  ``` 

- Now you need to mount the BPF file system:
  ```bash
  mount -t bpf bpffs /sys/fs/bpf
  ```  
- Since the agent needs to do read and write operations you need to check if the /sys/fs/bpf is in read-write mode:
  ```bash
  ls -ld /sys/fs/bpf
  ```  
This is the expected output:
```bash
drwx-----T 3 root root 0 Sep  4 16:34 /sys/fs/bpf
```  
Now the system can create sub-directories and pin maps into the /sys/fs/bpf directory to temporary store all the acquired data

In case your /sys/fs/bpf doesn't have the right permissions you can use this command to setup the **drwx** permissions:
```bash
chmod 700 /sys/fs/bpf 
```  


## Agent setup

=== "Minikube"
     To set up the CortexFlow Agent, first you have to install the core components. The easiest way to install all the core components is by using `cfcli`.

      - Firstly, install the CLI using the Cargo package manager:
      ```bash  
      cargo install cfcli
      ```
      - Then check where cfcli is installed
      ```bash  
      which cfcli
      ```
      You will get a path like this one */home/[USERNAME]/.cargo/bin/cfcli*

      - The third step is to install the core components
      ```bash  
      cfcli install cortexflow
      ```
      For this step, this is the expected output:
      ```bash 
      [SYSTEM] Founded config files
      [SYSTEM] Readed configs for env variable: "Kubernetes"
      =====> Preparing cortexflow installation
      =====> Creating the config files
      =====> Insert your cluster environment (e.g. Kubernetes)
      ```
      Here you need to enter your cluster environment (e.g. Kubernetes, Docker Swarm, etc...)
      After that, the installation will take place and will take no longer than 1 minute.  
      <details>
      <summary> This is the expected output</summary>
      ```bash 
      [SYSTEM] Configuration files saved in path :"/home/cortexflow/.config/cfcli/config.yaml"


      [SYSTEM] Readed configs for env variable: "Kubernetes"
      =====> Copying installation files
      ✅ Copied file from https://raw.githubusercontent.com/CortexFlow/CortexBrain/refs/heads/main/core/src/testing/configmap.yaml 
      ✅ Copied file from https://raw.githubusercontent.com/CortexFlow/CortexBrain/refs/heads/main/core/src/testing/configmap-role.yaml 
      ✅ Copied file from https://raw.githubusercontent.com/CortexFlow/CortexBrain/refs/heads/main/core/src/testing/rolebinding.yaml 
      ✅ Copied file from https://raw.githubusercontent.com/CortexFlow/CortexBrain/refs/heads/main/core/src/testing/cortexflow-rolebinding.yaml 
      ✅ Copied file from https://raw.githubusercontent.com/CortexFlow/CortexBrain/refs/heads/main/core/src/testing/identity.yaml 
      ✅ Copied file from https://raw.githubusercontent.com/CortexFlow/CortexBrain/refs/heads/feature/ebpf-core/core/src/testing/agent.yaml 


      =====> Creating cortexflow namespace
      =====> Installing cortexflow components
      =====> (1/ 6) Applying  configmap.yaml
      ✅ Applied configmap.yaml
      =====> (2/ 6) Applying  configmap-role.yaml
      ✅ Applied configmap-role.yaml
      =====> (3/ 6) Applying  rolebinding.yaml
      ✅ Applied rolebinding.yaml
      =====> (4/ 6) Applying  cortexflow-rolebinding.yaml
      ✅ Applied cortexflow-rolebinding.yaml
      =====> (5/ 6) Applying  identity.yaml
      ✅ Applied identity.yaml
      =====> (6/ 6) Applying  agent.yaml
      ✅ Applied agent.yaml


      =====> Removing temporary installation files
      ✅ Removed file configmap.yaml
      ✅ Removed file configmap-role.yaml
      ✅ Removed file rolebinding.yaml
      ✅ Removed file cortexflow-rolebinding.yaml
      ✅ Removed file identity.yaml
      ✅ Removed file agent.yaml
      =====> installation completed

      ```

      </details>

      - Now you can check if the install has been successful using the `status` command:
      ```bash 
      cfcli status
      ```
      ```bash 
      🔍 CortexFlow Status Report
      ==================================================

      📦 Namespace Status:
        ✅ cortexflow namespace: EXISTS

      🚀 Pods Status:
        ✅ cortexflow-agent-ffbb95665-l47dw: Running (1/1)
        ✅ cortexflow-identity-7579cd5974-4c9hv: Running (2/2)

      🌐 Services Status:
        🔗 cortexflow-agent: ClusterIP (10.96.88.219)

      ==================================================
      ```
      - The last step is to do a port-forward to let us access the API through the CLI
      ```bash 
      kubectl port-forward svc/cortexflow-agent 9090:9090 -n cortexflow
      ```

      Now the CLI can access the agent service, and you can start calling the agent API with the cfcli `monitoring` commands. At first, let's see which endpoints we can access using the list command

      ```bash 
      cfcli monitoring list
      ```
      This is the expected output with the list of agent functions:
      ```bash 
      =====> Connected to CortexFlow Server Reflection
      Available services:
      ActiveConnections
      ```
      Now we can use the `monitoring connections` command to get the latest detected TCP packets.
      ```bash 
      cfcli monitoring connections
      ```
      ```bash 
      [SYSTEM] Founded config files
      [SYSTEM] Readed configs for env variable: "Kubernetes"
      =====> Connecting to cortexflow Client
      =====> Connected to CortexFlow Client
      {"\"35655\"": "\"143.171.168.192\"", "\"48872\"": "\"133.171.168.192\"", "\"35623\"": "\"148.171.168.192\"", "\"48807\"": "\"173.171.168.192\"", "\"60011\"": "\"136.171.168.192\"", "\"48551\"": "\"163.171.168.192\"", "\"48582\"": "\"129.171.168.192\"", "\"48580\"": "\"133.171.168.192\"", "\"100228\"": "\"147.171.168.192\"", "\"46616\"": "\"133.171.168.192\"", "\"36079\"": "\"136.171.168.192\"", "\"36077\"": "\"136.171.168.192\"", "\"43845\"": "\"131.171.168.192\"", "\"35619\"": "\"136.171.168.192\"", "\"45042\"": "\"135.171.168.192\"", "\"42669\"": "\"148.171.168.192\"", "\"49747\"": "\"143.171.168.192\"", "\"45305\"": "\"147.171.168.192\"", "\"45280\"": "\"147.171.168.192\"", "0": "0", "\"45281\"": "\"147.171.168.192\"", "\"36682\"": "\"136.171.168.192\"", "\"35631\"": "\"148.171.168.192\"", "\"42722\"": "\"148.171.168.192\"", "\"44074\"": "\"136.171.168.192\"", "\"45\"": "\"133.171.168.192\"", "\"44335\"": "\"136.171.168.192\"", "\"35625\"": "\"148.171.168.192\"", "\"32\"": "\"156.171.168.192\"", "\"36073\"": "\"135.171.168.192\"", "\"49748\"": "\"143.171.168.192\"", "\"45282\"": "\"147.171.168.192\"", "\"49380\"": "\"129.171.168.192\"", "\"35620\"": "\"148.171.168.192\"", "\"90399\"": "\"158.171.168.192\"", "\"49077\"": "\"143.171.168.192\"", "\"45312\"": "\"147.171.168.192\"", "\"49383\"": "\"133.171.168.192\"", "\"48581\"": "\"133.171.168.192\"", "\"48809\"": "\"173.171.168.192\"", "\"49379\"": "\"156.171.168.192\"", "\"0\"": "\"173.171.168.192\"", "\"42725\"": "\"148.171.168.192\"", "\"42721\"": "\"148.171.168.192\"", "\"44075\"": "\"136.171.168.192\"", "\"41299\"": "\"135.171.168.192\"", "\"16\"": "\"143.171.168.192\"", "\"44071\"": "\"136.171.168.192\"", "\"42720\"": "\"148.171.168.192\""}
      ```


=== "Kind "
    To set up the CortexFlow Agent, first you have to install the core components. The easiest way to install all the core components is by using `cfcli`.

      - Firstly, install the CLI using the Cargo package manager:
      ```bash  
      cargo install cfcli
      ```
      - Then check where cfcli is installed
      ```bash  
      which cfcli
      ```
      You will get a path like this one */home/[USERNAME]/.cargo/bin/cfcli*

      - The third step is to install the core components
      ```bash  
      cfcli install cortexflow
      ```
      For this step, this is the expected output:
      ```bash 
      [SYSTEM] Founded config files
      [SYSTEM] Readed configs for env variable: "Kubernetes"
      =====> Preparing cortexflow installation
      =====> Creating the config files
      =====> Insert your cluster environment (e.g. Kubernetes)
      ```
      Here you need to enter your cluster environment (e.g. Kubernetes, Docker Swarm, etc...)
      After that, the installation will take place and will take no longer than 1 minute.  
      <details>
      <summary> This is the expected output</summary>
      ```bash 
      [SYSTEM] Configuration files saved in path :"/home/cortexflow/.config/cfcli/config.yaml"


      [SYSTEM] Readed configs for env variable: "Kubernetes"
      =====> Copying installation files
      ✅ Copied file from https://raw.githubusercontent.com/CortexFlow/CortexBrain/refs/heads/main/core/src/testing/configmap.yaml 
      ✅ Copied file from https://raw.githubusercontent.com/CortexFlow/CortexBrain/refs/heads/main/core/src/testing/configmap-role.yaml 
      ✅ Copied file from https://raw.githubusercontent.com/CortexFlow/CortexBrain/refs/heads/main/core/src/testing/rolebinding.yaml 
      ✅ Copied file from https://raw.githubusercontent.com/CortexFlow/CortexBrain/refs/heads/main/core/src/testing/cortexflow-rolebinding.yaml 
      ✅ Copied file from https://raw.githubusercontent.com/CortexFlow/CortexBrain/refs/heads/main/core/src/testing/identity.yaml 
      ✅ Copied file from https://raw.githubusercontent.com/CortexFlow/CortexBrain/refs/heads/feature/ebpf-core/core/src/testing/agent.yaml 


      =====> Creating cortexflow namespace
      =====> Installing cortexflow components
      =====> (1/ 6) Applying  configmap.yaml
      ✅ Applied configmap.yaml
      =====> (2/ 6) Applying  configmap-role.yaml
      ✅ Applied configmap-role.yaml
      =====> (3/ 6) Applying  rolebinding.yaml
      ✅ Applied rolebinding.yaml
      =====> (4/ 6) Applying  cortexflow-rolebinding.yaml
      ✅ Applied cortexflow-rolebinding.yaml
      =====> (5/ 6) Applying  identity.yaml
      ✅ Applied identity.yaml
      =====> (6/ 6) Applying  agent.yaml
      ✅ Applied agent.yaml


      =====> Removing temporary installation files
      ✅ Removed file configmap.yaml
      ✅ Removed file configmap-role.yaml
      ✅ Removed file rolebinding.yaml
      ✅ Removed file cortexflow-rolebinding.yaml
      ✅ Removed file identity.yaml
      ✅ Removed file agent.yaml
      =====> installation completed

      ```

      </details>

      - Now you can check if the install has been successful using the `status` command:
      ```bash 
      cfcli status
      ```
      ```bash 
      🔍 CortexFlow Status Report
      ==================================================

      📦 Namespace Status:
        ✅ cortexflow namespace: EXISTS

      🚀 Pods Status:
        ✅ cortexflow-agent-ffbb95665-l47dw: Running (1/1)
        ✅ cortexflow-identity-7579cd5974-4c9hv: Running (2/2)

      🌐 Services Status:
        🔗 cortexflow-agent: ClusterIP (10.96.88.219)

      ==================================================
      ```
      - The last step is to do a port-forward to let us access the API through the CLI
      ```bash 
      kubectl port-forward svc/cortexflow-agent 9090:9090 -n cortexflow
      ```

      Now the CLI can access the agent service, and you can start calling the agent API with the cfcli `monitoring` commands. At first, let's see which endpoints we can access using the list command

      ```bash 
      cfcli monitoring list
      ```
      This is the expected output with the list of agent functions:
      ```bash 
      =====> Connected to CortexFlow Server Reflection
      Available services:
      ActiveConnections
      ```
      Now we can use the `monitoring connections` command to get the latest detected TCP packets.
      ```bash 
      cfcli monitoring connections
      ```
      ```bash 
      [SYSTEM] Founded config files
      [SYSTEM] Readed configs for env variable: "Kubernetes"
      =====> Connecting to cortexflow Client
      =====> Connected to CortexFlow Client
      {"\"35655\"": "\"143.171.168.192\"", "\"48872\"": "\"133.171.168.192\"", "\"35623\"": "\"148.171.168.192\"", "\"48807\"": "\"173.171.168.192\"", "\"60011\"": "\"136.171.168.192\"", "\"48551\"": "\"163.171.168.192\"", "\"48582\"": "\"129.171.168.192\"", "\"48580\"": "\"133.171.168.192\"", "\"100228\"": "\"147.171.168.192\"", "\"46616\"": "\"133.171.168.192\"", "\"36079\"": "\"136.171.168.192\"", "\"36077\"": "\"136.171.168.192\"", "\"43845\"": "\"131.171.168.192\"", "\"35619\"": "\"136.171.168.192\"", "\"45042\"": "\"135.171.168.192\"", "\"42669\"": "\"148.171.168.192\"", "\"49747\"": "\"143.171.168.192\"", "\"45305\"": "\"147.171.168.192\"", "\"45280\"": "\"147.171.168.192\"", "0": "0", "\"45281\"": "\"147.171.168.192\"", "\"36682\"": "\"136.171.168.192\"", "\"35631\"": "\"148.171.168.192\"", "\"42722\"": "\"148.171.168.192\"", "\"44074\"": "\"136.171.168.192\"", "\"45\"": "\"133.171.168.192\"", "\"44335\"": "\"136.171.168.192\"", "\"35625\"": "\"148.171.168.192\"", "\"32\"": "\"156.171.168.192\"", "\"36073\"": "\"135.171.168.192\"", "\"49748\"": "\"143.171.168.192\"", "\"45282\"": "\"147.171.168.192\"", "\"49380\"": "\"129.171.168.192\"", "\"35620\"": "\"148.171.168.192\"", "\"90399\"": "\"158.171.168.192\"", "\"49077\"": "\"143.171.168.192\"", "\"45312\"": "\"147.171.168.192\"", "\"49383\"": "\"133.171.168.192\"", "\"48581\"": "\"133.171.168.192\"", "\"48809\"": "\"173.171.168.192\"", "\"49379\"": "\"156.171.168.192\"", "\"0\"": "\"173.171.168.192\"", "\"42725\"": "\"148.171.168.192\"", "\"42721\"": "\"148.171.168.192\"", "\"44075\"": "\"136.171.168.192\"", "\"41299\"": "\"135.171.168.192\"", "\"16\"": "\"143.171.168.192\"", "\"44071\"": "\"136.171.168.192\"", "\"42720\"": "\"148.171.168.192\""}
      ```

