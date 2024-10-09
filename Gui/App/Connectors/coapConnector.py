""" IGNORE THIS FUNCTION:
THIS FUNCTION IS NOT FINISHED, DO NOT USE IT 
"""

import asyncio
import unittest
from aiocoap import *
from aiocoap.resource import Resource, Site


class CoAPClient:
    def __init__(self, coap_url):
        self.url = coap_url
        self.coap_status = None

    async def connect(self):
        protocol = await Context.create_client_context()
        req = Message(code=GET, uri=self.url)
        try:
            res = await asyncio.wait_for(protocol.request(req).response, timeout=5.0)
            self.coap_status = str(res.code)
            print(f"Response code: {res.code}")
            print(f"Response payload: {res.payload.decode('utf-8')}")
        except asyncio.TimeoutError:
            self.coap_status = 'Error: Timeout'
            print(self.coap_status)
        except Exception as e:
            self.coap_status = 'Error'
            print(f"Error during request: {e}")

    def get_status(self):
        return self.coap_status


class TestCoAPClient(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        # Utilizza il server CoAP pubblico CoAP.me per i test
        cls.coap_url = "coap://coap.me:5683"
        cls.coap_client = CoAPClient(cls.coap_url)
        cls.loop = asyncio.get_event_loop()

    def test_connect(self):
        print("Testing connection to CoAP server...")
        self.loop.run_until_complete(self.coap_client.connect())
        self.assertIsNotNone(self.coap_client.get_status(),
                             "Connection status should not be None")

    def test_get_request(self):
        print("Sending GET request...")
        self.loop.run_until_complete(self.coap_client.connect())
        status = self.coap_client.get_status()
        print(f"GET request status: {status}")
        self.assertEqual(
            status, '2.05', f"Expected status '2.05', but got '{status}'")

    async def send_post_request(self):
        protocol = await Context.create_client_context()
        req = Message(code=POST, uri=self.coap_url, payload=b"Test payload")
        try:
            res = await asyncio.wait_for(protocol.request(req).response, timeout=5.0)
            return res.payload.decode('utf-8')
        except asyncio.TimeoutError:
            return "Error: Timeout"
        except Exception as e:
            return "Error"

    def test_post_request(self):
        print("Sending POST request...")
        response_payload = self.loop.run_until_complete(
            self.send_post_request())
        print(f"Response payload from POST request: {response_payload}")


if __name__ == "__main__":
    unittest.main()
