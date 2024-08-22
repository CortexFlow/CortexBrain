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

def GenerateHotelDatas(params, key, debugMode=True):
    config = params["Global"]
    num_hotels = config["num_hotels"]
    years = config["years"]

    location_config = params["Location"][key]
    seasonality = location_config["seasonality"]
    hotel_config = location_config["hotel"]

    p_singola = hotel_config["p_singola"]
    p_doppia = hotel_config["p_doppia"]

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
        hotel_capacities = {}
        for hotel in hotel_names:
            choices = ["L", "B", "S"]
            configuration = random.choice(choices)
            configurations[hotel] = configuration

            if configuration == "L":
                hotel_capacities[hotel] = {
                    "single": int(config["LuxuryRooms"] * p_singola),
                    "double": int(config["LuxuryRooms"] * p_doppia),
                    "triple": config["LuxuryRooms"] - int(config["LuxuryRooms"] * p_singola) - int(config["LuxuryRooms"] * p_doppia)
                }
            elif configuration == "B":
                hotel_capacities[hotel] = {
                    "single": int(config["BudgetRooms"] * p_singola),
                    "double": int(config["BudgetRooms"] * p_doppia),
                    "triple": config["BudgetRooms"] - int(config["BudgetRooms"] * p_singola) - int(config["BudgetRooms"] * p_doppia)
                }
            else:  # Standard
                hotel_capacities[hotel] = {
                    "single": int(config["StandardRooms"] * p_singola),
                    "double": int(config["StandardRooms"] * p_doppia),
                    "triple": config["StandardRooms"] - int(config["StandardRooms"] * p_singola) - int(config["StandardRooms"] * p_doppia)
                }

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
                else:  # Standard
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

                    num_single_rooms = hotel_capacities[hotel]["single"]
                    num_double_rooms = hotel_capacities[hotel]["double"]
                    num_triple_rooms = hotel_capacities[hotel]["triple"]
                    
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

                        # Track remaining capacity in the current hotel
                        remaining_single_rooms = num_single_rooms - singole_utilizzate
                        remaining_double_rooms = num_double_rooms - doppie_utilizzate
                        remaining_triple_rooms = num_triple_rooms - triple_utilizzate

                        redistributed = False
                        cancelled = False
                        if ordini_non_evasi > 0:
                            redistributed, cancelled = redistribute_bookings(
                                ordini_non_evasi,
                                remaining_single_rooms,
                                remaining_double_rooms,
                                remaining_triple_rooms,
                                hotel_capacities
                            )

                        num_single_guests = singole_utilizzate
                        num_couples = doppie_utilizzate
                        num_families = triple_utilizzate

                        revenue_single = num_single_guests * single_price * mean_days
                        revenue_double = num_couples * double_price * mean_days
                        revenue_triple = num_families * triple_price * mean_days

                        total_revenue = revenue_single + revenue_double + revenue_triple

                        # Calculate the actual considered orders
                        considered_orders = arrivals - ordini_non_evasi

                        date_str = f"{year}-{months.index(month)+1:02d}-{shift_start_day:02d}"
                        
                        data.append({
                            "HotelName": hotel,
                            "numero_stanze": rooms_per_hotel,
                            "date": date_str,
                            "category": configuration,
                            "shift": shift + 1,
                            "arrivals_ordini": arrivals,
                            "considered_orders": considered_orders,
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
                            "varfact": variability_factor,
                            "redistributed": redistributed,
                            "cancelled": cancelled
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
            
            output_data = {
                "Hotel List": data
            }
            DataEncoder.compressJson(output_data, 'hotel_data.json.gz')
            
            if debugMode:
                with pd.ExcelWriter('generazione.xlsx', engine='openpyxl') as writer:
                    df.to_excel(writer, sheet_name='RealData', index=False)
                    metricsData.to_excel(writer, sheet_name='Metrics', index=False)
                    return df, metricsData, output_data
            else:
                return df, metricsData, output_data
        else:
            print("Dataset non valido")
            print("Rigenerazione in corso....")

    return None, None


def redistribute_bookings(unfilled_bookings, remaining_single, remaining_double, remaining_triple, hotel_capacities, cancellation_prob=0.8):
    redistributed_bookings = 0
    remaining_bookings = unfilled_bookings
    
    for hotel, capacities in hotel_capacities.items():
        if remaining_bookings == 0:
            break
        
        # Calculate the number of rooms available in each category
        available_single = capacities["single"]
        available_double = capacities["double"]
        available_triple = capacities["triple"]
        
        # Fill single rooms
        if available_single > 0:
            fill_single = min(remaining_bookings, available_single)
            redistributed_bookings += fill_single
            remaining_bookings -= fill_single
        
        # Fill double rooms
        if available_double > 0 and remaining_bookings > 0:
            fill_double = min(remaining_bookings // 2, available_double)
            redistributed_bookings += fill_double * 2
            remaining_bookings -= fill_double * 2
        
        # Fill triple rooms
        if available_triple > 0 and remaining_bookings > 0:
            fill_triple = min(remaining_bookings // 3, available_triple)
            redistributed_bookings += fill_triple * 3
            remaining_bookings -= fill_triple * 3
    
    # Determine if the booking is canceled based on probability
    if remaining_bookings > 0 and random.random() < cancellation_prob:
        return redistributed_bookings, True  # Booking canceled

    return redistributed_bookings, False  # Booking not canceled


    
    
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
        
        
    
