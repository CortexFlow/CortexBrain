import pandas as pd
import random
import sys
import os
import warnings
import numpy as np
import json

warnings.filterwarnings("ignore")

# Aggiungi la directory parent alla variabile di percorso
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))

from utils.jsonEncoder import DataEncoder
from utils.DataProcessor import DataProcessor

def is_valid_data(df):
    """Check if the 'arrivals(n.ordini)' column contains zero for any month."""
    return not any(df['arrivals_ordini'] == 0)

def GenerateHotelDatas(params, key,debugMode=True):
    config = params["Global"]
    num_hotels = config["num_hotels"]
    years = config["years"]

    location_config = params["Location"][key]
    seasonality = location_config["seasonality"]
    hotel_config = location_config["hotel"]

    p_singola = hotel_config["p_singola"]
    p_doppia = hotel_config["p_doppia"]
    
    # Genera i nomi degli hotel in base al numero totale
    hotel_names = [f"Hotel {i+1}" for i in range(num_hotels)]
    
    months = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"]

    days_in_month = {
        "January": 31, "February": 28, "March": 31, "April": 30,
        "May": 31, "June": 30, "July": 31, "August": 31,
        "September": 30, "October": 31, "November": 30, "December": 31
    }
    
    while True:
        print("Parametri Generazione:")
        print("Location:", location_config)
        print("Building Config", hotel_config)
        print(".........")
        print("Generazione Dato...")
        data = []
        metrics = []

        configurations = {}
        for hotel in hotel_names:
            choices = ["L", "B", "S"]
            configurations[hotel] = random.choice(choices)

        distance_factors = {}
        
        for year in range(2025 - years, 2025):
            alpha = np.array([5.0, 0.5, 0.5])
            dirichlet_sample = np.random.dirichlet(alpha)
            a, b = 1, 2
            variability_factor = a + (b - a) * dirichlet_sample[random.randint(0, len(dirichlet_sample) - 1)]
            
            for hotel in hotel_names:
                configuration = configurations[hotel]
        
                if configuration == "L":
                    min_room_price = config["LuxuryMinPrice"]
                    max_room_price = config["LuxuryMaxPrice"]
                    rooms_per_hotel = config["LuxuryRooms"]
                elif configuration == "B":
                    min_room_price = config["BudgetMinPrice"]
                    max_room_price = config["BudgetMaxPrice"]
                    rooms_per_hotel = config["BudgetRooms"]
                else:  # standard
                    min_room_price = config["StandardMinPrice"]
                    max_room_price = config["StandardMaxPrice"]
                    rooms_per_hotel = config["StandardRooms"]
                
                if hotel not in distance_factors:
                    distance = random.uniform(0.5, 10)
                    distance_factor = DataProcessor.calculate_distance_factor(distance)
                    distance_factors[hotel] = (distance, distance_factor)
                    
                distance, distance_factor = distance_factors[hotel]
                
                double_price = np.random.uniform(min_room_price, max_room_price)
                single_price = double_price * 1.30
                triple_price = double_price * 1.50

                for month in months:
                    seasonal_factor = seasonality[month]
                    days = days_in_month[month]

                    num_single_rooms = int(rooms_per_hotel * p_singola)
                    num_double_rooms = int(rooms_per_hotel * p_doppia)
                    num_triple_rooms = rooms_per_hotel - (num_single_rooms + num_double_rooms)
                    
                    min_arrivals = 1 * rooms_per_hotel
                    max_arrivals = int((num_single_rooms * 1) + (num_double_rooms * 2) + (num_triple_rooms * 3))

                    mean_days = hotel_config["mean_stay"]

                    shifts_per_month = days // mean_days
                    if shifts_per_month == 0:
                        shifts_per_month = 1
                    shift_duration = mean_days
                    
                    for shift in range(shifts_per_month):
                        shift_start_day = shift * shift_duration + 1
                        shift_end_day = min(shift_start_day + shift_duration - 1, days)

                        base_arrivals = random.randint(min_arrivals, max_arrivals)
                        arrivals = int(base_arrivals * seasonal_factor * variability_factor * distance_factor)

                        singole_utilizzate, doppie_utilizzate, triple_utilizzate, ordini_non_evasi = DataProcessor.distribuisci_persone(arrivals, num_single_rooms, num_double_rooms, num_triple_rooms)

                        num_single_guests = singole_utilizzate
                        num_couples = doppie_utilizzate
                        num_families = triple_utilizzate

                        revenue_single = num_single_guests * single_price * mean_days
                        revenue_double = num_couples * double_price * mean_days
                        revenue_triple = num_families * triple_price * mean_days

                        total_revenue = revenue_single + revenue_double + revenue_triple

                        #date_str = f"{months.index(month) + 1:02d}-{year} (Days {shift_start_day} to {shift_end_day})"
                        #day=1
                        date_str= f"{year}-{months.index(month)+1:02d}-{shift_start_day:02d}"
                        
                        data.append({
                            "HotelName": hotel,
                            "numero_stanze": rooms_per_hotel,
                            "date": date_str,
                            "category": configuration,
                            "shift": shift + 1,
                            "arrivals_ordini": arrivals,
                            "revenue_single": round(revenue_single, 2),
                            "revenue_double": round(revenue_double, 2),
                            "revenue_triple": round(revenue_triple, 2),
                            "total_revenue": round(total_revenue, 2),
                            "room_price_single": round(single_price, 2),
                            "room_price_double": round(double_price, 2),
                            "room_price_triple": round(triple_price, 2),
                            "distance(Km)": round(distance, 2),
                            "distance_factor": round(distance_factor, 2),
                            "num_families": num_families,
                            "num_couples": num_couples,
                            "num_single_guests": num_single_guests,
                            "non evasi": ordini_non_evasi,
                            "varfact": variability_factor
                        })

                metrics.append({
                    "HotelName": hotel,
                    "Tipo di generazione": key,
                    "numero_stanze": rooms_per_hotel,
                    "Soggiorno Medio": mean_days,
                    "category": configuration,
                    "distance(Km)": round(distance, 2),
                    "distance_factor": round(distance_factor, 2),
                })

        df = pd.DataFrame(data)
        metricsData = pd.DataFrame(metrics)
        
        if is_valid_data(df):
            print("Valid dataset generated.")
            
            # Esporta i dati in formato JSON compresso
            output_data = {
                "Hotel List": data
            }
            DataEncoder.compressJson(output_data, 'hotel_data.json.gz')
            
            if debugMode==True:
                # Salva i dati reali in un foglio chisamato 'RealData'
                with pd.ExcelWriter('generazione.xlsx', engine='openpyxl') as writer:
                    df.to_excel(writer, sheet_name='RealData', index=False)
                    
                    # Salva le metriche in un foglio chiamato 'Metrics'
                    metricsData.to_excel(writer, sheet_name='Metrics', index=False)
                    return df, metricsData,output_data
            else:
                return df, metricsData,output_data
        else:
            print("Dataset non valido")
            print("Rigenerazione in corso....")

    return None, None
    
    
if __name__ == "__main__":
    
    # Carica i parametri dal file JSON
    with open('params.json', 'r') as file:
        params = json.load(file)
    
    # Genera i dataset e le metriche
    real_data, metrics,_ = GenerateHotelDatas(params=params,
                                                        key="Mare") 
                                                                
                                                                
    # Salva i dati reali in un foglio chisamato 'RealData'
    with pd.ExcelWriter('generazione.xlsx', engine='openpyxl') as writer:
        real_data.to_excel(writer, sheet_name='RealData', index=False)
        
        # Salva le metriche in un foglio chiamato 'Metrics'
        metrics.to_excel(writer, sheet_name='Metrics', index=False)
        
        
    
