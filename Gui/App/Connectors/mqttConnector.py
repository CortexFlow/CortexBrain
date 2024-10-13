import os
import sys
sys.path.append(os.path.abspath(
    os.path.join(os.path.dirname(__file__), '../')))
import paho.mqtt.client as mqtt
from Globals.imports import *
from Globals.constants import GLOBAL_VAR


class ConnectionEstablished(QMainWindow):
    def __init__(self):
        super(ConnectionEstablished, self).__init__()
        self.setWindowTitle(GLOBAL_VAR.CONNECTION_ESTABLISHED_TITLE)
        self.setWindowIcon(QIcon(GLOBAL_VAR.ICON))
        uic.loadUi(GLOBAL_VAR.CONNECTION_ESTABLISHED_UI, self)
        
        self.show()

class MQTTClient(QObject):
    # Signal to emit when the connection status changes
    status_changed = pyqtSignal(str)
    conn_status_changed = pyqtSignal(bool)

    def __init__(self, broker, port, client_id):
        super(MQTTClient, self).__init__()  # Properly call the super class __init__
        self.broker = broker
        self.port = port
        self.client_id = client_id
        self.client =  mqtt.Client(mqtt.CallbackAPIVersion.VERSION1, self.client_id)
        self.mqtt_status = None
        self.conn_status = None
    
    def connect_mqtt(self):
        def on_connect(client, userdata, flags, rc):
            if rc == 0:
                #self.mqtt_status = "Connected to MQTT broker"
                self.mqtt_status = str(0)
                self.conn_status=True #connection on
                print(f"connection status {self.conn_status}")
            else:
                self.mqtt_status = f"Connection failed, return code {rc}"
                self.conn_status=False #connection off
                self.conn_status_changed.emit(self.conn_status)
            # Emit the status changed signal
            self.status_changed.emit(self.mqtt_status)

        self.client.on_connect = on_connect
        self.client.connect(self.broker, self.port)
        self.client.loop_start()  # Start the loop to process network events

        
    def getStatus(self):
        return self.mqtt_status
    
    # Funzione di sottoscrizione
    def subscribe(self, topic):
        def on_message(client, userdata, message):
            print(f"Messaggio ricevuto: {message.payload.decode()} su topic {message.topic}")
            self.mqtt_status = message.payload.decode()
            self.status_changed.emit(self.mqtt_status)
        self.client.subscribe(topic)
        self.client.on_message = on_message

    # Funzione per pubblicare
    def publish(self, topic, message):
        self.client.publish(topic, message)
        
    def stop_mqtt(self):
        self.conn_status=False #connection stopped
        print(f"connection status {self.conn_status}")
        self.conn_status_changed.emit(self.conn_status)
        self.client.loop_stop()

    


