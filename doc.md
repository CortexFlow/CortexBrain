# Documentation
Full documentation of the CortexBrain library
### Getting Started

To get started with CortexBrain, follow these steps:

1. **Clone the Repository**: First, clone the repository to your local machine.

   ```bash
   git clone https://github.com/CortexFlow/CortexBrain.git
    ```

2. **Install required packages**:

   ```bash
   cd CortexBrain
   pip install -r requirements.txt

3. **Test Library**:

   ```bash
   python testLib.py

### Functions and Examples
# Class: Sensor
The Sensor class is a base class for various types of sensors. It encapsulates basic functionalities such as reading and updating the sensor's value, and it includes attributes for the sensor's type, label, and current value.

## Class Methods and Attributes
## Attributes:

name: A string representing the label or name of the sensor.
type: A string representing the type of sensor (e.g., Temperature, Humidity).
value: A float representing the current value of the sensor. Defaults to 0.0.
Methods:

__init__(self, SensorType, value=0.0, label="Sensor"): Initializes the sensor object with the specified type, value, and label.
__del__(self): Destructor method, called when the object is deleted. It prints a message indicating that the sensor has been deleted.
ReadValue(self): Returns the current value of the sensor.
UpdateValue(self, new_value): Updates the sensor's value with the given new_value and returns the updated value.

# Class: GPS_Sensor
The GPS_Sensor class inherits from the Sensor class and models a GPS sensor. It includes attributes for latitude, longitude, speed, and direction, along with methods for updating position, checking if the sensor is moving, and simulating movement over a period of time.

## Class Methods and Attributes
## Attributes:

lat: The latitude of the sensor.
lon: The longitude of the sensor.
speed: The speed of the sensor in units per second.
direction: The direction of movement ('N', 'S', 'E', 'W').
is_moving: A boolean indicating whether the sensor is currently moving.
Methods:

__init__(self, initial_position, speed=0.0, direction='N', label="GPS Sensor"): Initializes the GPS sensor with the given position, speed, direction, and label.
SetPosition(self, position): Sets the GPS sensor's position to the specified latitude and longitude.
GetSpeed(self): Returns the current speed of the sensor.
SetSpeed(self, speed): Sets the speed of the GPS sensor.
isMoving(self): Returns whether the GPS sensor is currently moving.
GetStatus(self): Prints the current status of the GPS sensor.
UpdateDirection(self): Updates the sensor's position based on its speed and direction.
SimulateMovement(self, duration): Simulates the movement of the sensor for a specified duration.

# Class: Map
The Map class manages a collection of sensors using a graph-based structure and provides functionalities to generate an interactive map displaying these sensors. The class uses networkx for graph management and folium for map visualization. Sensors are added as nodes in the graph, and their positions are marked on the map.

## Class Methods and Attributes
## Attributes:

G: A networkx.Graph object that stores sensors as nodes, with attributes such as position and category.
Methods:

__init__(self): Initializes the graph object G.
AddSensor(self, sensor): Adds a GPS sensor to the graph, storing its position and category.
CreateMap(self): Generates and saves an interactive map with markers representing the sensors.



### API
The api integration will be featured in the next release