""" Obiettivo: classe turista, genera un agent con gli attributi da turista

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

from utils.jsonEncoder import compressJson, testEncoding
from utils.DataProcessor import DataProcessor
from utils.PlotDatas import CreateNetworkData
from Analyze.analyze import add_mean, add_std, add_sum, classify_hotels
from SyntheticDatas.generateDatas import GenerateHotelDatas

class Tourist(Agent):
    """ Modello turista. """
    def __init__(self, unique_id, model, preferred_hotel):
        super().__init__(unique_id, model)
        self.preferred_hotel = preferred_hotel
        self.days_in_hotel = 0
        self.stay_duration = np.random.poisson(5)  # Durata del soggiorno, media di 5 giorni
        self.arrival_date = model.current_date
        self.departure_date = None

    def step(self):
        """ Prova a prenotare una stanza nel suo hotel preferito e gestisce il ricambio. """
        if self.model.book_room(self.preferred_hotel):
            print(f"Tourist {self.unique_id} booked a room in {self.preferred_hotel}")
            self.days_in_hotel += 1
            if self.days_in_hotel >= self.stay_duration:
                self.leaveHotel()
        else:
            available_hotels = [hotel for hotel in self.model.hotels if hotel != self.preferred_hotel and self.model.hotels[hotel] > 0]
            if available_hotels:
                other_hotel = self.random.choice(available_hotels)
                if self.model.book_room(other_hotel):
                    print(f"Tourist {self.unique_id} booked a room in {other_hotel}")
                    self.preferred_hotel = other_hotel
                    self.days_in_hotel += 1
                    if self.days_in_hotel >= self.stay_duration:
                        self.leaveHotel()
                else:
                    print(f"Tourist {self.unique_id} couldn't book any room")
            else:
                print(f"Tourist {self.unique_id} couldn't book any room")

    def leaveHotel(self):
        """ Rimuove il turista dall'hotel e dal modello, poi crea un nuovo turista. """
        print(f"Tourist {self.unique_id} leaving hotel after {self.days_in_hotel} days.")
        self.model.schedule.remove(self)
        self.model.grid.remove_agent(self)
        
        # Aggiorna la data di partenza
        if self.departure_date is None:
            self.departure_date = self.model.current_date
        
        # Aggiungi i dati del turista alla lista
        self.model.tourist_data.append({
            'Tourist ID': self.unique_id,
            'Hotel': self.preferred_hotel,
            'Days Stayed': self.days_in_hotel,
            'Arrival Date': self.arrival_date.strftime('%Y-%m-%d'),
            'Departure Date': self.departure_date.strftime('%Y-%m-%d')
        })
        
        # Crea un nuovo turista
        new_tourist = Tourist(self.model.num_agents, self.model, self.model.random.choice(list(self.model.hotels.keys())))
        self.model.schedule.add(new_tourist)
        self.model.grid.place_agent(new_tourist, self.model.central_place)
        self.model.num_agents += 1
