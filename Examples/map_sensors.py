import os
import sys
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '../')))

from SyntheticDatas.Sensors.sensorModel import GPS_Sensor
from SyntheticDatas.Sensors.sensorModel import Map

if __name__=="__main__":
    #create the sensors using the constructors
    gps_sensor_1 = GPS_Sensor(initial_position=[0, 0], label="Static GPS Sensor ")
    gps_sensor_2 = GPS_Sensor(initial_position=[0, 0], label="Static GPS Sensor 2")
    
    #set the positions
    gps_sensor_1.SetPosition((45.812460, 8.986586))
    gps_sensor_2.SetPosition((45.832460, 8.986586))
    
    #create the map
    map=Map()
    #add the sensors on the map
    map.AddSensor(gps_sensor_1)
    map.AddSensor(gps_sensor_2)
    #create the sensor_map.html file
    map.CreateMap()