import numpy as np
import time
import sys
import os

# Add parent directories to the system path
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '../../')))

import networkx as nx

from Map import Map

from BaseSensor import Sensor


class GPS_Sensor(Sensor):
    """Class for GPS sensors that can be static or moving."""

    def __init__(self, initial_position, speed=0.0, direction='N', label="GPS Sensor"):
        super().__init__(SensorType="GPS", initial_position=[0.0,0.0], value=[0.0, 0.0], label=label)
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


if __name__ == "__main__":
    # Test the Sensor class
    sensor = Sensor(SensorType="Generic Sensor", value=45.234, label="Sensor 1")
    print(f"Sensor Name: {sensor.name}")
    print(f"Sensor Value: {sensor.ReadValue()}")
    print(f"Sensor Type: {sensor.type}")
    sensor.UpdateValue(45.3)
    print(f"Updated Sensor Value: {sensor.ReadValue()}")

    print("\nTesting GPS Sensor (Static)")
    static_gps_sensor = GPS_Sensor(initial_position=[45.712460, 8.986586], label="Static GPS Sensor")
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
    
    # Testing the Map class
    static_gps_sensor2 = GPS_Sensor(initial_position=[45.812460, 8.986586], label="Static GPS Sensor 2")

    # Create a Map instance and add sensors
    map = Map()
    map.AddSensor(static_gps_sensor)
    map.AddSensor(static_gps_sensor2)

    # Generate the map
    map.CreateMap()