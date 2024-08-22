""" Contiene i modelli per analizzare le serie temporali di dati  """
import numpy as np
import pandas as pd
from prophet import Prophet
import matplotlib.pyplot as plt
from sklearn.metrics import mean_absolute_error, mean_squared_error

class Models:
    def __init__(self):
        pass
    
    def calculate_metrics(self, actual, predicted):
        """Calcola e stampa le metriche di valutazione."""
        mae = mean_absolute_error(actual, predicted)
        mse = mean_squared_error(actual, predicted)
        rmse = np.sqrt(mse)
        mape = np.mean(np.abs((actual - predicted) / actual)) * 100
        
        print(f"MAE: {mae:.2f}")
        print(f"MSE: {mse:.2f}")
        print(f"RMSE: {rmse:.2f}")
        print(f"MAPE: {mape:.2f}%")
        
        return mae, mse, rmse, mape
    
    def prophet_with_customizations(self):
        # Carica i dati
        df = pd.read_excel("../generazione.xlsx")
        
        # Verifica la presenza di valori mancanti
        if df.isnull().values.any():
            df = df.dropna()  # Rimuove le righe con valori mancanti o gestiscile diversamente
        
        # Filtra e rinomina le colonne per Prophet
        df_filtered = df[["date", "total_revenue"]].copy()
        df_filtered = df_filtered.rename(columns={"date": "ds", "total_revenue": "y"})
        
        # Assicurati che la colonna 'ds' sia di tipo datetime
        df_filtered['ds'] = pd.to_datetime(df_filtered['ds'])
        
        # Aggiungi una colonna di capacità per la crescita logistica
        df_filtered['cap'] = 100
        
        # Inizializza il modello Prophet
        m = Prophet(growth='logistic', interval_width=0.95)
        
        # Aggiungi stagionalità personalizzata
        m.add_seasonality(name='quarterly', period=91.25, fourier_order=8)
        
        # Fitta il modello con i dati storici
        m.fit(df_filtered)
        
        # Crea un dataframe per il futuro
        future = m.make_future_dataframe(periods=365)
        
        # Assicurati che anche 'future' abbia la colonna 'ds' di tipo datetime
        future['ds'] = pd.to_datetime(future['ds'])
        
        # Aggiungi la colonna 'cap' al dataframe futuro
        future['cap'] = 100
        
        # Fai la previsione
        forecast = m.predict(future)
        
        # Stampa le ultime previsioni
        print(forecast[["ds", "yhat"]].tail())
        
        # Calcola le metriche di valutazione sulle previsioni per il periodo di training
        actual = df_filtered['y']
        predicted = forecast.loc[forecast['ds'].isin(df_filtered['ds']), 'yhat']
        self.calculate_metrics(actual, predicted)
        
        # Visualizza i grafici
        fig1 = m.plot(forecast)
        fig2 = m.plot_components(forecast)
        
        plt.show()
        
        return forecast, fig1, fig2


if __name__ == "__main__":
    model = Models()
    model.prophet_with_customizations()  # Chiama la funzione con personalizzazioni
