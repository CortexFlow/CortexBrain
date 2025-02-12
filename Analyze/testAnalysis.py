import sys
import os
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))
from utils.jsonEncoder import DataEncoder
from Analyze.analyze import get_mean, get_std, get_sum, classify_hotels
import json
from utils.DataProcessor import DataProcessor
from SyntheticDatas.generateDatas import GenerateHotelDatas
import time

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
    with open(os.path.join(os.path.dirname(__file__), '..', 'SyntheticDatas', 'params.json'), 'r') as file:
        params = json.load(file)

    # Genera i dataset e le metriche e viene misurato il tempo
    start_time = time.perf_counter()  # Memorizza l'orario di inizio

    datas, metrics, dict_data = GenerateHotelDatas(params=params,
                                                   key="Mare",
                                                   debugMode=True)

    end_time = time.perf_counter()  # Memorizza l'orario di fine
    elapsed_time = end_time - start_time  # Calcola il tempo trascorso

    print("Tempo di generazione:", round(elapsed_time, 2), "secondi")
    print(dict_data)

    jsonData=json.dumps(dict_data)
    LHotel, SHotel, BHotel = classify_hotels(jsonData)

    JsonDataMean = get_mean(LHotel, "total_revenue")
    JsonDataDevStd = get_std(LHotel, "total_revenue")
    JsonDataDevSum = get_sum(LHotel, "total_revenue")

    JsonDataMean2 = get_mean(BHotel, "total_revenue")
    JsonDataDevStd2 = get_std(BHotel, "total_revenue")
    JsonDataDevSum2 = get_sum(BHotel, "total_revenue")

    JsonDataMean3 = get_mean(SHotel, "total_revenue")
    JsonDataDevStd3 = get_std(SHotel, "total_revenue")
    JsonDataDevSum3 = get_sum(SHotel, "total_revenue")

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
    print("Json Aggregato per la categoria B", BHotel_)

    # DataProcessor.CompareDatas1D(LHotel,"total_revenue(€)")
    # DataProcessor.CompareDatas1D(SHotel,"total_revenue(€)")
    # DataProcessor.CompareDatas1D(BHotel,"total_revenue(€)")
    # DataProcessor.CompareDatas2D(SHotel,"total_revenue(€)","num_families") #da fixare