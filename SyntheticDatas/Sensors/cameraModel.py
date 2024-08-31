import numpy as np
import math
import folium

from BaseSensor import Sensor # base class


def computeFovAngle(width, distance):
    """Calculates the visual field angle (angular FoV) based on width and distance."""
    return 2 * math.degrees(math.atan(width / (2 * distance)))

def generateCameraPoints(center_lat, center_lon, angle, fov_angle, radius, num_points=100):
    """Generate points for a semicircle centered on (center_lat, center_lon)."""
    points = []
    angle_rad = math.radians(angle)
    fov_angle_rad = math.radians(fov_angle)
    
    # Calculate the points of the semicircle
    for theta in np.linspace(-fov_angle_rad / 2, fov_angle_rad / 2, num_points):
        x = radius * math.cos(theta)
        y = radius * math.sin(theta)
        
        # Rotazione dei punti in base all'angolo del sensore
        x_rot = x * math.cos(angle_rad) - y * math.sin(angle_rad)
        y_rot = x * math.sin(angle_rad) + y * math.cos(angle_rad)
        
        # Conversion of rotational coordinates to latitude and longitude
        lat_gps = center_lat + y_rot / 111000
        lon_gps = center_lon + x_rot / (111000 * math.cos(math.radians(center_lat)))
        points.append([lat_gps, lon_gps])
    
    points.append([center_lat, center_lon])
    points.append([center_lat + radius / 111000, center_lon])
    
    return points

class IndoorArea:
    def __init__(self):
        self.sensors = []

    def addSensorIndoor(self, sensor):
        self.sensors.append(sensor)

    def create_map_with_sensors(self):
        folium_map = folium.Map(location=[45.81, 8.98], zoom_start=14,
                                tiles='https://mt1.google.com/vt/lyrs=s&x={x}&y={y}&z={z}', attr='Google Maps')

        for sensor in self.sensors:
            folium.Marker(
                location=[sensor.lat, sensor.lon],
                popup=f"""
                <strong>{sensor.label}</strong><br>
                Angle: {sensor.angle}°<br>
                FOV: {sensor.fov}°<br>
                Range: {sensor.range}
                """,
                icon=folium.Icon(color='blue', icon='cloud')
            ).add_to(folium_map)

            points = generateCameraPoints(sensor.lat, sensor.lon, sensor.angle, sensor.fov, sensor.range)
            if points:
                folium.Polygon(
                    locations=points,
                    color='green',
                    fill=True,
                    fill_color='green',
                    fill_opacity=0.4
                ).add_to(folium_map)

        folium_map.save("sensor_map.html")
        print("Folium map with sensors and fields of view has been saved to 'sensor_map.html'.")


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
        self.fov = computeFovAngle(width, self.range)  # evalutate the fov angle
        self.angle = 0  # Default angle

    def SetPosition(self, position):
        self.lat = float(position[0])
        self.lon = float(position[1])

    def SetFov(self, field_of_view):
        self.fov = field_of_view

    def SetRange(self, new_range):
        self.range = new_range
        self.fov = computeFovAngle(self.width, self.range)  #evaluates the fov angle

    def setAngle(self, new_angle):
        self.angle = new_angle

    def getFov(self):
        return self.fov

    def getAngle(self):
        return self.angle

    def getRange(self):
        return self.range

    def GetPosition(self):
        return (self.lat, self.lon)

if __name__ == "__main__":
    indoor_area = IndoorArea()

    cam1 = CameraSensor(position=[45.80, 8.953], width=60, range=100, label="cam1")
    cam2 = CameraSensor(position=[45.80, 8.955], width=60, range=100, label="cam2")
    cam3 = CameraSensor(position=[45.80, 8.956], width=110, range=100, label="cam3")

    cam1.setAngle(135)
    cam2.setAngle(135)
    cam3.setAngle(300)

    indoor_area.addSensorIndoor(cam1)
    indoor_area.addSensorIndoor(cam2)
    indoor_area.addSensorIndoor(cam3)

    indoor_area.create_map_with_sensors()
