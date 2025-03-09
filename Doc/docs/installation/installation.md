!!! warning
    CortexFlow is currently under active development and, therefore, is not yet available to the public for production use. If you are a developer and would like to contribute or try out CortexFlow, feel free to reach out to us.

## Install from source
To get started with CortexBrain, follow these steps:

- **Clone the Repository**: First, clone the repository to your local machine.

   ```bash
   git clone https://github.com/CortexFlow/CortexBrain.git
   ```
- **Install required packages**:

   | **Feature**              | **Requirements**                                                                 |
   | ------------------------- | -------------------------------------------------------------------------------- |
   | **CortexBrain Core**      | - Kubernetes or Minikube v1.34.0  <br> - Linux Ubuntu system (preferred for development)  <br> - Rust programming language (rustc >= 1.83.0)|
   | **CortexBrain Dashboard** | - npm v10.7.0  <br> - React v18.2.0  <br> - Electron v33.2.0                      |

## **Core Development**  
   1. Install Rust using RustUp tools : 
      ```bash
      curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
      ```  
   2. Install [Docker](https://www.docker.com/get-started/):  
      ```bash
      https://www.docker.com/get-started/
      ```  
   3. Install [Minikube](https://minikube.sigs.k8s.io/docs/start/?arch=%2Fwindows%2Fx86-64%2Fstable%2F.exe+download)  
   4. Run minikube
      ```bash
      minikube start
      ```
## **Dashboard Development**  
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
