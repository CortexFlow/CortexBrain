# README

We're currently moving the files from an old project. Please contact <lorenzotettamanti5@gmail.com> before cloning the repository
Currently working on:

- moving and testing all the models
- creating examples
- writing documentation
- expanding actual features
- creating new frameworks
- integrating big data analysis tools

# CortexBrain

Copyright (c) 2024

- Author: Tettamanti Lorenzo, Lorenzo Bradanini
- Contact: <lorenzotettamanti5@gmail.com>/<lorenzolollobrada@gmail.com>
![Auto Assign](https://github.com/CortexFlow/CortexBrain/actions/workflows/auto-assign.yml/badge.svg)
![CortexFlow Logo](banner.png)

## What is CortexBrain?

CortexBrain is a cutting-edge data simulation and big data analysis framework developed by CortexFlow, designed to simplify the simulation and analysis of IoT (Internet of Things) devices. CortexBrain enables you to model and train various IoT sensor properties, including smart lights, accelerometers, gyroscopes, and temperature sensors. With CortexBrain, you can simulate IoT sensors, position them on a geolocalized map, monitor key metrics, and simulate specific scenarios.


# Features

CortexBrain provides a range of powerful features, including:

- **IoT Sensor Simulation**: Simulate a variety of IoT sensors, such as temperature, humidity, and light sensors.
- **Geolocalized Map**: Position simulated sensors on a geolocalized map.
- **Scenario Simulation**: Create and test various scenarios with simulated sensors.
-  **Advanced data analysis**: Leverage big data tools for in-depth analysis and seamless visualization of simulated data.
-  **Machine Learning Integration**: Implement and test machine learning models on your simulated data to uncover patterns and make predictions.
-  **Predictive Maintenance**: Utilize the simulated data for predictive maintenance, identifying potential issues before they occur.
-  **Real-time Data Streams**: Integrate real-time data streams to enhance the accuracy and responsiveness of your simulations.


# Getting Started

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
   python .\testLib.py

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

Open [sensor_map_tutorial.html](./SyntheticDatas/Sensors/Examples/sensor_map_tutorial.html) and check the sensors on the map
![tutorial.png](./SyntheticDatas/Sensors/Examples/img/tutorial.png)

Check all the examples in the [Examples](./Examples/) folder
Explore all the features in the [documentation](doc.md).

# Roadmap

![RoadMap](ROADMAP.png)

# Documentation

For a comprehensive guide on getting started and making the most of CortexBrain, visit the [official documentation](doc.md). The documentation includes:

- **Getting Started**: Instructions on how to install and set up CortexBrain.
- **Examples**: Practical examples to help you understand and use the main features.
- **API Reference**: Information on the available APIs and their usage (coming soon).

# Future Developments

At CortexFlow, we're continually working to expand the capabilities of CortexBrain. Our current focus includes:

- **Enhanced Scenario Simulation**: We're improving the scenario simulation engine to support more complex and dynamic environments.
- **Real-time Data Integration**: Future updates will allow CortexBrain to integrate real-time data streams, making the simulations even more accurate and responsive.
- **API Expansion**: We're actively working on extending our API to give developers more flexibility and control over their simulations.

To stay updated on our progress and view what we're currently working on, check out our [Trello board](https://trello.com/invite/b/66c731aab6030598aef7aed3/ATTIdfd7d08e42dca6f8b56a8b26f499ab8c95EB547E/cortexbrain).

# Contributing

We welcome contributions from the community! To contribute to the project, please follow these steps:

1. Fork the repository.
2. Create a new branch for your feature (`git checkout -b feature/feature-name`).
3. Submit a Pull Request with a detailed explanation of your changes.


**Proposing New Features**

If you would like to contribute a new feature to the project, we ask that you open a discussion before submitting a PR. This is to ensure that all new features align with the project's goals and to avoid overlapping work or conflicting views.

Please initiate a discussion in the [GitHub Discussions](https://github.com/CortexFlow/CortexBrain/discussions) section where we can collectively review, refine, and approve your idea before you begin implementation. Pull Requests for new features that have not been discussed beforehand may be declined to maintain project coherence and ensure alignment with the broader roadmap.

By collaborating in this manner, we can maintain clarity and consistency, ensuring that all contributors are working towards the same objectives. Thank you for your understanding and contributions!
