""" Contains models for analyzing time series data """
import numpy as np
import pandas as pd
from prophet import Prophet
import matplotlib.pyplot as plt
from sklearn.metrics import mean_absolute_error, mean_squared_error

class Models:
    def __init__(self):
        pass
    
    def calculate_metrics(self, actual, predicted):
        """Calculates and prints evaluation metrics."""
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
        # Load data
        df = pd.read_excel("../SyntheticDatas/output/generazione.xlsx")
        
        # Check for missing values
        if df.isnull().values.any():
            df = df.dropna()  # Remove rows with missing values or handle them differently
        
        # Filter and rename columns for Prophet
        df_filtered = df[["date", "arrivals_ordini"]].copy()
        df_filtered = df_filtered.rename(columns={"date": "ds", "arrivals_ordini": "y"})
        
        # Ensure the 'ds' column is of datetime type
        df_filtered['ds'] = pd.to_datetime(df_filtered['ds'])
        
        # Add a capacity column for logistic growth
        df_filtered['cap'] = 100
        
        # Initialize the Prophet model
        m = Prophet(growth='logistic', interval_width=0.95)
        
        # Add custom seasonality
        m.add_seasonality(name='quarterly', period=91.25, fourier_order=8)
        
        # Fit the model with historical data
        m.fit(df_filtered)
        
        # Create a dataframe for the future
        future = m.make_future_dataframe(periods=365)
        
        # Ensure that 'future' also has the 'ds' column as datetime
        future['ds'] = pd.to_datetime(future['ds'])
        
        # Add the 'cap' column to the future dataframe
        future['cap'] = 100
        
        # Make the prediction
        forecast = m.predict(future)
        
        # Print the latest predictions
        print(forecast[["ds", "yhat"]].tail())
        
        # Calculate evaluation metrics on predictions for the training period
        actual = df_filtered['y']
        predicted = forecast.loc[forecast['ds'].isin(df_filtered['ds']), 'yhat']
        self.calculate_metrics(actual, predicted)
        
        # Display the plots
        fig1 = m.plot(forecast)
        fig2 = m.plot_components(forecast)
        
        plt.show()
        
        return forecast, fig1, fig2


if __name__ == "__main__":
    model = Models()
    model.prophet_with_customizations()  # Call the function with customizations
