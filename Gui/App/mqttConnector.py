
        
    
import paho.mqtt.client as mqtt
import random
from PyQt5.QtWidgets import (QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout,
                             QPushButton, QLabel, QFrame, QLineEdit, QFileDialog, QMessageBox,  QPlainTextEdit)

from PyQt5.QtCore import pyqtSignal, QObject
from PyQt5 import QtWidgets, uic


class ConnectionEstablished(QMainWindow):
    def __init__(self):
        super(ConnectionEstablished, self).__init__()
        uic.loadUi("./assets/UI Components/ConnectionEstablished.ui", self)
        
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

    


# Test del connettore MQTT senza mocking
import unittest
import time
class TestMQTTClient(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        # Setup a mqtt connection
        cls.broker = 'test.mosquitto.org'
        cls.port = 1883
        cls.client_id = f'python-mqtt-{random.randint(0, 1000)}'
        # Usa un topic casuale per evitare di ricevere messaggi precedenti
        cls.topic = f"test/topic/{random.randint(0, 1000)}"
        cls.mqtt_client = MQTTClient(cls.broker, cls.port, cls.client_id)
        cls.mqtt_client.connect_mqtt()
        time.sleep(2)

    def test_mqtt_connect(self):
        # test whether the connection was successful
        self.assertEqual(self.mqtt_client.getStatus(), "0")

    def test_mqtt_publish_and_subscribe(self):
        # test subscribe
        test_message = "Hello MQTT"
        self.mqtt_client.subscribe(self.topic)

        time.sleep(1)

        # test publish
        self.mqtt_client.publish(self.topic, test_message)

        time.sleep(1)

        # check if the test message is received
        self.assertEqual(self.mqtt_client.mqtt_status, test_message)

    @classmethod
    def tearDownClass(cls):
        # stop mqtt client
        cls.mqtt_client.stop_mqtt()
        
if __name__ == "__main__":
    unittest.main()