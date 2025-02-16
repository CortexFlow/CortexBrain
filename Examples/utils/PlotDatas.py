""" THIS FUNCTION IS OUTDATED AND WILL BE UPDATED IN THE NEXT VERSION """



import sys
import os
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))
from folium.plugins import MarkerCluster
import folium
import matplotlib.patches as mpatches
import matplotlib.colors as mcolors
import numpy as np
from SyntheticDatas.generateDatas import GenerateHotelDatas
import matplotlib.pyplot as plt
import networkx as nx
import json
import pandas as pd



def CreateNetworkDataGraph(hotels, poi="CentralPlace", plotGraph=True):
    """
    Creates a graph showing hotels connected to a point of interest and returns the created graph.

    Parameters:
    hotels (list): List of dictionaries containing hotel data. Each dictionary should have the keys 'HotelName', 'distance(Km)', and 'numero_stanze'.
    poi (str): Name of the point of interest. Default is "CentralPlace".
    plotGraph (bool): If True, displays the graph. Default is True.

    Returns:
    G (networkx.Graph): The graph created with nodes and connections.
    """

    # Create an empty graph
    G = nx.Graph()

    # Add the point of interest node
    G.add_node(poi, category='generic', pos=(0, 0))

    # Add hotel nodes and connect them to the point of interest
    for hotel in hotels:
        hotel_name = hotel["HotelName"]
        pos = (np.random.uniform(-10, 10), np.random.uniform(-10, 10))
        G.add_node(hotel_name, category='S',
                   capacity=hotel.get('numero_stanze', 0), pos=pos)

        # Add an edge from the point of interest to the hotel
        G.add_edge(poi, hotel_name, distance=hotel.get('distance(Km)', 1))

    # Check the nodes of the graph (for debugging)
    print("Nodes in the graph:", G.nodes)

    # Position the nodes in the graph
    pos = nx.spring_layout(G)

    # Draw nodes and edges
    nx.draw(G, pos, with_labels=True, node_size=1000,
            node_color="lightblue", font_size=10, font_weight='bold')

    # Add edge labels for distances
    edge_labels = nx.get_edge_attributes(G, 'distance')
    nx.draw_networkx_edge_labels(G, pos, edge_labels=edge_labels)

    # Show the graph
    if plotGraph:
        plt.title("Hotels Connected to the Point of Interest")
        plt.show()

    # Return the created graph
    return G


def CreateNetworkData(df, poi="CentralPlace", pos=(0, 0), poi_types=None, plotGraph=True, savePath="../Gui/output"):
    """
    Creates a graph showing points of interest of certain types connected to a central point and returns the created graph.
    Includes a geolocated map.

    Parameters:
    df (pd.DataFrame or dict): DataFrame containing POI data with columns 'Nome', 'Tipo', 'Coordinate', 'Distance_from_POI', and 'Image'.
                               If a dict is passed, it will be converted to a DataFrame.
    poi (str or list): Name of the central point of interest. Can be a string or a list of strings.
                       If it's a list, it will be concatenated into a single string. Default is "CentralPlace".
    pos (tuple): Coordinates of the central point of interest. Default is (0, 0).
    poi_types (list of str): List of POI types to include in the graph. If None, includes all types. Default is None.
    plotGraph (bool): If True, displays the graph. Default is True.

    Returns:
    G (networkx.Graph): The graph created with nodes and connections.
    """

    # Convert dictionary to DataFrame if needed
    if isinstance(df, dict):
        # Check if all lists in the dictionary have the same length
        lengths = [len(v) for v in df.values()]
        if len(set(lengths)) > 1:
            raise ValueError("All lists in the dictionary must have the same length")
        df = pd.DataFrame(df)

    # If 'poi' is a list, convert it to a concatenated string
    if isinstance(poi, list):
        poi = "_".join(poi)  # Concatenate list into a string with "_"

    # Filter the DataFrame to include only POIs of specified types
    if poi_types:
        df = df[df["Tipo"].isin(poi_types)]

    # Create an empty graph
    G = nx.Graph()

    # Add the central point of interest node
    G.add_node(poi, pos=pos, category='central_place')

    # Add POI nodes and connect them to the central point of interest
    distances = []  # List to store distances
    for index, row in df.iterrows():
        place_name = row["Nome"]
        place_type = row["Tipo"]
        distance = row["Distance_from_POI"]
        coordinates = row['Coordinate']
        
        # Check the type of coordinates and convert if necessary
        if isinstance(coordinates, str):
            # If coordinates are a string, convert to float
            coordinates = tuple(map(float, coordinates.strip('()').split(', ')))
        elif isinstance(coordinates, (tuple, list)) and len(coordinates) == 2:
            # If coordinates are a tuple or list of two elements, convert to tuple
            coordinates = tuple(coordinates)
        else:
            # Handle error for unexpected formats
            print(f"Unexpected format for coordinates in row {index}: {coordinates}")
            distances.append(None)
            continue
        
        # Retrieve the image URL if available
        image_url = row.get("Image", "")

        # Add POI node with its attributes
        G.add_node(place_name, pos=coordinates, category=place_type)

        # Add an edge from the central point of interest to the POI
        G.add_edge(poi, place_name, distance=round(distance, 3))
        distances.append(round(distance, 3))

    # Extract node positions
    pos = nx.get_node_attributes(G, 'pos')

    # Create a satellite map centered on the central point of interest
    mappa = folium.Map(location=pos[poi], zoom_start=13,
                       tiles='http://mt1.google.com/vt/lyrs=s&x={x}&y={y}&z={z}', attr='Google')

    # Add markers for each node on the map
    for node, coords in pos.items():
        if node == poi:
            # Custom marker for the central point with informational popup
            popup_content = f"<strong>{node}</strong><br>Type: Central Point"
            folium.Marker(
                location=coords,
                popup=popup_content,
                icon=folium.Icon(color='red', icon='info-sign')
            ).add_to(mappa)
        else:
            # Retrieve information from the graph
            node_type = G.nodes[node]['category']
            distance = G[poi][node]['distance']

            # Custom popup with additional information
            if image_url:
                popup_content = (
                    f"<strong>{node}</strong><br>"
                    f"Type: {node_type}<br>"
                    f"Distance from central point: {distance} km<br>"
                    f"Coordinates: {coords}<br>"
                    f"<img src='{image_url}' alt='Image of {node}' style='width:200px; height:auto;'>"
                )
            else:
                popup_content = (
                    f"<strong>{node}</strong><br>"
                    f"Type: {node_type}<br>"
                    f"Distance from central point: {distance} km<br>"
                    f"Coordinates: {coords}"
                )

            # Add marker to the map
            folium.Marker(
                location=coords,
                popup=popup_content,
                icon=folium.Icon(color='blue', icon='cloud')
            ).add_to(mappa)

    # Add lines connecting the central node to other POIs
    for edge in G.edges:
        folium.PolyLine([pos[edge[0]], pos[edge[1]]],
                        color='grey').add_to(mappa)

    # Show the map
    if plotGraph:
        mappa.save(f"{savePath}/mappa_punti_interesse_satellitare.html")
        print("Satellite map created and saved as 'mappa_punti_interesse_satellitare.html'")

    return G


if __name__ == "__main__":
    """ Creates a city based on random generation of structures around a point of interest """
    if os.path.exists('./output/generazione.xlsx'):
        print("Generation file found. Loading data from file.")
        metrics = pd.read_excel('./output/generazione.xlsx', sheet_name=1)
    else:
        with open('../SyntheticDatas/params.json', 'r') as file:
            params = json.load(file)

        datas, metrics, dict_data = GenerateHotelDatas(
            params=params, key="Mare", debugMode=True)
        metrics.to_excel('./output/generazione.xlsx', index=False)

    data = metrics.to_dict(orient='records')
    
    CreateNetworkDataGraph(data)  # Test function with graph
    dataDF=pd.read_csv("../Gui/output/DistanceFromPoi.csv")
    CreateNetworkData(dataDF, poi="Duomo", pos=(45.81178183320625, 9.083683148070666), savePath="../Gui/output")  # Test function with satellite visualization
