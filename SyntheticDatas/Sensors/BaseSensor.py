import networkx as nx
import sys
import os

# Add parent directories to the system path
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))
sys.path.append(os.path.abspath(
    os.path.join(os.path.dirname(__file__), '../../')))


class Sensor:
    """Base class for all sensors."""

    def __init__(self, SensorType, initial_position=(0.0, 0.0), value=0.0, label="Sensor"):
        self.name = label
        self.type = SensorType
        self.value = value
        self.position = initial_position  # Add position attribute

    def __del__(self):
        print(f"Sensor {self.type}, name: {self.name} deleted")

    def ReadValue(self):
        """Returns the current value of the sensor."""
        return self.value

    def UpdateValue(self, new_value):
        """Updates the sensor's value."""
        self.value = new_value
        return self.value


if __name__ == "__main__":
    # this is a dumb sensor
    sensor = Sensor("DumbSensor", initial_position=(
        0.0, 0.0), value=0, label="dumb sensor")
    print("Reading the initial value from the dumb sensor", sensor.ReadValue())
    sensor.UpdateValue(3.1)
    print("Updated Values", sensor.ReadValue())
