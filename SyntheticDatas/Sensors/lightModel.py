from BaseSensor import Sensor

class Light(Sensor):
    def __init__(self, position,label="Smart Light"):
        super().__init__(SensorType="Light",value=[0.0,0.0],label=label)
        self.lat=float(position[0])
        self.lon=float(position[1])
        self.label=label
    
    def SetPosition(self, position):
        self.lat = float(position[0])
        self.lon = float(position[1])

    def GetPosition(self):
        return (self.lat, self.lon)
    

    def GetStatus(self):
        """Prints the current status of the light sensor."""
        print("-----------------------------")
        print("Sensor Status:")
        print(f"Name: {self.name}")
        print(f"Coordinates: {self.GetPosition()}")
        print("-----------------------------")
        
        
if __name__=="__main__":
    light = Light(position=[0.0,0.0],label="My Light")
    light.GetStatus()
    