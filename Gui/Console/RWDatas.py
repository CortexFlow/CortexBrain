import os
import sys
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))
from utils.PlotDatas import CreateNetworkData
import osmnx as ox
import requests
import pandas as pd
from geopy.distance import geodesic
import json



class RealWorld:
    # Funzione per estrarre le coordinate a partire da una geometria
    def GetCoordinates(geometry):
        if geometry.geom_type == 'Point':
            return geometry.y, geometry.x
        elif geometry.geom_type in ['Polygon', 'MultiPolygon']:
            # Usa il centroide per i poligoni
            return geometry.centroid.y, geometry.centroid.x
        return None, None

    
    #ritorna le informazioni di un luogo
    def GetPlaceInfo(place_name="Como,CO"):
        graph = ox.graph_from_place(place_name, network_type='all')

        # Visualizza la rete stradale
        ox.plot_graph(graph)

        # Scarica i punti di interesse (POI) nel quartiere utilizzando il nuovo modulo 'features'
        # Puoi personalizzare i tag che ti interessano
        tags = {'amenity': True}
        pois = ox.features_from_place(place_name, tags)

        # Filtra i POI per mantenere solo quelli con nome
        pois_with_names = pois[pois['name'].notna()]

        df = pd.DataFrame(columns=["Nome", "Tipo", "Coordinate"])

        # Lista per raccogliere tutti i dizionari dei POI
        pois_list = []

        # Stampa una lista dei POI con i loro nomi e tipi
        for idx, row in pois_with_names.iterrows():
            coords = RealWorld.GetCoordinates(row['geometry'])
            if coords[0] is not None and coords[1] is not None:
                # Crea un dizionario per il POI e aggiungilo alla lista
                pois_list.append({
                    "Nome": row['name'],
                    "Tipo": row['amenity'],
                    "Coordinate": (coords[0], coords[1])
                })

                print(f"Nome: {row['name']}, Tipo: {
                      row['amenity']}, Coordinate: ({coords[0]}, {coords[1]})")

        # Converti la lista di dizionari in un DataFrame
        df = pd.concat([df, pd.DataFrame(pois_list)], ignore_index=True)
        return df

    def EvaluateDistance(df, centralPlaceCoords):
        # Estrae le coordinate del punto centrale
        lat1, lon1 = centralPlaceCoords

        # Lista per memorizzare le distanze
        distances = []

        # Itera su ogni riga del DataFrame per calcolare la distanza
        for index, row in df.iterrows():
            coords = row['Coordinate']
            
            # Verifica il tipo di dati di 'coords' per decidere quale approccio usare
            if isinstance(coords, str):
                # Se la coordinata è una stringa, la convertiamo in float
                coords = coords.strip('()').split(', ')
                lat2, lon2 = float(coords[0]), float(coords[1])
            elif isinstance(coords, (tuple, list)) and len(coords) == 2:
                # Se la coordinata è una tupla o una lista di lunghezza 2
                lat2, lon2 = coords
            else:
                # Gestione dell'errore per formati inattesi
                print(f"Formato inatteso per le coordinate nella riga {index}: {coords}")
                distances.append(None)
                continue

            # Definisce i punti
            point1 = (lat1, lon1)
            point2 = (lat2, lon2)

            # Calcola la distanza tra il punto centrale e il POI
            distance = geodesic(point1, point2).km

            # Salva la distanza nella lista
            distances.append(distance)

            # Stampa la distanza
            print(f"POI {row['Nome']}: Distanza dal punto centrale: {distance:.2f} km")

        # Aggiunge le distanze al DataFrame originale
        if len(distances) == len(df):
            df['Distance_from_POI'] = distances
        else:
            print("La lunghezza della lista delle distanze non corrisponde al numero di righe del DataFrame")

        # Ritorna il DataFrame aggiornato con le distanze
        return df


    #applicazione
    def app(df, start_coords, start_poi_name=None, poiTypes=None,savePath="./output"):
        # calcola la distanza dai punti di interesse
        df = RealWorld.EvaluateDistance(
            df, start_coords)
        df.to_csv(f"{savePath}/DistanceFromPoi.csv", index=False)
        # creazione del grafo
        CreateNetworkData(df, poi=start_poi_name,
                          pos=start_coords, poi_types=poiTypes)


if __name__ == "__main__":
    
    if not os.path.exists("config.json"):
        print("File di configurazione non trovato")
    else:
        # Carica i parametri dal file JSON
        with open('config.json', 'r') as file:
            config = json.load(file)
            
        coordsDuomo = config["coordinates"]
        cityName=config["city"]
        startingPlace=config["startingPlace"]
        searchList=config["searchCategory"]
        savePathPoi=config["savePOI"]
        
        # coordsDuomo=(45.5684917, 9.2416796)
        if not os.path.exists(f"{savePathPoi}/POI.csv"):
            df = RealWorld.GetPlaceInfo(place_name=cityName)
            print(df.head())
            df.to_csv(f"{savePathPoi}/POI.csv", index=False)
            RealWorld.app(df, coordsDuomo,startingPlace, searchList,savePathPoi)
        else:
            print("File delle coordinate trovato. Caricamento dei dati dal file.")
            df = pd.read_csv(f"{savePathPoi}/POI.csv")
            RealWorld.app(df, coordsDuomo,startingPlace,searchList)
