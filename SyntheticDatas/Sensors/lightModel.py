from BaseSensor import Sensor
from Map import LightMap
import math


# The `Light` class represents a smart light sensor with properties such as position, power, lumen,
# height, diffusion angle, and orientation angle, along with methods to get and set these properties
# and compute the maximum range covered by the light sensor.

# Add Photometric Curves --->coming soon
# Improvements in the max range covered --> coming soon
class Light(Sensor):
    def __init__(self, position, power, lumen, height, diffusion_angle, orientation_angle, label="Smart Light"):
        super().__init__(SensorType="Light", value=[0.0, 0.0], label=label)
        self.lat = float(position[0])
        self.lon = float(position[1])
        self.power = power
        self.lumen = lumen
        self.label = label
        self.height = height
        self.diffusion_angle = diffusion_angle
        self.angle = orientation_angle

        self.light_efficiency = self.lumen/self.power

    def SetPosition(self, position):
        self.lat = float(position[0])
        self.lon = float(position[1])

    def getPosition(self):
        return (self.lat, self.lon)

    def getLumen(self):
        return self.lumen

    def getPower(self):
        return self.power

    def getLightEfficiency(self):
        return self.light_efficiency

    def getHeight(self):
        return self.height

    def getDiffusionAngle(self):
        return self.diffusion_angle

    def setAngle(self, new_angle):
        self.angle = new_angle
        return self.angle

    def getAngle(self):
        return self.angle

    def computeMaxRange(self):
        return round(self.height*(math.tan(math.radians(120/2))), 3)

    def getStatus(self):
        """Prints the current status of the light sensor."""
        print("-----------------------------")
        print("Sensor Status:")
        print(f"Name: {self.name}")
        print(f"Coordinates: {self.getPosition()}")
        print(f"Power: {self.getPower()} W")
        print(f"Lumen: {self.getLumen()} lm")
        print(f"Height: {self.getHeight()} m")
        print(f"Diffusion Angle: {self.getDiffusionAngle()}° ")
        print(f"Orientation Angle: {self.getAngle()}° ")
        print(f"Max Range Covered: {self.computeMaxRange()} m ")
        print(f"Light Efficiency: {self.getLightEfficiency()} lm/W")
        print("-----------------------------")


if __name__ == "__main__":
    map = LightMap()
    light = Light(position=[45.80, 8.953], power=100,
                  lumen=20000, height=8, diffusion_angle=120, 
                  orientation_angle=180, label="light 1")
    light.getStatus()
    map.addSensor(light)
    map.CreateMap()
