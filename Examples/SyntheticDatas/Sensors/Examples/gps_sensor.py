import os
import sys
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '../')))

from GPSModel import GPS_Sensor

if __name__=="__main__":
    print("\nTesting GPS Sensor (Static)")
    
    #create a Gps sensor model using the constructor
    static_gps_sensor = GPS_Sensor(initial_position=[45.712460, 8.986586], label="Static GPS Sensor")
    
    #retrieve sensor complete info
    static_gps_sensor.GetStatus()