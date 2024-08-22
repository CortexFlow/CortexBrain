from utils.jsonEncoding import decompressJson
from Analyze.analyze import add_mean, add_std, add_sum, classify_hotels
import json
from utils.DataProcessor import DataProcessor
from SyntheticDatas.generateDatas import GenerateHotelDatas
import time

from utils.SendData import SendData
import json
import requests


def AggregateJson(*jsons):
    # Inizializza un dizionario per memorizzare i dati aggregati
    aggregated_data = {}

    # Flag per verificare se ci sono dati aggregabili
    data_found = False

    for data in jsons:
        # Carica il JSON dalla stringa
        json_data = json.loads(data)

        # Se il JSON è vuoto o non contiene dati aggregabili, salta l'elaborazione
        if not json_data or not list(json_data.values())[0]:
            continue

        # Estrai la categoria e il tipo di aggregazione dal JSON
        category = list(json_data.keys())[0]

        if category not in aggregated_data:
            aggregated_data[category] = {}

        # Elenco dei tipi di aggregazione
        for aggregation_type, content in json_data[category].items():
            if aggregation_type not in aggregated_data[category]:
                aggregated_data[category][aggregation_type] = {
                    "total_revenue": {}}

            # Estrai i dati del revenue per gli hotel
            revenue_data = content.get("total_revenue", {})

            # Aggrega i dati degli hotel
            for hotel, revenue in revenue_data.items():
                if hotel not in aggregated_data[category][aggregation_type]["total_revenue"]:
                    aggregated_data[category][aggregation_type]["total_revenue"][hotel] = revenue
                else:
                    # Se l'hotel è già presente, somma il valore
                    aggregated_data[category][aggregation_type]["total_revenue"][hotel] += revenue

            # Imposta il flag a True se sono stati trovati dati
            data_found = True

    # Verifica se ci sono dati aggregabili
    if not data_found:
        return json.dumps({"message": "dati non aggregati, nessun dato disponibile"}, indent=4)

    return json.dumps(aggregated_data, indent=4)


if __name__ == "__main__":
    # Carica i parametri dal file JSON
    with open('./SyntheticDatas/params.json', 'r') as file:
        params = json.load(file)

    # Genera i dataset e le metriche e viene misurato il tempo
    start_time = time.perf_counter()  # Memorizza l'orario di inizio

    datas, metrics, dict_data = GenerateHotelDatas(params=params,
                                                   key="Mare",
                                                   debugMode=True)

    end_time = time.perf_counter()  # Memorizza l'orario di fine
    elapsed_time = end_time - start_time  # Calcola il tempo trascorso

    print("Tempo di generazione:", round(elapsed_time, 2), "secondi")
    
    #prova di invio dati al server
    # Inizializza la sessione
    session = requests.Session()

    # Effettua il login al cluster e al server
    if SendData.LoginToCluster() and SendData.LoginToServer(session):
        SendData.pushDataToServer(session, dict_data,batch_size=1000)
    else:
        print("Connessione con il cluster o login al server fallito")



    """ LHotel, SHotel, BHotel = classify_hotels(jsonDatas)

    JsonDataMean = add_mean(LHotel, "total_revenue")
    JsonDataDevStd = add_std(LHotel, "total_revenue")
    JsonDataDevSum = add_sum(LHotel, "total_revenue")

    JsonDataMean2 = add_mean(BHotel, "total_revenue")
    JsonDataDevStd2 = add_std(BHotel, "total_revenue")
    JsonDataDevSum2 = add_sum(BHotel, "total_revenue")

    JsonDataMean3 = add_mean(SHotel, "total_revenue")
    JsonDataDevStd3 = add_std(SHotel, "total_revenue")
    JsonDataDevSum3 = add_sum(SHotel, "total_revenue")

    print("Test")
    print("Category L")
    print(JsonDataMean)
    print(JsonDataDevStd)
    print(JsonDataDevSum)
    print("\n")
    print("Category B")
    print(JsonDataMean2)
    print(JsonDataDevStd2)
    print(JsonDataDevSum2)
    print("\n")
    print("Category S")
    print(JsonDataMean3)
    print(JsonDataDevStd3)
    print(JsonDataDevSum3)

    print("\n")
    LHotel_ = AggregateJson(JsonDataMean, JsonDataDevStd, JsonDataDevSum)
    SHotel_ = AggregateJson(JsonDataMean2, JsonDataDevStd2, JsonDataDevSum2)
    BHotel_ = AggregateJson(JsonDataMean3, JsonDataDevStd3, JsonDataDevSum3)
    print("Json Aggregato per la categoria L", LHotel_)
    print("\n")
    print("Json Aggregato per la categoria S", SHotel_)
    print("\n")
    print("Json Aggregato per la categoria B", BHotel_) """

    # DataProcessor.CompareDatas1D(LHotel,"total_revenue(€)")
    # DataProcessor.CompareDatas1D(SHotel,"total_revenue(€)")
    # DataProcessor.CompareDatas1D(BHotel,"total_revenue(€)")
    # DataProcessor.CompareDatas2D(SHotel,"total_revenue(€)","num_families") #da fixare
