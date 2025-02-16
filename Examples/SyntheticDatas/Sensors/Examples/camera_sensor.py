import sys
import os
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '../')))

from Map import CameraMap
from BaseSensor import Sensor # base class
from cameraModel import CameraSensor

if __name__ == "__main__":
    cameraMap = CameraMap()

    cam1 = CameraSensor(position=[45.80, 8.953], sensor_width=12.8, focal_length=15.9, label="cam1")

    cam1.SetAngle(100)

    
    cam1.getStatus()


    cameraMap.addSensor(cam1)

    cameraMap.CreateMap()
