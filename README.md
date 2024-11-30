<div align="center" style="display: flex; justify-content: left; align-items: center; height: 100px;">
  <img src="https://www.cortexflow.org/favicon.svg" alt="Logo" width="60" height="60">
  <span style="margin-left: 10px; font-size:25px">CortexBrain © 2024</span>
</div>

[![Release](https://img.shields.io/badge/Release-Currently%20under%20development-red?style=flat-square&logo=github)](https://github.com/CortexFlow/CortexBrain/releases) 
![Auto Assign](https://img.shields.io/github/actions/workflow/status/CortexFlow/CortexBrain/auto-assign.yml?style=flat-square&logo=github&logoColor=white)
[![Docker](https://img.shields.io/badge/Docker-Containerized-%232496ED.svg?style=flat-square&logo=docker&logoColor=white)](https://www.docker.com)
[![Trello](https://img.shields.io/badge/Trello-Project%20Management-%23026AA7.svg?style=flat-square&logo=Trello&logoColor=white)](https://trello.com/invite/b/66c731aab6030598aef7aed3/ATTIdfd7d08e42dca6f8b56a8b26f499ab8c95EB547E/cortexbrain)
[![Documentation](https://img.shields.io/badge/Docs-In%20Progress-red?style=flat-square&logo=readthedocs&logoColor=white)](./doc.md)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg?style=flat-square&logo=open-source-initiative&logoColor=white)](./LICENSE)
[![Discussions](https://img.shields.io/github/discussions/CortexFlow/CortexBrain?style=flat-square&logo=github-discussions&logoColor=white)](https://github.com/CortexFlow/CortexBrain/discussions)
[![Contributors](https://img.shields.io/badge/Contributors-Welcome-brightgreen?style=flat-square&logo=github&logoColor=white)](https://github.com/CortexFlow/CortexBrain#contributing)
[![Kubernetes](https://img.shields.io/badge/Kubernetes-Orchestrator-%23326CE5.svg?style=flat-square&logo=Kubernetes&logoColor=white)](https://kubernetes.io)  
  
## 📬Contacts

- **Tettamanti Lorenzo**  [📧 lorenzotettamanti5@gmail.com](mailto:lorenzotettamanti5@gmail.com)

- **Lorenzo Bradanini**  [📧 lorenzolollobrada@gmail.com](mailto:lorenzolollobrada@gmail.com)

## 🧑‍💻What is CortexBrain?

CortexBrain is a cutting-edge data simulation and big data analysis framework developed by CortexFlow, designed to simplify the simulation and analysis of IoT (Internet of Things) devices. CortexBrain enables you to model and train various IoT sensor properties, including smart lights, accelerometers, gyroscopes, and temperature sensors. With CortexBrain, you can simulate IoT sensors, position them on a geolocalized map, monitor key metrics, and simulate specific scenarios.

## ⚛️Currently Development Focus

Currently working on:

- creating examples
- writing documentation
- working on predictive mantainance
- expanding actual features
- creating new frameworks
- integrating big data analysis tools

# 🧪Features

CortexBrain provides a range of powerful features, including:

- **IoT Sensor Simulation**: Simulate a variety of IoT sensors, such as temperature, humidity, and light sensors.
- **Geolocalized Map**: Position simulated sensors on a geolocalized map.
- **Scenario Simulation**: Create and test various scenarios with simulated sensors.
-  **Advanced data analysis**: Leverage big data tools for in-depth analysis and seamless visualization of simulated data.
-  **Machine Learning Integration**: Implement and test machine learning models on your simulated data to uncover patterns and make predictions.
-  **Predictive Maintenance**: Utilize the simulated data for predictive maintenance, identifying potential issues before they occur.
-  **Real-time Data Streams**: Integrate real-time data streams to enhance the accuracy and responsiveness of your simulations.  

# 🤖 Getting Started
## 🐋 Install with Docker
## 🥷 Install from source
To get started with CortexBrain, follow these steps:

1. **Clone the Repository**: First, clone the repository to your local machine.

   ```bash
   git clone https://github.com/CortexFlow/CortexBrain.git
    ```

2. **Install required packages**:

   ```bash
   cd CortexBrain
   pip install -r requirements.txt

3. **Test Library**:

   ```bash
   python .\checkLibs.py

4. **Create a simple program**:

   ```bash
   import os
   import sys
   sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '../')))

   from GPSModel import GPS_Sensor
   from Map import Map

   if __name__=="__main__":
      #create the sensors using the constructors
      gps_sensor_1 = GPS_Sensor(initial_position=[0, 0], label="Static GPS Sensor ")
      gps_sensor_2 = GPS_Sensor(initial_position=[0, 0], label="Static GPS Sensor 2")
      
      #set the positions
      gps_sensor_1.SetPosition((45.812460, 8.986586))
      gps_sensor_2.SetPosition((45.832460, 8.986586))
      
      #create the map
      map=Map()
      #add the sensors on the map
      map.AddSensor(gps_sensor_1)
      map.AddSensor(gps_sensor_2)
      #create the sensor_map.html file
      map.CreateMap()



Check all the examples in the [Examples](./Examples/) folder
Explore all the features in the [documentation](doc.md).

# Documentation


# Future Developments

At CortexFlow, we're continually working to expand the capabilities of CortexBrain. Our current focus includes:

- **Enhanced Scenario Simulation**: We're improving the scenario simulation engine to support more complex and dynamic environments.
- **Real-time Data Integration**: Future updates will allow CortexBrain to integrate real-time data streams, making the simulations even more accurate and responsive.
- **API Expansion**: We're actively working on extending our API to give developers more flexibility and control over their simulations.

To stay updated on our progress and view what we're currently working on, check out our [Trello board](https://trello.com/invite/b/66c731aab6030598aef7aed3/ATTIdfd7d08e42dca6f8b56a8b26f499ab8c95EB547E/cortexbrain).

# 🖥️ Contributing

We welcome contributions from the community! To contribute to the project, please follow these steps:

1. Fork the repository.
2. Create a new branch for your feature (`git checkout -b feature/feature-name`).
3. Submit a Pull Request with a detailed explanation of your changes.

## 🙋**Proposing New Features**

If you would like to contribute a new feature to the project, we ask that you open a discussion before submitting a PR. This is to ensure that all new features align with the project's goals and to avoid overlapping work or conflicting views.

Please initiate a discussion in the [GitHub Discussions](https://github.com/CortexFlow/CortexBrain/discussions) section where we can collectively review, refine, and approve your idea before you begin implementation. Pull Requests for new features that have not been discussed beforehand may be declined to maintain project coherence and ensure alignment with the broader roadmap.

By collaborating in this manner, we can maintain clarity and consistency, ensuring that all contributors are working towards the same objectives. Thank you for your understanding and contributions!

## 🐐 Top contributors
[![Top contributors](https://images.repography.com/54717595/CortexFlow/CortexBrain/top-contributors/bRL3WTk3lP0LlkiA2QM-GAH_NLqgBwcXYg8aH_s_9Fg/_YHQeQ-ptyH2aRy6rfxNfiMSSDWLoxKWQgKovd2sKJM_table.svg)](https://github.com/CortexFlow/CortexBrain/graphs/contributors)
