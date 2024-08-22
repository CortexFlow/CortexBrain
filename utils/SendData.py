from utils.jsonEncoding import decompressJson
import requests
from pymongo import MongoClient
from pymongo.errors import ConnectionFailure
import json
import sys
import os
import concurrent.futures

# Aggiungi la directory parent alla variabile di percorso
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))


class SendData:
    """ N.B Si possono implementare programmazione parallela/caching e divisione dei dati in batch """
    @staticmethod
    def ClusterCredentials():
        username = input("Inserisci username per il cluster: ")
        password = input("Inserisci password per il cluster: ")
        return username, password

    @staticmethod
    def ServerCredentials():
        email = input("Inserisci email per il server: ")
        password = input("Inserisci password per il server: ")
        return email, password

    @staticmethod
    def LoginToCluster():
        user, password = SendData.ClusterCredentials()
        try:
            url = f"mongodb+srv://{user}:{
                password}@cluster0.agcz3gt.mongodb.net/EcommerceBackend"
            client = MongoClient(url)
            client.admin.command("ping")
            print("Connessione riuscita, cluster disponibile!")
            return True
        except ConnectionFailure:
            print("Errore. Server Offline")
            return False

    @staticmethod
    def LoginToServer(session):
        email, password = SendData.ServerCredentials()
        login_data = {"email": email, "password": password}
        url = "http://localhost:5000/api/user/login"

        try:
            req = session.post(url, json=login_data)
            req.raise_for_status()  # Verifica che la richiesta sia andata a buon fine
            print("Login al server riuscito!")
            return True
        except requests.exceptions.RequestException as e:
            print(f"Login non riuscito: {e}")
            return False

    @staticmethod
    def batchData(data, batch_size):
        """Divide i dati in batch."""
        for i in range(0, len(data), batch_size):
            yield data[i:i + batch_size]

    @staticmethod
    def SendBatch(session, url, jsonData):
        try:
            req = session.post(url, json=jsonData)
            req.raise_for_status()
            print("Dato inviato con successo")
        except requests.exceptions.RequestException as e:
            print(f"Errore durante l'invio del batch: {e}")
            print(f"Risposta: {req.text}")

    @staticmethod
    def pushDataToServer(session, jsonData, batch_size=100):
        """
        Invia i dati al server in batch.

        Args:
        - session: sessione di richieste per mantenere la connessione.
        - jsonData: dati JSON da inviare.
        - batch_size: numero di record per batch.
        """
        url = "http://localhost:5000/api/analytics/upload-test-data"

        # Suddividi i dati in batch
        data_list = jsonData["Hotel List"]
        batches=SendData.batchData(data_list,batch_size)
        

        with concurrent.futures.ThreadPoolExecutor() as executor:
            futures = [executor.submit(SendData.SendBatch, session, url, {"Hotel List":batch}) for batch in batches]
            
            #aspetta che tutti i batch siano stati inviati
            for future in concurrent.futures.as_completed(futures):
                try:
                    future.result()
                except Exception as exc:
                    print(f"Errore durante l'invio del batch:{exc}")

if __name__ == "__main__":
    # jsonData = decompressJson("../hotel_data.json.gz") #decomprime l'archivio contenente i dati da inviare
    jsonData = {
        "HotelName": "Hotel 1",
        "metrics": {
            "numero_stanze": 120,
            "arrivals_ordini": [
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

    # Inizializza la sessione
    session = requests.Session()

    # Effettua il login al cluster e al server
    if SendData.LoginToCluster() and SendData.LoginToServer(session):
        SendData.pushDataToServer(session, jsonData)
    else:
        print("Connessione con il cluster o login al server fallito")
