# data_gatherer.py

import snowflake.connector
import pandas as pd
from Config.dbConfig import SNOWFLAKE_CONFIG  # Import corretto per la struttura

def fetch_data(query):
    try:
        # Connessione a Snowflake
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
