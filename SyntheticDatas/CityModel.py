""" Obiettivo: simulazione turistica in una piccola città con 30 Hotel generati a partire dal file params.json 

    N.B: Da aggiungere classificazione dei turisti in singoli,coppie e famiglie e successiva suddivisione nelle stanze degli hotel
    Implementare logica per la quale vengono preferite strutture più vicine al punto di interesse rispetto che ad altre

"""
import numpy as np
import pandas as pd
import json
import sys
import os
import networkx as nx
from datetime import datetime, timedelta
from mesa import Agent, Model
from mesa.time import RandomActivation
from mesa.space import NetworkGrid
import matplotlib.pyplot as plt

# Aggiungi la directory parent alla variabile di percorso
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))

from utils.jsonEncoding import compressJson, testEncoding
from utils.DataProcessor import DataProcessor
from utils.PlotDatas import CreateNetworkData
from utils.jsonEncoding import decompressJson
from Analyze.analyze import add_mean, add_std, add_sum, classify_hotels
from SyntheticDatas.generateDatas import GenerateHotelDatas

#classe turista
from TouristModel import Tourist

class CityModel(Model):
    """ Simula la città con turisti e hotel. """
    def __init__(self, num_tourists_per_day, city_graph):
        self.num_agents = 0
        self.num_tourists_per_day = num_tourists_per_day  # Numero di turisti da aggiungere ogni giorno
        self.schedule = RandomActivation(self)
        self.grid = NetworkGrid(city_graph)
        self.G = city_graph
        
        # Crea un dizionario degli hotel e verifica che ci siano
        self.hotels = {node: data['capacity'] for node, data in city_graph.nodes(data=True) if data.get('category') == 'S'}
        self.central_place = 'CentralPlace'
        
        if not self.hotels:
            raise ValueError("No hotels found in the city graph.")
        
        # Inizializza il DataFrame per tenere traccia degli arrivi
        self.tourist_data = []
        self.start_date = datetime(2024, 1, 1)
        self.current_date = self.start_date
        self.end_date = datetime(2024, 12, 31)
        self.current_day = 0

    def AddTourists(self):
        """ Aggiunge turisti al modello ogni giorno. """
        for _ in range(self.num_tourists_per_day):
            if self.hotels:
                preferred_hotel = self.random.choice(list(self.hotels.keys()))
            else:
                preferred_hotel = None
            
            tourist = Tourist(self.num_agents, self, preferred_hotel)
            self.schedule.add(tourist)
            self.grid.place_agent(tourist, self.central_place)
            self.num_agents += 1

    def book_room(self, hotel_name):
        """ Prenota una stanza in un hotel se disponibile. """
        if hotel_name in self.hotels and self.hotels[hotel_name] > 0:
            self.hotels[hotel_name] -= 1
            return True
        return False

    def step(self):
        self.schedule.step()
        self.AddTourists()  # Aggiungi nuovi turisti ogni giorno
        self.current_date += timedelta(days=1)
        self.current_day += 1
        self.print_available_rooms()

    def print_available_rooms(self):
        """ Stampa il numero di posti disponibili in ogni hotel. """
        available_rooms = {hotel: capacity for hotel, capacity in self.hotels.items() if capacity > 0}
        print("Available rooms:", available_rooms)

    def SaveTouristData(self):
        """ Salva le statistiche dei turisti in un file CSV. """
        df = pd.DataFrame(self.tourist_data)
        df.to_csv('tourist_data.csv', index=False)
        print("Saved tourist data to 'tourist_data.csv'.")

def createCity(generatePlot=True):
    """ Crea una città a partire dalla generazione randomica delle strutture attorno ad un punto di interesse """
    if os.path.exists('generazione.xlsx'):
        print("File di generazione trovato. Caricamento dei dati dal file.")
        metrics = pd.read_excel('./generazione.xlsx', sheet_name=1) 
    else:
        with open('../SyntheticDatas/params.json', 'r') as file:
            params = json.load(file)
        
        datas, metrics, dict_data = GenerateHotelDatas(params=params, key="Mare", debugMode=True)
        metrics.to_excel('generazione.xlsx', index=False)
    
    data = metrics.to_dict(orient='records')
    

    # Se generatePlot è True, visualizza il grafo
    if generatePlot:
        G=CreateNetworkData(data)
    
    return G, data

def runSim(model, steps=365):
    """ Esegue il modello per un numero specificato di passi. """
    for step in range(steps):
        model.step()
        # Aggiungi stampe per il debug
        print(f"Step {step} completed")
        print(f"Number of tourists: {len(model.schedule.agents)}")
        print(f"Hotel capacities: {model.hotels}")
    
    model.SaveTouristData()  # Salva i dati dei turisti al termine della simulazione
    

def main():
    G, data = createCity()  # Crea il grafo e carica i dati
    
    # Definisci il modello con i parametri necessari
    model_params = {
        "num_tourists_per_day": 10,  # Numero di turisti da aggiungere ogni giorno
        "city_graph": G
    }
    
    # Crea il modello
    model = CityModel(**model_params)
    
    # Esegui il modello automaticamente per 365 giorni
    runSim(model, steps=25)

if __name__ == "__main__":
    main()