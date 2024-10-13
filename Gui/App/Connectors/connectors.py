import sys
import os
sys.path.append(os.path.abspath(
    os.path.join(os.path.dirname(__file__), '../')))
from Globals.imports import *
import numpy as np
from mqttConnector import MQTTClient, ConnectionEstablished
from Connectors.httpConnector import HTTPClient
import random
from Globals.constants import GLOBAL_VAR



class Connectors(QMainWindow):
    def __init__(self, main_window):
        super(Connectors, self).__init__()
        self.setWindowTitle(GLOBAL_VAR.CONNECTORS_SCREEN_TITLE)
        self.setWindowIcon(QIcon(GLOBAL_VAR.TITLE))
        uic.loadUi(GLOBAL_VAR.CONNECTORS_SCREEN_UI, self)

        self.main_window = main_window
        self.mqtt_client = None
        self.http_client = None
        self.connection_established = None
        self.stop_connection = None
        self.protocol_selector.currentIndexChanged.connect(
            self.on_protocol_selected)

        self.btn_connect_mqtt.clicked.connect(self.connectMqtt)
        self.btn_connect_http.clicked.connect(self.connectHttp)

        self.main_window.btn_stopconn.clicked.connect(
            self.stopServerConnection)
        self.show()

        self.x_data = np.arange(0, 10, 0.1)
        self.y_data = np.zeros_like(self.x_data)
        self.figure = None
        self.canvas = None
        self.ax = None
        self.line = None

        self.timer = QTimer()
        self.timer.timeout.connect(self.update_plot)

    def goMQTT_protocol(self):
        self.stackedWidget.setCurrentWidget(self.page_mqtt)

    def goHTTP_protocol(self):
        self.stackedWidget.setCurrentWidget(self.page_http)

    def goCoAP_protocol(self):
        self.stackedWidget.setCurrentWidget(self.page_coap)

    def on_protocol_selected(self):
        sel_opt = self.protocol_selector.currentText()
        if sel_opt == "MQTT":
            self.goMQTT_protocol()
        elif sel_opt == "HTTP":
            self.goHTTP_protocol()
        elif sel_opt == "COAP":
            self.goCoAP_protocol()

    def connectMqtt(self):
        client_id = f'python-mqtt-{random.randint(0, 1000)}'
        self.mqtt_client = MQTTClient(self.broker_text_mqtt.toPlainText(), int(
            self.port_text_mqtt.toPlainText()), client_id)

        self.mqtt_client.status_changed.connect(self.updateStatus)
        self.mqtt_client.connect_mqtt()
        self.mqtt_client.subscribe(self.topic_text_mqtt.toPlainText())
        self.connection_established = ConnectionEstablished()
        self.handle_server_status()

        self.loading_timer = QTimer(self)
        self.loading_timer.start(2000)
        self.loading_timer.timeout.connect(self.on_timeout_mqtt)

    def connectHttp(self):
        self.http_client = HTTPClient(url=self.broker_text_http.toPlainText(
        ), port=int(self.port_text_http.toPlainText()))

        self.http_client.status_changed.connect(self.updateStatus)
        self.http_client.connect_http()

        if self.http_client.conn_status:
            self.connection_established = ConnectionEstablished()
            self.connection_established.show()
            QApplication.processEvents()

            self.loading_timer = QTimer(self)
            self.loading_timer.setSingleShot(True)
            self.loading_timer.timeout.connect(self.on_timeout)
            QThreadPool.globalInstance().start(self.receive_messages)
            self.loading_timer.start(2000)
        else:
            print("HTTP connection failed.")

    def handle_server_status(self):
        if self.mqtt_client.conn_status is False:
            self.pixmap = QPixmap('./play-blue.png')
        elif self.mqtt_client.conn_status is True:
            self.pixmap = QPixmap('./stop-red.png')
        else:
            self.pixmap = QPixmap('./stop-red.png')

        self.main_window.server_status_icon.setPixmap(self.pixmap)

    def handle_http_server_status(self):
        if self.http_client.conn_status is False:
            self.pixmap = QPixmap('./play-blue.png')
        elif self.http_client.conn_status is True:
            self.pixmap = QPixmap('./stop-red.png')
        else:
            self.pixmap = QPixmap('./stop-red.png')

        self.main_window.server_status_icon.setPixmap(self.pixmap)

    def receive_messages(self):
        try:
            self.http_client.get_received_messages(
                endpoint=self.topic_text_http.toPlainText())
        except Exception as e:
            print(f"Error receiving messages: {e}")

    def on_timeout_mqtt(self):
        self.connection_established.close()

    def on_timeout(self):
        if hasattr(self, 'connection_established'):
            self.connection_established.close()
        else:
            print("Connection window not initialized.")

    def updateStatus(self, status):
        print("Response: ", status)
        self.main_window.compiler_.append(status)
        self.add_to_table(status)

        if self.figure is None:
            self.create_plot()
        self.y_data = np.append(self.y_data[1:], status)
        self.update_plot()

    def create_plot(self):
        self.figure = Figure()
        self.canvas = FigureCanvas(self.figure)
        self.plot_widget_layout = self.main_window.sim_1_widget.layout()

        if self.plot_widget_layout is None:
            self.plot_widget_layout = QVBoxLayout(
                self.main_window.sim_1_widget)
            self.main_window.sim_1_widget.setLayout(self.plot_widget_layout)

        for i in reversed(range(self.plot_widget_layout.count())):
            widget = self.plot_widget_layout.itemAt(i).widget()
            if widget is not None:
                widget.deleteLater()

        self.plot_widget_layout.addWidget(self.canvas)
        self.ax = self.figure.add_subplot(111)
        self.line, = self.ax.plot(self.x_data, self.y_data)
        self.ax.set_ylim(0, 1000)
        self.ax.set_title('Status over Time')
        self.ax.set_xlabel('Time')
        self.ax.set_ylabel('Status')
        self.timer.start(1000)

    def add_to_table(self, status):
        self.main_window.data_table.setColumnCount(1)
        row_count = self.main_window.data_table.rowCount()
        self.main_window.data_table.insertRow(row_count)
        self.main_window.data_table.setItem(
            row_count, 0, QTableWidgetItem(str(status)))

    def update_plot(self):
        try:
            numeric_y_data = [float(val)
                              for val in self.y_data if self.is_float(val)]
        except ValueError:
            print("Errore nella conversione dei dati in float")

        if numeric_y_data:
            self.line.set_ydata(numeric_y_data)
            self.canvas.draw()

    def is_float(self, value):
        try:
            float(value)
            return True
        except ValueError:
            return False

    def stopServerConnection(self):
        self.stop_connection = StopConnections(self)
        self.loading_timer = QTimer(self)
        self.loading_timer.start(2000)
        self.loading_timer.timeout.connect(self.on_timeout_stop_connection)

    def on_timeout_stop_connection(self):
        self.stop_connection.close()


class StopConnections(QMainWindow):
    def __init__(self, main_window):
        super(StopConnections, self).__init__()
        self.setWindowTitle(GLOBAL_VAR.STOP_CONNECTION_SCREEN_TITLE)
        self.setWindowIcon(QIcon(GLOBAL_VAR.ICON))
        uic.loadUi(GLOBAL_VAR.STOP_CONNECTION_SCREEN_UI, self)
        self.main_window = main_window

        # Recupero degli attributi mqtt_client e http_client dal main_window
        self.mqtt_client = main_window.mqtt_client
        self.http_client = main_window.http_client

        self.stop_mqtt_checkbox.stateChanged.connect(self.StopMqttConnHandler)
        self.stop_http_checkbox.stateChanged.connect(self.StopHttpConnHandler)

        self.show()

    def StopMqttConnHandler(self, state):
        if state == 2 and self.mqtt_client is not None:
            self.mqtt_client.stop_mqtt()

    def StopHttpConnHandler(self, state):
        if state == 2 and self.http_client is not None:
            self.http_client.stop_http()
