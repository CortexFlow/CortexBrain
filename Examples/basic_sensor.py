import os
import sys
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '../')))

from SyntheticDatas.Sensors.sensorModel import Sensor

if __name__=="__main__":
    #create a dumb sensor 
    sensor=Sensor(SensorType="Dumb Sensor",value=1,label="Sensor 1")
    
    #retrive basic sensor information
    print(f"Sensor Name: {sensor.name}")
    print(f"Sensor Value: {sensor.ReadValue()}")
    print(f"Sensor Type: {sensor.type}")
    
    #update sensor value
    sensor.UpdateValue(45.3)
    
    #retrive updated sensor information
    print(f"Updated Sensor Value: {sensor.ReadValue()}")