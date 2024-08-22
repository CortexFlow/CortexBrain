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
    Crea un grafo che mostra gli hotel collegati al punto di interesse e restituisce il grafo creato.

    Parameters:
    hotels (list): Lista di dizionari contenenti i dati degli hotel. Ogni dizionario deve avere le chiavi 'HotelName', 'distance(Km)', e 'numero_stanze'.
    poi (str): Nome del punto di interesse. Default è "CentralPlace".
    plotGraph (bool): Se True, visualizza il grafo. Default è True.

    Returns:
    G (networkx.Graph): Il grafo creato con i nodi e i collegamenti.
    """

    # Creare un grafo vuoto
    G = nx.Graph()

    # Aggiungere il nodo del punto di interesse
    G.add_node(poi, category='generic', pos=(0, 0))

    # Aggiungere i nodi degli hotel e collegarli al punto di interesse
    for hotel in hotels:
        hotel_name = hotel["HotelName"]
        pos = (np.random.uniform(-10, 10), np.random.uniform(-10, 10))
        G.add_node(hotel_name, category='S',
                   capacity=hotel.get('numero_stanze', 0), pos=pos)

        # Aggiungere un arco dal punto di interesse all'hotel
        G.add_edge(poi, hotel_name, distance=hotel.get('distance(Km)', 1))

    # Verifica i nodi del grafo (per debugging)
    print("Nodi nel grafo:", G.nodes)

    # Posizionare i nodi nel grafico
    pos = nx.spring_layout(G)

    # Disegnare i nodi e i collegamenti
    nx.draw(G, pos, with_labels=True, node_size=1000,
            node_color="lightblue", font_size=10, font_weight='bold')

    # Aggiungere le etichette delle distanze sui collegamenti
    edge_labels = nx.get_edge_attributes(G, 'distance')
    nx.draw_networkx_edge_labels(G, pos, edge_labels=edge_labels)

    # Mostrare il grafo
    if plotGraph:
        plt.title("Hotel Collegati al Punto di Interesse")
        plt.show()

    # Restituire il grafo creato
    return G


def CreateNetworkData(df, poi="CentralPlace", pos=(0, 0), poi_types=None, plotGraph=True, savePath="./output"):
    """
    Crea un grafo che mostra i punti di interesse di certi tipi collegati al punto centrale e restituisce il grafo creato.
    Include la mappa geolocalizzata.

    Parameters:
    df (pd.DataFrame or dict): DataFrame contenente i dati dei POI con colonne 'Nome', 'Tipo', 'Coordinate', 'Distance_from_POI', e 'Image'.
                               Se viene passato un dict, viene convertito in un DataFrame.
    poi (str or list): Nome del punto di interesse centrale. Può essere una stringa o una lista di stringhe.
                       Se è una lista, verrà convertito in una stringa concatenata. Default è "CentralPlace".
    pos (tuple): Coordinate del punto di interesse centrale. Default è (0, 0).
    poi_types (list of str): Lista dei tipi di POI da includere nel grafo. Se None, include tutti i tipi. Default è None.
    plotGraph (bool): Se True, visualizza il grafo. Default è True.

    Returns:
    G (networkx.Graph): Il grafo creato con i nodi e i collegamenti.
    """

    # Converti il dizionario in un DataFrame se necessario
    if isinstance(df, dict):
        # Verifica se tutte le liste nel dizionario hanno la stessa lunghezza
        lengths = [len(v) for v in df.values()]
        if len(set(lengths)) > 1:
            raise ValueError("Tutte le liste nel dizionario devono avere la stessa lunghezza")
        df = pd.DataFrame(df)

    # Se 'poi' è una lista, convertirla in una stringa concatenando gli elementi
    if isinstance(poi, list):
        poi = "_".join(poi)  # Converti la lista in una stringa unendo gli elementi con un "_"

    # Filtrare il DataFrame per includere solo i POI dei tipi specificati
    if poi_types:
        df = df[df["Tipo"].isin(poi_types)]

    # Creare un grafo vuoto
    G = nx.Graph()

    # Aggiungere il nodo del punto di interesse centrale
    G.add_node(poi, pos=pos, category='central_place')

    # Aggiungere i nodi dei POI e collegarli al punto di interesse centrale
    distances = []  # Lista per memorizzare le distanze
    for index, row in df.iterrows():
        place_name = row["Nome"]
        place_type = row["Tipo"]
        distance = row["Distance_from_POI"]
        coordinates = row['Coordinate']
        
        # Verifica il tipo di coordinate e converte se necessario
        if isinstance(coordinates, str):
            # Se la coordinata è una stringa, la convertiamo in float
            coordinates = tuple(map(float, coordinates.strip('()').split(', ')))
        elif isinstance(coordinates, (tuple, list)) and len(coordinates) == 2:
            # Se la coordinata è una tupla o una lista di due elementi, la convertiamo in una tupla
            coordinates = tuple(coordinates)
        else:
            # Gestione dell'errore per formati inattesi
            print(f"Formato inatteso per le coordinate nella riga {index}: {coordinates}")
            distances.append(None)
            continue
        
        # Recupera l'URL dell'immagine se disponibile
        image_url = row.get("Image", "")

        # Aggiungere il nodo del POI con le sue caratteristiche
        G.add_node(place_name, pos=coordinates, category=place_type)

        # Aggiungere un arco dal punto di interesse centrale al POI
        G.add_edge(poi, place_name, distance=round(distance, 3))
        distances.append(round(distance, 3))

    # Estrai le posizioni dei nodi
    pos = nx.get_node_attributes(G, 'pos')

    # Crea una mappa satellitare centrata sul punto di interesse centrale
    mappa = folium.Map(location=pos[poi], zoom_start=13,
                       tiles='http://mt1.google.com/vt/lyrs=s&x={x}&y={y}&z={z}', attr='Google')

    # Aggiungi i marker per ogni nodo sulla mappa
    for node, coords in pos.items():
        if node == poi:
            # Segnalino personalizzato per il punto centrale con popup informativo
            popup_content = f"<strong>{node}</strong><br>Tipo: Punto Centrale"
            folium.Marker(
                location=coords,
                popup=popup_content,
                icon=folium.Icon(color='red', icon='info-sign')
            ).add_to(mappa)
        else:
            # Recupera le informazioni dal grafo
            node_type = G.nodes[node]['category']
            distance = G[poi][node]['distance']

            # Popup personalizzato con più informazioni
            if image_url:
                popup_content = (
                    f"<strong>{node}</strong><br>"
                    f"Tipo: {node_type}<br>"
                    f"Distanza dal punto centrale: {distance} km<br>"
                    f"Coordinate: {coords}<br>"
                    f"<img src='{image_url}' alt='Immagine di {node}' style='width:200px; height:auto;'>"
                )
            else:
                popup_content = (
                    f"<strong>{node}</strong><br>"
                    f"Tipo: {node_type}<br>"
                    f"Distanza dal punto centrale: {distance} km<br>"
                    f"Coordinate: {coords}"
                )

            # Aggiungere il segnalino alla mappa
            folium.Marker(
                location=coords,
                popup=popup_content,
                icon=folium.Icon(color='blue', icon='cloud')
            ).add_to(mappa)

    # Aggiungi le linee che collegano il nodo centrale agli altri POI
    for edge in G.edges:
        folium.PolyLine([pos[edge[0]], pos[edge[1]]],
                        color='grey').add_to(mappa)

    # Mostra la mappa
    if plotGraph:
        mappa.save(f"{savePath}/mappa_punti_interesse_satellitare.html")
        print("Mappa satellitare creata e salvata come 'mappa_punti_interesse_satellitare.html'")

    return G

if __name__ == "__main__":
    """ Crea una città a partire dalla generazione randomica delle strutture attorno ad un punto di interesse """
    if os.path.exists('./output/generazione.xlsx'):
        print("File di generazione trovato. Caricamento dei dati dal file.")
        metrics = pd.read_excel('./output/generazione.xlsx', sheet_name=1)
    else:
        with open('../SyntheticDatas/params.json', 'r') as file:
            params = json.load(file)

        datas, metrics, dict_data = GenerateHotelDatas(
            params=params, key="Mare", debugMode=True)
        metrics.to_excel('./output/generazione.xlsx', index=False)

    data = metrics.to_dict(orient='records')
    
    CreateNetworkDataGraph(data)  # test della funzione con grafo
    dataDF=pd.read_csv("../RealWorld/output/DistanceFromPoi.csv")
    CreateNetworkData(dataDF,poi="Duomo",pos=(45.81178183320625, 9.083683148070666),savePath="../RealWorld/output")  # test della funzione con visualizzazione satellitare
