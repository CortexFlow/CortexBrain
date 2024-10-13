# Test del connettore MQTT senza mocking
import os
import sys
sys.path.append(os.path.abspath(
    os.path.join(os.path.dirname(__file__), '../')))
import unittest
import time
import random
from Connectors.mqttConnector import MQTTClient
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