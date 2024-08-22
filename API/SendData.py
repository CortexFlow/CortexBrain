import sys
import os
# Add the parent directory to the path
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))
from utils.jsonEncoding import decompressJson
import requests
from pymongo import MongoClient
from pymongo.errors import ConnectionFailure
import json
import concurrent.futures


class SendData:
    @staticmethod
    def ClusterCredentials():
        # MongoDB cluster credentials
        username = input("Insert your username: ")
        password = input("Insert your password: ")
        return username, password

    @staticmethod
    def ServerCredentials():
        # Backend server credentials
        email = input("Insert your backend username: ")
        password = input("Insert your backend password: ")
        return email, password

    @staticmethod
    def LoginToCluster():
        # Login to your MongoDB cluster
        # Get your credentials from the config file
        with open('../Config/dbConfig.json', 'r') as file:
            params = json.load(file)
        
        user, password = SendData.ClusterCredentials()
        try:
            MONGODB_CONFIG = params["MongoDb"]
            url = f"mongodb+srv://{user}:{password}@{MONGODB_CONFIG['cluster']}/{MONGODB_CONFIG['dbName']}"
            client = MongoClient(url)
            client.admin.command("ping")
            print("Cluster Available!")
            return True
        except ConnectionFailure:
            print("Error. Cluster offline")
            return False

    @staticmethod
    def LoginToServer(session):
        email, password = SendData.ServerCredentials()
        login_data = {"email": email, "password": password}
        url = "http://localhost:5000/api/user/login"

        try:
            req = session.post(url, json=login_data)
            req.raise_for_status()  # Verify the request status
            print("Logged in!")
            return True
        except requests.exceptions.RequestException as e:
            print(f"Login failed: {e}")
            return False

    @staticmethod
    def batchData(data, batch_size):
        """Send data in batches."""
        for i in range(0, len(data), batch_size):
            yield data[i:i + batch_size]

    @staticmethod
    def SendBatch(session, url, jsonData):
        try:
            req = session.post(url, json=jsonData)
            req.raise_for_status()
            print("Data sent successfully")
        except requests.exceptions.RequestException as e:
            print(f"Error sending batch: {e}")
            print(f"Response: {req.text}")

    @staticmethod
    def pushDataToServer(session, jsonData, batch_size=100):
        """
        Send data to the server in batches.

        Args:
        - session: request session to maintain connection.
        - jsonData: JSON data to send.
        - batch_size: number of records per batch.
        """
        url = "http://localhost:5000/api/analytics/upload-test-data"

        # Split data into batches
        data_list = jsonData["Hotel List"]
        batches = SendData.batchData(data_list, batch_size)
        
        with concurrent.futures.ThreadPoolExecutor() as executor:
            futures = [executor.submit(SendData.SendBatch, session, url, {"Hotel List": batch}) for batch in batches]
            
            # Wait for all batches to be sent
            for future in concurrent.futures.as_completed(futures):
                try:
                    future.result()
                except Exception as exc:
                    print(f"Error sending batch: {exc}")

if __name__ == "__main__":
    jsonData = {
        "Hotel List": [
            {
                "HotelName": "Hotel 1",
                "metrics": {
                    "number_of_rooms": 120,
                    "arrival_orders": [
                        {
                            "date": "2023-08-01T10:00:00Z",
                            "number": 10
                        },
                        {
                            "date": "2023-08-02T10:00:00Z",
                            "number": 12
                        }
                    ],
                    "revenue_single": 5000.50,
                    "revenue_double": 7000.75,
                    "revenue_triple": 8500.30,
                    "total_revenue": 20500.55,
                    "room_price_single": 200.00,
                    "room_price_double": 350.00,
                    "room_price_triple": 450.00,
                    "num_families": 20,
                    "num_couples": 15,
                    "num_single_guests": 5
                }
            }
        ]
    }

    # Initialize the session
    session = requests.Session()

    # Log in to the cluster and the server
    if SendData.LoginToCluster() and SendData.LoginToServer(session):
        SendData.pushDataToServer(session, jsonData)
    else:
        print("Connection to the cluster or server login failed")
