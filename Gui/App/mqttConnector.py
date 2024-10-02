
        
    
import paho.mqtt.client as mqtt
import random
from PyQt5.QtWidgets import (QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout,
                             QPushButton, QLabel, QFrame, QLineEdit, QFileDialog, QMessageBox,  QPlainTextEdit)

from PyQt5.QtCore import pyqtSignal, QObject
from PyQt5 import QtWidgets, uic

import asyncio
from aiocoap import *

class ConnectionEstablished(QMainWindow):
    def __init__(self):
        super(ConnectionEstablished, self).__init__()
        uic.loadUi("./ConnectionEstablished.ui", self)
        
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

class CoAP():
    # Signal to emit when the connection status changes
    #status_changed = pyqtSignal(str)

    def __init__(self, coap_url):
        super(CoAP, self).__init__()  # Properly call the super class __init__
        self.url = coap_url
        self.coap_status = None
    
    async def connect_coap(self):
        protocol= await Context.create_client_context()
        req = Message(code=GET,uri=self.url)
        try:
            res=await protocol.request(req).response
            print(f"Response code : {res.code}")
            print(f"Response payload : {res.payload.decode('utf-8')}")
            self.coap_status = str(res.code)
        except Exception as e:
            print(f"Error: {e}")
            self.coap_status = 'Error'
            
        #self.status_changed.emit(self.coap_status) #connect to the signal

        
    def getStatus(self):
        return self.coap_status
    


# Esempio principale
if __name__ == "__main__":
    """     
    broker = 'test.mosquitto.org'
    port = 1883
    topic = "python/mqtt"
    client_id = f'python-mqtt-{random.randint(0, 1000)}'

    mqtt_client = MQTTClient(broker, port, client_id)
    client = mqtt_client.connect_mqtt()

    client.loop_start()
    mqtt_client.subscribe(topic)
    mqtt_client.publish(topic, "Messaggio con MQTT 2.1.0")
    """
    
    coapp_client = CoAP(coap_url="coap://coap.me/test")
    
    asyncio.run(coapp_client.connect_coap())
    #import time
    #time.sleep(15)

    #client.loop_stop()
