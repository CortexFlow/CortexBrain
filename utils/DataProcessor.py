import matplotlib.pyplot as plt
from matplotlib import style 
import json

import math
import numpy as np

class DataProcessor:
    def CompareDatas1D(data, key):
        try:
            datas = json.loads(data)  # Convert JSON string to a dictionary
        except json.JSONDecodeError:
            print("Error decoding JSON.")
            return None

        # Check if the key "Hotel List" exists and if the list is not empty
        if "Hotel List" not in datas or not datas["Hotel List"]:
            print("No data available")
            return None
        
        # Access the list under the main key 'Hotel List'
        hotel_list = datas["Hotel List"]
        
        # Verify that the key exists in each dictionary in the list
        if not all(key in entry for entry in hotel_list):
            print(f"The key '{key}' is not present in all list items.")
            return None
        
        # Extract the dates and the values associated with the provided key
        dates = [entry["date"] for entry in hotel_list]
        setY = [entry[key] for entry in hotel_list]
        
        # Create the plot
        plt.figure(figsize=(10, 6))
        plt.style.use('ggplot')
        plt.plot(dates, setY, color='b', marker='o')
        plt.xticks(fontsize=8)  # Adjust font size as needed
        
        # Add titles and axis labels
        plt.title(f'({key}) Over Time')
        plt.xlabel('Date')
        plt.ylabel(f'({key})')
        
        # Display the plot
        plt.grid(True)
        plt.xticks(rotation=45)  # Rotate date labels for better readability
        plt.tight_layout()  # Prevent label cutting
        plt.show()

        return None
    
    
    def CompareDatas2D(data, key1, key2):
        # Access the list under the main key 'Hotel List'
        hotel_list = data['Hotel List']
        
        # Extract the values using the provided keys
        setX = [entry[key1] for entry in hotel_list]
        setY = [entry[key2] for entry in hotel_list]
        
        # Create the scatter plot
        plt.figure(figsize=(20,12))
        plt.style.use('ggplot')  # Apply ggplot style
        plt.scatter(setX, setY, color='b')
        
        # Add titles and axis labels
        plt.title(f'Comparison of ({key1}) and ({key2})')
        plt.xlabel(f'({key1})')
        plt.ylabel(f'({key2})')
        
        # Display the plot
        plt.grid(True)
        plt.show()
        return 0
    
    def calculate_distance_factor(distance):
        """Calculate the distance factor based on the provided distance ranges.
        In version 2 of the generator, a multiplicative factor is used.
        """
        if 0.5 <= distance <= 1.5:
            return 20
        elif 1.5 < distance <= 2:
            return 10
        elif 2 < distance <= 5:
            return 1 / distance
        elif 5 < distance <= 10:
            return 1 / (distance * distance)
        else:
            return 1 / 100
        
    def distribuisci_persone(numero_persone, camere_singole, camere_doppie, camere_triple):
        singole_utilizzate = 0
        doppie_utilizzate = 0
        triple_utilizzate = 0
        
        # Distribute people into triple rooms
        while numero_persone >= 3 and camere_triple > 0:
            numero_persone -= 3
            camere_triple -= 1
            triple_utilizzate += 1
            
        # Distribute people into double rooms
        while numero_persone >= 2 and camere_doppie > 0:
            numero_persone -= 2
            camere_doppie -= 1
            doppie_utilizzate += 1
            
        # Distribute people into single rooms
        while numero_persone >= 1 and camere_singole > 0:
            numero_persone -= 1
            camere_singole -= 1
            singole_utilizzate += 1
        
        return singole_utilizzate, doppie_utilizzate, triple_utilizzate, numero_persone

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
