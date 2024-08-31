from BaseSensor import Sensor # base class
from Map import IndoorArea
import sys
import os
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '../../')))

from utils.DataProcessor import DataProcessor



# The `CameraSensor` class represents a camera sensor with properties such as position, field of view,
# range, and angle.
class CameraSensor(Sensor):
    # Camera sensor class

    def __init__(self, position, width, range, label="Camera Sensor"):
        super().__init__(SensorType="Camera", value=[0.0, 0.0], label=label)
        self.lat = float(position[0])
        self.lon = float(position[1])
        self.range = float(range)
        self.width = float(width)
        self.label = label
        self.fov = DataProcessor.computeFovAngle(width, self.range)  # evalutate the fov angle
        self.angle = 0  # Default angle

    def SetPosition(self, position):
        self.lat = float(position[0])
        self.lon = float(position[1])

    def SetFov(self, field_of_view):
        self.fov = field_of_view

    def SetRange(self, new_range):
        self.range = new_range
        self.fov = DataProcessor.computeFovAngle(self.width, self.range)  #evaluates the fov angle

    def setAngle(self, new_angle):
        self.angle = new_angle

    def getFov(self):
        return round(self.fov,2)

    def getAngle(self):
        return round(self.angle,2)

    def getRange(self):
        return self.range

    def GetPosition(self):
        return (self.lat, self.lon)
    
    def GetStatus(self):
        """Prints the current status of the GPS sensor."""
        print("-----------------------------")
        print("Sensor Status:")
        print(f"Name: {self.name}")
        print(f"Coordinates: {self.GetPosition()}")
        print(f"Fov: {self.getFov()} °")
        print(f"Angle: {self.getAngle()}° ")
        print(f"Range: {self.getRange()} units")
        print("-----------------------------")

if __name__ == "__main__":
    indoor_area = IndoorArea()

    cam1 = CameraSensor(position=[45.80, 8.953], width=60, range=100, label="cam1")
    cam2 = CameraSensor(position=[45.80, 8.955], width=60, range=100, label="cam2")
    cam3 = CameraSensor(position=[45.80, 8.956], width=110, range=100, label="cam3")

    cam1.setAngle(135)
    cam2.setAngle(135)
    cam3.setAngle(300)
    
    cam1.GetStatus()
    cam2.GetStatus()
    cam3.GetStatus()

    indoor_area.addSensorIndoor(cam1)
    indoor_area.addSensorIndoor(cam2)
    indoor_area.addSensorIndoor(cam3)

    indoor_area.create_map_with_sensors()
