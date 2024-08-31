import os
import sys
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '../')))

from SyntheticDatas.Sensors.GPSModel import GPS_Sensor

if __name__=="__main__":
    print("\nTesting GPS Sensor (Moving)")
    start_position = [0.0, 0.0]
    speed = 0.11  # Speed in units per second
    direction = 'E'  # Direction (N, S, E, W)
    
    #create gps sensor model using the constructor
    moving_gps_sensor = GPS_Sensor(initial_position=start_position, speed=speed, direction=direction, label="Moving GPS Sensor")

    #retrive the status of the sensor
    moving_gps_sensor.GetStatus()
    
    #run the simulation
    moving_gps_sensor.SimulateMovement(duration=10)  # Simulate for 10 seconds
