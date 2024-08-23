import numpy as np
import time
import sys
import os

# Add parent directories to the system path
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '../../')))

import networkx as nx
import folium

class Sensor:
    """Base class for all sensors."""

    def __init__(self, SensorType, value=0.0, label="Sensor"):
        self.name = label
        self.type = SensorType
        self.value = value

    def __del__(self):
        print(f"Sensor {self.type}, name: {self.name} deleted")

    def ReadValue(self):
        """Returns the current value of the sensor."""
        return self.value

    def UpdateValue(self, new_value):
        """Updates the sensor's value."""
        self.value = new_value
        return self.value


class GPS_Sensor(Sensor):
    """Class for GPS sensors that can be static or moving."""

    def __init__(self, initial_position, speed=0.0, direction='N', label="GPS Sensor"):
        super().__init__(SensorType="GPS", value=[0.0, 0.0], label=label)
        self.lat = float(initial_position[0])
        self.lon = float(initial_position[1])
        self.speed = float(speed)
        self.direction = direction.upper()
        self.is_moving = self.speed > 0

        # Initialize position
        self.SetPosition(initial_position)

    def SetPosition(self, position):
        """Sets the GPS sensor's position."""
        self.value = [position[0], position[1]]
        self.lat = float(position[0])
        self.lon = float(position[1])
        return self.value

    def GetSpeed(self):
        """Returns the speed of the GPS sensor."""
        return self.speed

    def SetSpeed(self, speed):
        """Sets the speed of the GPS sensor."""
        self.speed = float(speed)
        self.is_moving = self.speed > 0
        return self.speed

    def isMoving(self):
        """Returns whether the GPS sensor is moving."""
        return self.is_moving

    def GetStatus(self):
        """Prints the current status of the GPS sensor."""
        print("-----------------------------")
        print("Sensor Status:")
        print(f"Name: {self.name}")
        print(f"Coordinates: {self.value}")
        print(f"Speed: {self.speed} units/sec")
        print(f"Latitude: {self.lat}")
        print(f"Longitude: {self.lon}")
        print(f"Is Moving: {'Yes' if self.is_moving else 'No'}")
        print("-----------------------------")

    def UpdateDirection(self):
        """Updates the position based on the speed and direction."""
        if not self.is_moving:
            return
        
        if self.direction == 'N':
            delta = np.array([0.0, 1.0])  # Movement towards North (positive y-direction)
        elif self.direction == 'S':
            delta = np.array([0.0, -1.0])  # Movement towards South (negative y-direction)
        elif self.direction == 'E':
            delta = np.array([1.0, 0.0])  # Movement towards East (positive x-direction)
        elif self.direction == 'W':
            delta = np.array([-1.0, 0.0])  # Movement towards West (negative x-direction)
        else:
            raise ValueError("Invalid direction. Use 'N', 'S', 'E', or 'W'.")
        
        # Normalize the delta based on speed
        delta_normalized = delta * self.speed
        
        # Update the position
        new_position = np.array(self.value) + delta_normalized
        self.SetPosition(new_position)

    def SimulateMovement(self, duration):
        """Simulates the movement of the GPS sensor for a given duration."""
        if not self.is_moving:
            print("Sensor is not moving.")
            return
        
        start_time = time.time()
        while time.time() - start_time < duration:
            self.UpdateDirection()
            print(f"Current Position: {self.ReadValue()}")
            time.sleep(1)  # Wait for 1 second before updating again: Simulate the refresh rate of the sensor

        print(f"Final Position: {self.ReadValue()}")
        
    def PlaceSensor(self, pois):
            """Places the sensor on the map and adds markers for each POI."""
            # Create a graph and add nodes for each POI
            G = nx.Graph()
            for poi in pois:
                # Here we assume pois is a list of tuples where each tuple contains (name, position)
                name, position = poi
                G.add_node(name, pos=(position[0], position[1]), category=name)
            
            # Initialize the map centered around the average position of all POIs
            pos = nx.get_node_attributes(G, 'pos')
            if not pos:
                print("No positions found in the graph.")
                return

            # Compute the center position (average of all positions) for map initialization
            avg_lat = np.mean([p[0] for p in pos.values()])
            avg_lon = np.mean([p[1] for p in pos.values()])
            center_position = [avg_lat, avg_lon]

            mappa = folium.Map(location=center_position, zoom_start=13,
                            tiles='http://mt1.google.com/vt/lyrs=s&x={x}&y={y}&z={z}', attr='Google')
            
            # Add markers for each node on the map
            for node, coords in pos.items():
                # Custom marker for POIs with informational popup
                popup_content = f"<strong>{node}</strong><br>Type: POI"
                folium.Marker(
                    location=coords,
                    popup=popup_content,
                    icon=folium.Icon(color='blue', icon='cloud')
                ).add_to(mappa)
            
            # Save the map to an HTML file or display it in a Jupyter notebook
            mappa.save('sensor_map.html')
            print("Map has been saved to 'sensor_map.html'.")

    def __del__(self):
        print(f"Sensor {self.type}, name: {self.name} deleted")


if __name__ == "__main__":
    # Test the Sensor class
    sensor = Sensor(SensorType="Generic Sensor", value=45.234, label="Sensor 1")
    print(f"Sensor Name: {sensor.name}")
    print(f"Sensor Value: {sensor.ReadValue()}")
    print(f"Sensor Type: {sensor.type}")
    sensor.UpdateValue(45.3)
    print(f"Updated Sensor Value: {sensor.ReadValue()}")

    print("\nTesting GPS Sensor (Static)")
    static_gps_sensor = GPS_Sensor(initial_position=[0.0, 0.0], label="Static GPS Sensor")
    print(f"Sensor Name: {static_gps_sensor.name}")
    print(f"Sensor Coordinates: {static_gps_sensor.ReadValue()}")
    static_gps_sensor.GetStatus()

    print("\nTesting GPS Sensor (Moving)")
    start_position = [0.0, 0.0]
    speed = 3.11  # Speed in units per second
    direction = 'E'  # Direction (N, S, E, W)
    
    moving_gps_sensor = GPS_Sensor(initial_position=start_position, speed=speed, direction=direction, label="Moving GPS Sensor")
    
    print(f"Sensor Name: {moving_gps_sensor.name}")
    print(f"Initial Position: {moving_gps_sensor.ReadValue()}")
    print(f"Speed: {moving_gps_sensor.GetSpeed()}")
    print(f"Direction: {moving_gps_sensor.direction}")
    print(f"Is Moving? {'Yes' if moving_gps_sensor.isMoving() else 'No'}")
    print("\nStarting GPS sensor simulation...")
    moving_gps_sensor.SimulateMovement(duration=10)  # Simulate for 10 seconds
    
    # Test placing sensors on the map
    pois = [
        ("Static GPS Sensor", [45.712460, 8.986586]),
        ("Static GPS Sensor2", [45.812460, 8.986586])
    ]
    static_gps_sensor.PlaceSensor(pois=pois)
