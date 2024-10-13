import os
import sys
sys.path.append(os.path.abspath(
    os.path.join(os.path.dirname(__file__), '../../')))

from Globals.imports import *
from Connectors.httpConnector import HTTPClient,RunServer
# Test del client HTTP
class TestHTTPClient(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        # Avvio del server Flask
        cls.flask_thread = threading.Thread(target=RunServer)
        cls.flask_thread.start()
        time.sleep(1)  # Attendere che il server si avvii
        cls.http_client = HTTPClient(url='http://127.0.0.1',port=5000)
        cls.http_client.connect_http()

    def test_http_connect(self):
        # test whether the connection was successful
        self.assertTrue(self.http_client.conn_status)

    def test_http_send_and_receive(self):
        # test sending a message
        test_message = "Hello HTTP"
        self.http_client.send_message(test_message)

        time.sleep(1)

        # test receiving messages
        received = self.http_client.get_received_messages()
        self.assertIn(test_message, received)

    @classmethod
    def tearDownClass(cls):
        # Terminare il server Flask
        # Non c'è un modo diretto per farlo; si può considerare di terminare il thread manualmente
        pass

if __name__ == "__main__":
    unittest.main()
