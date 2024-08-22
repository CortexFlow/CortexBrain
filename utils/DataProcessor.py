import matplotlib.pyplot as plt
from matplotlib import style 
import json


class DataProcessor:
    def CompareDatas1D(data, key):
        try:
            datas = json.loads(data)  # Converte la stringa JSON in un dizionario
        except json.JSONDecodeError:
            print("Errore nella decodifica del JSON.")
            return None

        
        # Controlla se la chiave "Hotel List" esiste e se la lista è vuota
        if "Hotel List" not in datas or not datas["Hotel List"]:
            print("No data available")
            return None
        
        # Accediamo alla lista sotto la chiave principale 'Hotel List'
        hotel_list = datas["Hotel List"]
        
        # Verifica che la chiave esista in ogni dizionario nella lista
        if not all(key in entry for entry in hotel_list):
            print(f"La chiave '{key}' non è presente in tutti gli elementi della lista.")
            return None
        
        # Estraiamo le date e il numero di arrivi usando la chiave degli arrivi fornita
        dates = [entry["date"] for entry in hotel_list]
        setY = [entry[key] for entry in hotel_list]
        
        # Creiamo il grafico
        plt.figure(figsize=(10, 6))
        plt.style.use('ggplot')
        plt.plot(dates, setY, color='b', marker='o')
        plt.xticks(fontsize=8)  # Puoi cambiare 14 con la dimensione del font che desideri
        
        # Aggiungiamo i titoli e le etichette agli assi
        plt.title(f'({key}) nel Tempo')
        plt.xlabel('Data')
        plt.ylabel(f'({key})')
        
        # Mostriamo il grafico
        plt.grid(True)
        plt.xticks(rotation=45)  # Rotazione delle etichette delle date per una migliore leggibilità
        plt.tight_layout()  # Per evitare il taglio delle etichette
        plt.show()

        return None
    
    
    def CompareDatas2D(data,key1,key2):
        # Accediamo alla lista sotto la chiave principale 'Hotel List'
        hotel_list = data['Hotel List']
        
        # Estraiamo le date e il numero di arrivi usando la chiave degli arrivi fornita
        setX = [entry[key1] for entry in hotel_list]
        setY = [entry[key2] for entry in hotel_list]
        
        # Creiamo il grafico
        plt.figure(figsize=(20,12))
        # using the style for the plot 
        plt.style.use('ggplot') 
        plt.scatter(setX, setY,color='b')
        
        # Aggiungiamo i titoli e le etichette agli assi
        plt.title(f'Confronto ({key1}) - ({key2})')
        plt.xlabel(f'({key1})')
        plt.ylabel(f'({key2})')
        
        # Mostriamo il grafico
        plt.grid(True)
        plt.show()
        return 0
    
    def calculate_distance_factor(distance):
        """Calculate the distance factor based on the distance ranges provided.
            nella versione 2 del generatore si ha un fattore moltiplicativo
        """
        if 0.5 <= distance <= 1.5:
            return 20
        elif 1.5 < distance <= 2:
            return 10
        elif 2 < distance <= 5:
            return 1 / distance
        elif 5 < distance <= 10:
            return 1 / (distance * distance)
        else:
            return 1 / 100
        
    def distribuisci_persone(numero_persone, camere_singole, camere_doppie, camere_triple):
        singole_utilizzate = 0
        doppie_utilizzate = 0
        triple_utilizzate = 0
        
        # Distribuisci persone nelle camere triple
        while numero_persone >= 3 and camere_triple > 0:
            numero_persone -= 3
            camere_triple -= 1
            triple_utilizzate += 1
            
        # Distribuisci persone nelle camere doppie
        while numero_persone >= 2 and camere_doppie > 0:
            numero_persone -= 2
            camere_doppie -= 1
            doppie_utilizzate += 1
            
        # Distribuisci persone nelle camere singole
        while numero_persone >= 1 and camere_singole > 0:
            numero_persone -= 1
            camere_singole -= 1
            singole_utilizzate += 1
        
        return singole_utilizzate, doppie_utilizzate, triple_utilizzate, numero_persone

