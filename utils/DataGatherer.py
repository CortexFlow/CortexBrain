# data_gatherer.py
import snowflake.connector
import pandas as pd
import json
import os

def fetchFromSnowFlake(query):
    # load parameters from json file
    with open('../Config/dbConfig.json', 'r') as file:
        params = json.load(file)
    try:
        SNOWFLAKE_CONFIG = params["Snowflake"]
        # connect to snowflake services
        conn = snowflake.connector.connect(
            user=SNOWFLAKE_CONFIG['user'],
            password=SNOWFLAKE_CONFIG['password'],
            account=SNOWFLAKE_CONFIG['account'],
            role=SNOWFLAKE_CONFIG['role'],
            warehouse=SNOWFLAKE_CONFIG['warehouse'],
            database=SNOWFLAKE_CONFIG['database'],
            schema=SNOWFLAKE_CONFIG['schema']
        )

        # Esegui la query
        cur = conn.cursor()
        cur.execute(query)

        # Recupera i risultati come un DataFrame di Pandas
        df = cur.fetch_pandas_all()
        return df

    except snowflake.connector.errors.Error as e:
        print(f"Error: {e}")
        return None

    finally:
        # Chiudi la connessione
        if conn:
            conn.close()



if __name__=="__main__":
    # Definisci la query
    query_CHATS = "SELECT * FROM CHATS"
    query_HOTELS = "SELECT * FROM HOTELS"
    query_USERS = "SELECT * FROM USERS"

    # Recupera i dati
    df_chats = fetchFromSnowFlake(query_CHATS)
    df_hotels = fetchFromSnowFlake(query_HOTELS)
    df_users = fetchFromSnowFlake(query_USERS)

    # Definisci il percorso di salvataggio
    path = r"./output"

    # Crea la directory se non esiste
    os.makedirs(path, exist_ok=True)

    # Funzione per salvare i dati in CSV
    def save_to_csv(df, filename):
        if df is not None:
            print(df)
            file_path = os.path.join(path, filename)
            df.to_csv(file_path, index=False)
            print(f"Dati esportati con successo in '{file_path}'")
        else:
            print(f"Errore nel recupero dei dati per {filename}")

    # Salva i dati in CSV
    save_to_csv(df_chats, 'Chat-output.csv')
    save_to_csv(df_hotels, 'Hotels-output.csv')
    save_to_csv(df_users, 'Users-output.csv')
