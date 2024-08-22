""" Contiene il codice per classificare gli hotel e calcolare semplici metriche """
import os
import sys
import numpy as np
import pandas as pd
import json
from datetime import date, timedelta

# Aggiungi la directory parent alla variabile di percorso
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))

#from SyntheticDatas.generateDatas import generateDataset
from utils.jsonEncoding import decompressJson


# Funzione per la divisione in categorie degli hotel
def classify_hotels(json_file):
    # converte in Python
    data = json.loads(json_file)

    # controlla se il la lista è vuota
    if not data["Hotel List"]:
       print("Hotel List is empty") 

    L_hotels = {"Hotel List": []}
    S_hotels = {"Hotel List": []}
    B_hotels = {"Hotel List": []}

    for info in data["Hotel List"]:
       if info["category"] == "L":
          L_hotels["Hotel List"].append(info)
       
       elif info["category"] == "S":
          S_hotels["Hotel List"].append(info)

       elif info["category"] == "B":
          B_hotels["Hotel List"].append(info)
    
    L_json = json.dumps(L_hotels)
    S_json = json.dumps(S_hotels)
    B_json = json.dumps(B_hotels)
    return L_json, S_json, B_json
   
# FUNZIONI PER IL CALCOLO DI METRICHE

# Calcola e restituisce la media di una chiave selezionata per ciascun hotel in un file JSON
def add_mean(json_file, key):
    # Converte il JSON in un dizionario Python
    data = json.loads(json_file)

    # Controlla se la lista degli hotel è vuota
    if not data.get("Hotel List"):
        print("Hotel List is empty")
        return json_file

    # Controlla se la chiave esiste almeno in un hotel
    if not any(key in hotel for hotel in data["Hotel List"]):
        print(f"key {key} does not exist")
        return json_file

    # Dizionario per accumulare i valori della chiave per ciascun hotel
    hotel_values = {}

    # Raccoglie i valori della chiave specificata per ciascun hotel
    for hotel in data["Hotel List"]:
        hotel_name = hotel["HotelName"]
        if key in hotel:
            if hotel_name not in hotel_values:
                hotel_values[hotel_name] = []
            hotel_values[hotel_name].append(hotel[key])

    # Calcola la media per ciascun hotel
    hotel_means = {hotel: round(np.mean(values),2) if values else 0 for hotel, values in hotel_values.items()}

    # Crea la struttura del nuovo JSON con la chiave "Mean" all'interno di "Hotel List"
    result_data = {
        "Hotel List": {
            "Mean": {
              f"{key}" :hotel_means
            }
        }
    }

    # Restituisce il risultato come JSON
    return json.dumps(result_data)
    
# Calcola e aggiunge la media di una chiave selezionata in un file json nelll'ultimo trimestre
#da ottimizzare ancora
def add_mean_trimester(json_file, key):
    # converte in Python
    data = json.loads(json_file)

    # controlla se il la lista è vuota
    if not data["Hotel List"]:
       print("Hotel List is empty")
       return json_file 
    
    # data di oggi
    today = date.today()

    # controlla se la chiave esiste
    if key in data["Hotel List"][0].keys(): # il tipo di dato deve contenere la chiave "Hotel List", perde generalità

      all_key_data = [] # raccoglie tutti i dati chiave in una lista

      for key_info in data["Hotel List"]:
          mm_yyyy = key_info["date"].split("-") # si può cambiare per gestire date DD-MM-YYYY
          month = float(mm_yyyy[0])
       
          if month > today.month-3: # sceglie gli ultimi tre mesi consecutivi 
             key_data = key_info[key]
             all_key_data.append(key_data)

      if not all_key_data:
          all_key_data = 0  # nel caso in cui non ci sono valori

      key_mean = np.mean(all_key_data)
      data.update({key + "_mean_last_trimester" : key_mean})

      # conversione in JSON
      return json.dumps(data)
    
    else:
       print("key does not exist")
       return json_file
    
# Calcola e aggiunge la deviazione standard di una chiave selezionata in un file JSON
def add_std(json_file, key):
    # Converte il JSON in un dizionario Python
    data = json.loads(json_file)

    # Controlla se la lista degli hotel è vuota
    if not data.get("Hotel List"):
        print("Hotel List is empty")
        return json_file

    # Controlla se la chiave esiste almeno in un hotel
    if not any(key in hotel for hotel in data["Hotel List"]):
        print(f"key {key} does not exist")
        return json_file

    # Dizionario per accumulare i valori della chiave per ciascun hotel
    hotel_values = {}

    # Raccoglie i valori della chiave specificata per ciascun hotel
    for hotel in data["Hotel List"]:
        hotel_name = hotel["HotelName"]
        if key in hotel:
            if hotel_name not in hotel_values:
                hotel_values[hotel_name] = []
            hotel_values[hotel_name].append(hotel[key])

    # Calcola la deviazione standard per ciascun hotel
    hotel_std_devs = {hotel: round(np.std(values),2) if values else 0 for hotel, values in hotel_values.items()}

    # Crea la struttura del nuovo JSON con la chiave "DevStd" all'interno di "Hotel List"
    result_data = {
        "Hotel List": {
            "DevStd": {
               f"{key}":hotel_std_devs
            }
        }
    }

    # Restituisce il risultato come JSON
    return json.dumps(result_data)
      
def add_sum(json_file, key):
    # Converte il JSON in un dizionario Python
    data = json.loads(json_file)

    # Controlla se la lista degli hotel è vuota
    if not data.get("Hotel List"):
        print("Hotel List is empty")
        return json_file

    # Controlla se la chiave esiste almeno in un hotel
    if not any(key in hotel for hotel in data["Hotel List"]):
        print(f"key {key} does not exist")
        return json_file

    # Dizionario per accumulare le somme per ogni hotel
    hotel_sums = {}

    # Calcola la somma per ogni hotel
    for hotel in data["Hotel List"]:
        hotel_name = hotel["HotelName"]
        if key in hotel:
            if hotel_name not in hotel_sums:
                hotel_sums[hotel_name] = 0
            hotel_sums[hotel_name] += hotel[key]
            
    # Crea la struttura del nuovo JSON con le somme    
    result_data = {
        "Hotel List": {
           "Sum":{
              f"{key}":hotel_sums
         }
      }
   }

    # Restituisce il risultato come JSON
    return json.dumps(result_data)
 
if __name__ == "__main__":
   # oggetto python (dict):
   test_hotel_data = {
      "Hotel List": [
         {
            "hotel": "Hotel Freddo",
            "numero stanze": 300,
            "date": "05-2024",
            "category": "B",
            "shift": 6,
            "arrivals(n.ordini)": 5,
            "revenue_single(€)": 2844.07,
            "revenue_double(€)": 2496.55,
            "revenue_triple(€)": 11288.67,
            "total_revenue(€)": 17198.95,
            "room_price_single(€)": 442.29,
            "room_price_double(€)": 499.31,
            "room_price_triple(€)": 375.92,
            "distance(Km)": 5.8,
            "distance_factor": 0.03,
            "num_families": 10,
            "num_couples": 0,
            "num_single_guests": 0,
            "non evasi": 0,
            "varfact": 1.1468058806363377
         },
         {
            "hotel": "Hotel Caldo",
            "numero stanze": 300,
            "date": "07-2024",
            "category": "L",
            "shift": 5,
            "arrivals(n.ordini)": 12,
            "revenue_single(€)": 21.09,
            "revenue_double(€)": 69.62,
            "revenue_triple(€)": 10959.84,
            "total_revenue(€)": 6387.91,
            "room_price_single(€)": 384.38,
            "room_price_double(€)": 499.31,
            "room_price_triple(€)": 567.5,
            "distance(Km)": 5.8,
            "distance_factor": 0.03,
            "num_families": 2,
            "num_couples": 0,
            "num_single_guests": 0,
            "non evasi": 0,
            "varfact": 1.1290770956638854
         },
         {
            "hotel": "Hotel Caldo",
            "numero stanze": 300,
            "date": "08-2024",
            "category": "L",
            "shift": 3,
            "arrivals(n.ordini)": 25,
            "revenue_single(€)": 70.09,
            "revenue_double(€)": 117.15,
            "revenue_triple(€)": 17620.81,
            "total_revenue(€)": 7982.62,
            "room_price_single(€)": 370.12,
            "room_price_double(€)": 409.57,
            "room_price_triple(€)": 748.36,
            "distance(Km)": 5.8,
            "distance_factor": 0.03,
            "num_families": 7,
            "num_couples": 0,
            "num_single_guests": 1,
            "non evasi": 0,
            "varfact": 1.7516182950748733
         }
      ]
   }

   # conversione in JSON:
   test_hotel_json = json.dumps(test_hotel_data)
   print("Debug test")
   print("Test Json")   
   print(test_hotel_json)
   print("\n")   
   
   L_hotel_json, S_hotel_json, B_hotel_json = classify_hotels(test_hotel_json)
   print("Hotel Category: L",L_hotel_json)
   print("\n")
   print("Hotel Category: S",S_hotel_json)
   print("\n")
   print("Hotel Category: B",B_hotel_json)
   print("\n")
   
   L_hotel_json = add_mean(L_hotel_json, "revenue_single(€)")
   L_hotel_json = add_std(L_hotel_json, "revenue_single(€)")
   #L_hotel_json = add_mean_trimester(L_hotel_json, "num_families") #ancora da ottimizzare
   L_hotel_json = add_mean(L_hotel_json, "num_families")
   L_hotel_json = add_sum(L_hotel_json, "num_families")
   
   #N.B: Serve un aggregatore di parametri
   
   print("Debug for L Category:")
   print(L_hotel_json)
   

    

