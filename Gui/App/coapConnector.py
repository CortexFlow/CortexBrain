from aiocoap import *
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
