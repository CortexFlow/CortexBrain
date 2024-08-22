import os
import sys
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '../../')))
from utils.PlotDatas import CreateNetworkData
import osmnx as ox
import requests
import pandas as pd
from geopy.distance import geodesic
import json



class RealWorld:
    # Function to extract coordinates from a geometry
    def GetCoordinates(geometry):
        if geometry.geom_type == 'Point':
            return geometry.y, geometry.x
        elif geometry.geom_type in ['Polygon', 'MultiPolygon']:
            # Use the centroid for polygons
            return geometry.centroid.y, geometry.centroid.x
        return None, None

    
    # Returns information about a place
    def GetPlaceInfo(place_name="Como,CO"):
        graph = ox.graph_from_place(place_name, network_type='all')

        # Visualize the street network
        ox.plot_graph(graph)

        # Download points of interest (POI) in the neighborhood using the new 'features' module
        # You can customize the tags you are interested in
        tags = {'amenity': True}
        pois = ox.features_from_place(place_name, tags)

        # Filter POIs to keep only those with names
        pois_with_names = pois[pois['name'].notna()]

        df = pd.DataFrame(columns=["Nome", "Tipo", "Coordinate"])

        # List to collect all POI dictionaries
        pois_list = []

        # Print a list of POIs with their names and types
        for idx, row in pois_with_names.iterrows():
            coords = RealWorld.GetCoordinates(row['geometry'])
            if coords[0] is not None and coords[1] is not None:
                # Create a dictionary for the POI and add it to the list
                pois_list.append({
                    "Nome": row['name'],
                    "Tipo": row['amenity'],
                    "Coordinate": (coords[0], coords[1])
                })

                print(f"Name: {row['name']}, Type: {row['amenity']}, "
                    f"Coordinates: ({coords[0]}, {coords[1]})")


        # Convert the list of dictionaries into a DataFrame
        df = pd.concat([df, pd.DataFrame(pois_list)], ignore_index=True)
        return df

    def EvaluateDistance(df, centralPlaceCoords):
        # Extract the central place coordinates
        lat1, lon1 = centralPlaceCoords

        # List to store distances
        distances = []

        # Iterate over each row of the DataFrame to calculate distance
        for index, row in df.iterrows():
            coords = row['Coordinate']
            
            # Check the type of 'coords' to decide the approach to use
            if isinstance(coords, str):
                # If the coordinate is a string, convert it to float
                coords = coords.strip('()').split(', ')
                lat2, lon2 = float(coords[0]), float(coords[1])
            elif isinstance(coords, (tuple, list)) and len(coords) == 2:
                # If the coordinate is a tuple or list of length 2
                lat2, lon2 = coords
            else:
                # Handle error for unexpected formats
                print(f"Unexpected format for coordinates in row {index}: {coords}")
                distances.append(None)
                continue

            # Define the points
            point1 = (lat1, lon1)
            point2 = (lat2, lon2)

            # Calculate the distance between the central point and the POI
            distance = geodesic(point1, point2).km

            # Save the distance in the list
            distances.append(distance)

            # Print the distance
            print(f"POI {row['Nome']}: Distance from central point: {distance:.2f} km")

        # Add the distances to the original DataFrame
        if len(distances) == len(df):
            df['Distance_from_POI'] = distances
        else:
            print("The length of the distance list does not match the number of rows in the DataFrame")

        # Return the updated DataFrame with distances
        return df


    # Application function
    def app(df, start_coords, start_poi_name=None, poiTypes=None, savePath="../output"):
        # Calculate the distance from points of interest
        df = RealWorld.EvaluateDistance(df, start_coords)
        df.to_csv(f"{savePath}/DistanceFromPoi.csv", index=False)
        # Create the graph
        CreateNetworkData(df, poi=start_poi_name, pos=start_coords, poi_types=poiTypes)


if __name__ == "__main__":
    
    if not os.path.exists("config.json"):
        print("Configuration file not found")
    else:
        # Load parameters from the JSON file
        with open('config.json', 'r') as file:
            config = json.load(file)
            
        coordsDuomo = config["coordinates"]
        cityName = config["city"]
        startingPlace = config["startingPlace"]
        searchList = config["searchCategory"]
        savePathPoi = config["savePOI"]
        
        # coordsDuomo=(45.5684917, 9.2416796)
        if not os.path.exists(f"{savePathPoi}/POI.csv"):
            df = RealWorld.GetPlaceInfo(place_name=cityName)
            print(df.head())
            df.to_csv(f"{savePathPoi}/POI.csv", index=False)
            RealWorld.app(df, coordsDuomo, startingPlace, searchList, savePathPoi)
        else:
            print("Coordinate file found. Loading data from file.")
            df = pd.read_csv(f"{savePathPoi}/POI.csv")
            RealWorld.app(df, coordsDuomo, startingPlace, searchList)