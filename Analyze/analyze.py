""" Contains code for classifying hotels and calculating simple metrics """
import os
import sys
import numpy as np
import pandas as pd
import json
from datetime import date, timedelta

# Add the parent directory to the path variable
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))

#from SyntheticDatas.generateDatas import generateDataset



# Function to classify hotels into categories
def classify_hotels(json_file):
    # Convert JSON to Python dictionary
    data = json.loads(json_file)

    # Check if the hotel list is empty
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
   
# FUNCTIONS FOR CALCULATING METRICS

# Calculates and returns the average of a selected key for each hotel in a JSON file
def get_mean(json_file, key):
    # Convert JSON to a Python dictionary
    data = json.loads(json_file)

    # Check if the hotel list is empty
    if not data["Hotel List"]:
       print("Hotel List is empty")
       return json_file

    # Check if the key exists in at least one hotel
    if not any(key in hotel for hotel in data["Hotel List"]):
        print(f"key {key} does not exist")
        return json_file

    # Dictionary to accumulate the key values for each hotel
    hotel_values = {}

    # Collect the specified key values for each hotel
    for hotel in data["Hotel List"]:
        hotel_name = hotel["HotelName"]
        if key in hotel:
            if hotel_name not in hotel_values:
                hotel_values[hotel_name] = []
            hotel_values[hotel_name].append(hotel[key])

    # Calculate the mean for each hotel
    hotel_means = {hotel: round(np.mean(values),2) if values else 0 for hotel, values in hotel_values.items()}

    # Create the structure of the new JSON with the "Mean" key inside "Hotel List"
    result_data = {
        "Hotel List": {
            "Mean": {
              f"{key}" :hotel_means
            }
        }
    }

    # Return the result as JSON
    return json.dumps(result_data)
    
# Calculates the average of a selected key for each hotel in a JSON file for the last quarter
def get_mean_trimester(json_file, key):
    # Convert JSON to Python dictionary
    data = json.loads(json_file)

    # Check if the hotel list is empty
    if not data["Hotel List"]:
       print("Hotel List is empty")
       return json_file 
    
    # Check if the key exists in at least one hotel
    if not any(key in hotel for hotel in data["Hotel List"]):
        print(f"key {key} does not exist")
        return json_file
    
    # Dictionary to accumulate the key values for each hotel
    hotel_values = {}
    
    # Current date
    today = date.today()

    # Collect the specified key values for each hotel
    for hotel in data["Hotel List"]:
        hotel_name = hotel["HotelName"]
        if key in hotel:
            mm_yyyy = hotel["date"].split("-") # Can be changed to handle DD-MM-YYYY dates
            month = float(mm_yyyy[0])
            if hotel_name not in hotel_values:
                hotel_values[hotel_name] = []
            if month > today.month-3: # Chooses the last three consecutive months
                hotel_values[hotel_name].append(hotel[key])

    # Calculate the mean for each hotel
    hotel_means = {hotel: round(np.mean(values),2) if values else 0 for hotel, values in hotel_values.items()}

    # Create the structure of the new JSON with the "Mean" key inside "Hotel List"
    result_data = {
        "Hotel List": {
            "Mean_Last_Trimester": {
              f"{key}" :hotel_means
            }
        }
    }

    # Return the result as JSON
    return json.dumps(result_data)
    
# Calculates the standard deviation of a selected key in a JSON file
def get_std(json_file, key):
    # Convert JSON to a Python dictionary
    data = json.loads(json_file)

    # Check if the hotel list is empty
    if not data.get("Hotel List"):
        print("Hotel List is empty")
        return json_file

    # Check if the key exists in at least one hotel
    if not any(key in hotel for hotel in data["Hotel List"]):
        print(f"key {key} does not exist")
        return json_file

    # Dictionary to accumulate the key values for each hotel
    hotel_values = {}

    # Collect the specified key values for each hotel
    for hotel in data["Hotel List"]:
        hotel_name = hotel["HotelName"]
        if key in hotel:
            if hotel_name not in hotel_values:
                hotel_values[hotel_name] = []
            hotel_values[hotel_name].append(hotel[key])

    # Calculate the standard deviation for each hotel
    hotel_std_devs = {hotel: round(np.std(values),2) if values else 0 for hotel, values in hotel_values.items()}

    # Create the structure of the new JSON with the "DevStd" key inside "Hotel List"
    result_data = {
        "Hotel List": {
            "DevStd": {
               f"{key}":hotel_std_devs
            }
        }
    }

    # Return the result as JSON
    return json.dumps(result_data)
      
def get_sum(json_file, key):
    # Convert JSON to a Python dictionary
    data = json.loads(json_file)

    # Check if the hotel list is empty
    if not data.get("Hotel List"):
        print("Hotel List is empty")
        return json_file

    # Check if the key exists in at least one hotel
    if not any(key in hotel for hotel in data["Hotel List"]):
        print(f"key {key} does not exist")
        return json_file

    # Dictionary to accumulate sums for each hotel
    hotel_sums = {}

    # Calculate the sum for each hotel
    for hotel in data["Hotel List"]:
        hotel_name = hotel["HotelName"]
        if key in hotel:
            if hotel_name not in hotel_sums:
                hotel_sums[hotel_name] = 0
            hotel_sums[hotel_name] += hotel[key]
            
    # Create the structure of the new JSON with the sums    
    result_data = {
        "Hotel List": {
           "Sum":{
              f"{key}":hotel_sums
         }
      }
   }

    # Return the result as JSON
    return json.dumps(result_data)
 
if __name__ == "__main__":
   # Python object (dict):
   test_hotel_data = {
      "Hotel List": [
         {
            "HotelName": "Hotel Freddo",
            "numero stanze": 300,
            "date": "09-2024",
            "category": "L",
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
            "HotelName": "Hotel Caldo",
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
            "HotelName": "Hotel Caldo",
            "numero stanze": 300,
            "date": "06-2024",
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

   # Convert to JSON:
   test_hotel_json = json.dumps(test_hotel_data)
   print("Debug test")
   print("Test Json")   
   print(test_hotel_json)
   print("\n")   
   
   L_hotel_json, S_hotel_json, B_hotel_json = classify_hotels(test_hotel_json)
   print("Hotel Category: L", L_hotel_json)
   print("\n")
   print("Hotel Category: S", S_hotel_json)
   print("\n")
   print("Hotel Category: B", B_hotel_json)
   print("\n")
   
   L_hotel_rev_mean = get_mean(L_hotel_json, "revenue_single(€)")
   
   L_hotel_rev_std = get_std(L_hotel_json, "revenue_single(€)")
   
   L_hotel_rev_mean_tri = get_mean_trimester(L_hotel_json, "revenue_single(€)") # still needs optimization
   
   # Note: An aggregator of parameters is needed
   
   print("Debug for L Category:")
   print(L_hotel_rev_mean)
   print(L_hotel_rev_std)
   print(L_hotel_rev_mean_tri)
