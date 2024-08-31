import numpy as np
import networkx as nx
import folium

from BaseSensor import Sensor  # Make sure this import is correct


class Map:
    """Class responsible for creating and managing the map with sensors."""

    def __init__(self):
        self.G = nx.Graph()

    def AddSensor(self, sensor):
        """Adds a sensor to the graph."""
        if not isinstance(sensor, Sensor):
            raise TypeError("sensor must be an instance of Sensor")

        self.G.add_node(sensor.name, pos=sensor.value, category=sensor.type)

    def CreateMap(self):
        """Creates and saves the map with all sensors added."""
        pos = nx.get_node_attributes(self.G, 'pos')
        if not pos:
            print("No positions found in the graph.")
            return

        # Compute the center position (average of all positions) for map initialization
        avg_lat = np.mean([p[0] for p in pos.values()])
        avg_lon = np.mean([p[1] for p in pos.values()])
        center_position = [avg_lat, avg_lon]

        # Create the map with the calculated center position
        map = folium.Map(location=center_position, zoom_start=15,
                         tiles='https://mt1.google.com/vt/lyrs=s&x={x}&y={y}&z={z}', attr='Google')

        # Add markers for each node on the map
        for node, coords in pos.items():
            # Custom marker for sensors with informational popup
            popup_content = f"<strong>{node}</strong><br>Type: {self.G.nodes[node]['category']}"
            folium.Marker(
                location=coords,
                popup=popup_content,
                icon=folium.Icon(color='blue', icon='cloud')
            ).add_to(map)

        # Save the map to an HTML file or display it in a Jupyter notebook
        map.save('sensor_map.html')
        print("Map has been saved to 'sensor_map.html'.")
