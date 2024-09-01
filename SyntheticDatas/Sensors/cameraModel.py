from BaseSensor import Sensor # base class
import sys
import os
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '../../')))

from utils.DataProcessor import DataProcessor
from Map import CameraMap



# The `CameraSensor` class represents a camera sensor with properties such as position, field of view,
# range, and angle.
class CameraSensor(Sensor):
    # Camera sensor class

    def __init__(self, position, sensor_width, focal_length, label="Camera Sensor"):
        super().__init__(SensorType="Camera", value=[0.0, 0.0], label=label)
        self.lat = float(position[0])
        self.lon = float(position[1])
        self.focal_length = float(focal_length)
        self.sensor_width = float(sensor_width)
        self.label = label
        self.fov = DataProcessor.computeFovAngle(sensor_width, self.focal_length)  # evalutate the fov angle
        self.angle = 0  # Default angle
        self.max_range = DataProcessor.computeMaxDistance(self.fov,self.sensor_width) #max distance covered

    def SetPosition(self, position):
        self.lat = float(position[0])
        self.lon = float(position[1])

    def SetFov(self, field_of_view):
        self.fov = field_of_view

    def SetFocalLenght(self, new_focal):
        self.focal_length = new_focal
        self.fov = DataProcessor.computeFovAngle(self.sensor_width, self.focal_length)  #evaluates the fov angle

    def SetAngle(self, new_angle):
        self.angle = new_angle
    
    def SetSensorWidth(self,new_sensor_width):
        self.sensor_width=new_sensor_width

    def getFov(self):
        return round(self.fov,2)

    def getAngle(self):
        return round(self.angle,2)

    def getFocalLenght(self):
        return self.focal_length

    def getPosition(self):
        return (self.lat, self.lon)
    
    def getSensorWidth(self):
        return self.sensor_width
    
    def getMaxDistance(self):
        return self.max_range
    
    def getStatus(self):
        """Prints the current status of the GPS sensor."""
        print("-----------------------------")
        print("Sensor Status:")
        print(f"Name: {self.name}")
        print(f"Coordinates: {self.getPosition()}")
        print(f"Focal Lenght: {self.getFocalLenght()} mm")
        print(f"Sensor Width: {self.getSensorWidth()} mm ")
        print(f"Fov: {self.getFov()} °")
        print(f"Max Distance covered: {self.getMaxDistance()} m")
        print(f"Angle: {self.getAngle()}° ")
        print("-----------------------------")

if __name__ == "__main__":
    cameraMap = CameraMap()

    cam1 = CameraSensor(position=[45.80, 8.953], sensor_width=12.8, focal_length=15.9, label="cam1")
    cam2 = CameraSensor(position=[45.80, 8.955], sensor_width=12.8, focal_length=15.9, label="cam2")
    cam3 = CameraSensor(position=[45.80, 8.956], sensor_width=12.8, focal_length=15.9, label="cam3")
    cam4 = CameraSensor(position=[45.803, 8.953], sensor_width=12.8, focal_length=15.9, label="cam4")
    cam5 = CameraSensor(position=[45.8025, 8.956], sensor_width=12.8, focal_length=15.9, label="cam5")

    cam1.SetAngle(135)
    cam2.SetAngle(135)
    cam3.SetAngle(150)
    cam4.SetAngle(135)
    cam5.SetAngle(135)
    
    
    cam1.getStatus()
    cam2.getStatus()
    cam3.getStatus()
    cam4.getStatus()
    cam5.getStatus()

    cameraMap.addSensor(cam1)
    cameraMap.addSensor(cam2)
    cameraMap.addSensor(cam3)
    cameraMap.addSensor(cam4)
    cameraMap.addSensor(cam5)

    cameraMap.CreateMap()
