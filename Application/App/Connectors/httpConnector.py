from Globals.imports import *

app = Flask(__name__)

# Variabile globale per memorizzare i messaggi ricevuti
received_messages = []

@app.route('/send', methods=['POST'])
def send_message():
    message = request.json.get('message')
    if message:
        received_messages.append(message)
        return jsonify({"status": "success", "message": f"Received: {message}"}), 200
    return jsonify({"status": "error", "message": "No message provided"}), 400

@app.route('/messages', methods=['GET'])
def get_messages():
    return jsonify({"messages": received_messages}), 200

class HTTPClient(QObject):
    status_changed = pyqtSignal(str)
    conn_status_changed = pyqtSignal(bool)

    def __init__(self,url,port):
        super(HTTPClient, self).__init__()
        self.port=str(port)
        self.url=url
        self.server_url = f"{self.url}:{self.port}"  # URL del server Flask
        self.conn_status = None
        self.http_status = None

    def connect_http(self):
        try:
            print(f"Attempting to connect to {self.server_url}")
            response = requests.get(f"{self.server_url}")  # Fai una richiesta al server
            if response.status_code == 200:
                self.conn_status = True
                self.http_status = "Connected to HTTP server"
                print(f"Connection successful to {self.server_url}")
            else:
                self.conn_status = False
                self.http_status = f"Failed to connect, status code {response.status_code}"
                print(f"Failed to connect: status code {response.status_code}")
        except Exception as e:
            self.conn_status = False
            self.http_status = f"Connection failed: {e}"
            print(f"Error during connection: {e}")
        self.status_changed.emit(self.http_status)
        self.conn_status_changed.emit(self.conn_status)


    def getStatus(self):
        return self.http_status

    def send_message(self, message):
        response = requests.post(f"{self.server_url}/send", json={"message": message})
        if response.status_code == 200:
            self.http_status = response.json()["message"]
        else:
            self.http_status = "Failed to send message"
        self.status_changed.emit(self.http_status)

    def get_received_messages(self, endpoint):
        """
        Effettua una connessione HTTP continua con il sensore per ricevere e scannerizzare i dati in arrivo.
        Usa una connessione persistente e processa i dati del sensore.
        
        :param endpoint: L'endpoint da cui ricevere i dati del sensore.
        :param retries: Numero massimo di tentativi di riconnessione.
        :param delay: Tempo di attesa tra i tentativi in caso di errore.
        :return: Dati JSON o testo in tempo reale o lista vuota in caso di errori.
        """
        session = requests.Session()  # Creiamo una sessione persistente
        session.headers.update({'Connection': 'keep-alive'})  # Manteniamo la connessione attiva
        while self.conn_status == True:
            try:
                # Effettuiamo la richiesta get con un timeout per evitare blocchi indefiniti
                response = session.get(f"{self.server_url}{endpoint}", stream=True,timeout=5) 
                # Se la risposta è 200 (OK), continuiamo a processare i dati in arrivo
                if response.status_code == 200:
                    print(response.text) #upgrade this with a json response
            except requests.exceptions.RequestException as e:
                print(f"Errore di connessione: {e}")
                return []
            time.sleep(2)
    
    def stop_http(self):
        self.conn_status = False
        print(f"connection status {self.conn_status}")
        self.conn_status_changed.emit(self.conn_status)


# Avvia il server Flask in un thread separato
def RunServer():
    app.run(port=5000)
    

