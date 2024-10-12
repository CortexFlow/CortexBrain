import sys
import os
sys.path.append(os.path.abspath(
    os.path.join(os.path.dirname(__file__), '../')))

from PyQt5.QtGui import QIcon, QSyntaxHighlighter, QTextCharFormat, QTextCursor, QPixmap
from PyQt5.QtCore import QTimer, QThreadPool
from PyQt5.QtGui import QIcon, QColor
from PyQt5.QtCore import QRect, QSize, Qt
from PyQt5.QtGui import QPainter, QColor, QTextFormat, QIcon
from PyQt5.QtCore import Qt, QRegularExpression
from PyQt5.QtWidgets import QFileDialog
from matplotlib.figure import Figure
from matplotlib.backends.backend_qt5agg import FigureCanvasQTAgg as FigureCanvas
from PyQt5 import QtWidgets, uic
from PyQt5.QtWidgets import (QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout,
                             QPushButton, QLabel, QFrame, QLineEdit, QTableWidgetItem, QFileDialog, QMessageBox,  QPlainTextEdit, QTextEdit)
from PyQt5.QtCore import QTimer, Qt, QUrl, pyqtSlot, QObject
from PyQt5.QtWebEngineWidgets import QWebEngineView
from PyQt5.QtWebChannel import QWebChannel
from PyQt5.QtWidgets import QComboBox
import traceback
import random
import io
from Connectors.httpConnector import HTTPClient
from mqttConnector import MQTTClient, ConnectionEstablished
import time

import numpy as np


"""         UPDATE: Implement a way to select which protocol you want to stop
        the connection """



# SplashScreen Class
class SplashScreen(QMainWindow):
    def __init__(self):
        super(SplashScreen, self).__init__()
        uic.loadUi("./assets/UI Components/SplashScreen.ui", self)
        self.setWindowTitle('CortexBrain')
        self.setWindowIcon(QIcon("./public/icon.png"))
        self.quit = self.findChild(QPushButton, 'Quit')
        self.status = self.findChild(QLabel, 'status')
        self.setWindowFlag(Qt.FramelessWindowHint)

        # Set the loading timer for 6 seconds
        self.loading_timer = QTimer(self)
        self.loading_timer.timeout.connect(self.finish_loading)
        self.loading_timer.start(6000)  # 6000 ms = 6 seconds

        self.show()

    def finish_loading(self):
        # Stop the timer
        self.loading_timer.stop()
        self.status.setText("Completed!")

        # Start the login window
        self.login_window = Login()
        self.login_window.show()

        # Close the splash screen
        self.close()


# Login Class
class Login(QMainWindow):
    def __init__(self):
        super(Login, self).__init__()
        self.setWindowTitle('CortexBrain')
        self.setWindowIcon(QIcon("icon.png"))
        uic.loadUi("./assets/UI Components/loginWindow2.ui", self)

        # Find the login button and connect it to the login function
        self.loginButton = self.findChild(QPushButton, "login")
        self.loginButton.clicked.connect(self.handle_login)
        self.side_images = self.findChild(QLabel, "side_img")

        self.show()

    def handle_login(self):
        # next-->put login

        # Once login is successful, open the main window
        self.main_app = MainWindow()
        self.main_app.show()

        # Close the login window
        self.close()


class Connectors(QMainWindow):
    def __init__(self, main_window):
        super(Connectors, self).__init__()
        self.setWindowTitle('Connectors')
        self.setWindowIcon(QIcon("icon.png"))
        uic.loadUi("./assets/UI Components/Connectors.ui", self)

        self.main_window = main_window
        self.mqtt_client = None
        self.http_client = None
        self.connection_established = None
        self.stop_connection = None
        self.protocol_selector.currentIndexChanged.connect(self.on_protocol_selected)

        self.btn_connect_mqtt.clicked.connect(self.connectMqtt)
        self.btn_connect_http.clicked.connect(self.connectHttp)

        self.main_window.btn_stopconn.clicked.connect(self.stopServerConnection)
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
            self.plot_widget_layout = QVBoxLayout(self.main_window.sim_1_widget)
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
            numeric_y_data = [float(val) for val in self.y_data if self.is_float(val)]
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
        self.setWindowTitle('Stop Connection')
        self.setWindowIcon(QIcon("../App/public/icon.png"))
        uic.loadUi("./assets/UI Components/StopConnectionPanel.ui", self)
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



# highlights the words
class SyntaxHighlighter(QSyntaxHighlighter):
    def __init__(self, parent=None):
        super(SyntaxHighlighter, self).__init__(parent)

        self.highlightingWords = []  # list for the highlithed words

        # Colore blue
        blue_format = QTextCharFormat()
        blue_format.setForeground(QColor(116, 151, 178))
        # Colore yellow
        yellow_format = QTextCharFormat()
        yellow_format.setForeground(QColor(255, 255, 51))

        # blue keywords
        blue_keywords = ["def", "class", "import", "from", "as", "if", "else", "elif", "return",
                         "while", "for", "in", "break", "continue", "try", "except", "with", "lambda"]
        # yellow keywords
        yellow_keywords = ["\[", "\]", "\(", "\)", "\[\]", "\(\)"]

        for keyword in blue_keywords:
            pattern = QRegularExpression(r'\b' + keyword + r'\b')
            self.highlightingWords.append((pattern, blue_format))

        for keyword in yellow_keywords:
            pattern_y = QRegularExpression(keyword)
            self.highlightingWords.append((pattern_y, yellow_format))

    def highlightBlock(self, text):
        # apply the rules for coloring yellow and blue words
        for pattern, format in self.highlightingWords:
            match_iterator = pattern.globalMatch(text)
            while match_iterator.hasNext():
                match = match_iterator.next()
                self.setFormat(match.capturedStart(),
                               match.capturedLength(), format)


# main window
class MainWindow(QMainWindow):
    def __init__(self):
        super(MainWindow, self).__init__()
        uic.loadUi('./assets/UI Components/AppInterface.ui', self)
        self.setWindowTitle('CortexBrain')
        self.setWindowIcon(QIcon("icon.png"))

        # insert the buttons
        self.btn_settings.clicked.connect(self.open_settings)
        self.customer_support.clicked.connect(self.custom_support)
        self.donate_btn.clicked.connect(self.donate)
        self.go_home_btn.clicked.connect(self.GoHome)
        self.go_sim_btn.clicked.connect(self.GoSim)
        self.go_datas_btn.clicked.connect(self.GoDatas)
        self.go_progetta_btn.clicked.connect(self.GoProgetta)

        # ------------------------------------------------------------

        # Initialize the text editor
        self.text_editor.setText("Benvenuto nel text editor!")

        # inizialize the syntax highlighter
        self.highlighter = SyntaxHighlighter(self.text_editor.document())

        self.btn_new.clicked.connect(self.newFile)
        self.btn_save.clicked.connect(self.saveFile)
        self.btn_open_file.clicked.connect(self.openFile)
        self.btn_new_text.clicked.connect(self.newFile)
        self.btn_copy_text.clicked.connect(self.copy)
        self.btn_paste_text.clicked.connect(self.paste)
        self.btn_undo_text.clicked.connect(self.undo)
        self.btn_redo_text.clicked.connect(self.redo)
        self.btn_compile_code.clicked.connect(self.compile_code)
        self.btn_run_code.clicked.connect(self.run_code)

        # --------------------------------------------------
        # inizialize CONNECTORS
        self.btn_connectors.clicked.connect(self.open_connectors_window)
        self.connectors_window = None
        self.stackedWidget.setCurrentWidget(self.page_home)

    def highlightCurrentLine(self):
        extraSelections = []

        if not self.text_editor.isReadOnly():
            selection = QTextEdit.ExtraSelection()
            lineColor = QColor(Qt.yellow).lighter(160)
            selection.format.setBackground(lineColor)
            selection.format.setProperty(QTextFormat.FullWidthSelection, True)
            selection.cursor = self.text_editor.textCursor()
            selection.cursor.clearSelection()
            extraSelections.append(selection)

        self.text_editor.setExtraSelections(extraSelections)

    def open_settings(self):
        # change page to the settings page
        self.stackedWidget.setCurrentWidget(self.page_settings)

    def GoHome(self):
        # print("Home button clicked")
        self.stackedWidget.setCurrentWidget(
            self.page_home)  # go to home page

    def GoSim(self):
        self.stackedWidget.setCurrentWidget(
            self.page_sim)  # go to the sim page

    def GoDatas(self):
        self.stackedWidget.setCurrentWidget(
            self.page_datas)  # go to the data page

    def GoProgetta(self):
        # go to the project design page
        self.stackedWidget.setCurrentWidget(self.page_progetta)

    def newFile(self):
        pass

    def custom_support(self):
        pass

    def donate(self):
        pass

    # "save"  file function (TEXT EDITOR FEATURE)
    def saveFile(self):
        if self.current_path is not None:
            filetext = self.text_editor.toPlainText()
            with open(self.current_path, 'w') as f:
                f.write(filetext)
        else:
            self.saveFileAs()

    # "save as" file function (TEXT EDITOR FEATURE)
    def saveFileAs(self):
        pathname = QFileDialog.getSaveFileName(
            self, 'Save file', 'D:\codefirst.io\PyQt5 Text Editor', 'Text files(*.txt)')
        filetext = self.text_editor.toPlainText()
        with open(pathname[0], 'w') as f:
            f.write(filetext)
        self.current_path = pathname[0]
        self.setWindowTitle(pathname[0])

    # "open" file function (TEXT EDITOR FEATURE)
    def openFile(self):
        fname = QFileDialog.getOpenFileName(
            self, 'Open file', 'D:\codefirst.io\PyQt5 Text Editor', 'Text files (*.txt)')
        self.setWindowTitle(fname[0])
        with open(fname[0], 'r') as f:
            filetext = f.read()
            self.text_editor.setText(filetext)
        self.current_path = fname[0]

    # "undo" file function (TEXT EDITOR FEATURE)

    def undo(self):
        self.text_editor.undo()

    # "redo" file function (TEXT EDITOR FEATURE)
    def redo(self):
        self.text_editor.redo()

    # "copy" file function (TEXT EDITOR FEATURE)
    def copy(self):
        self.text_editor.copy()

    # "paste" file function (TEXT EDITOR FEATURE)
    def paste(self):
        self.text_editor.paste()

    # "compile" code function (TEXT EDITOR FEATURE)
    def compile_code(self):
        code = self.text_editor.toPlainText()
        self.compiler_.clear()  # clear previous output

        output, error = self.compile_code_internal(code)
        if error:
            self.compiler_.append(error)
        else:
            self.compiler_.append("Compiled with no errors")

    # compile code internal--->return no output only for the "compile function" associated with the compile button
    def compile_code_internal(self, code):
        try:
            compiled_code = compile(code, '<string>', 'exec')
            exec_output = {}
            exec(compiled_code, exec_output)
            return None, None  # Return no output and no errors
        except SyntaxError as e:
            return None, f"Errore di sintassi: {e}"  # error handler
        except Exception as e:
            error_message = traceback.format_exc()
            # error message
            return None, f"Errore di esecuzione:\n{error_message}"

    # Run the code from the text editor and display the result in the output window and the compilation result in the compiler window
    def run_code(self):
        # Retrieve the code entered in the text editor
        code = self.text_editor.toPlainText()

        # Clear the compiler window to reset previous messages
        self.compiler_.clear()

        # Compile the code (assuming self.compile_code handles any compilation or syntax checking)
        self.compile_code()

        # Redirect output to a buffer
        buffer = io.StringIO()  # Create a buffer to capture printed output
        # Store the current stdout (console output)
        original_stdout = sys.stdout
        sys.stdout = buffer  # Redirect stdout to the buffer

        try:
            # Dictionary for local variables in the exec environment
            local_vars = {}

            # Execute the code within a controlled environment
            exec(code, {}, local_vars)

            # Get the output from the buffer
            output = buffer.getvalue()

            # If there's output, append it to the compiler side window
            if output:
                self.compiler_side_window.append(output)

        # Catch any exception that occurs during code execution
        except Exception as e:
            # Get the full error traceback and display it in the compiler window
            error_message = traceback.format_exc()
            self.compiler_.append(f"Errore:\n{error_message}")

        # Ensure stdout is always restored, even if an error occurs
        finally:
            sys.stdout = original_stdout  # Restore the original stdout
            buffer.close()  # Close the buffer to free up memory

    # open the connector window
    def open_connectors_window(self):
        # If the Connectors window is already open, bring it to the foreground
        if self.connectors_window is None or not self.connectors_window.isVisible():
            self.connectors_window = Connectors(
                self)  # Create a new Connectors window
        else:
            self.connectors_window.raise_()  # Bring the already open window to the foreground
            self.connectors_window.activateWindow()  # Activates the window

        print("Connection established")

    def on_close(self, event):
        # set the connect window to none when the connectionEstablished window is closed
        self.connectors_window = None
        event.accept()  # Accept the closing event


def main():
    app = QtWidgets.QApplication(sys.argv)
    # load the splash screen when the program starts
    splash = SplashScreen()

    # starts the event loop of the application, which is necessary for handling user input,
    # updating the interface, and processing events (ie. button clicks, window updates, etc)
    sys.exit(app.exec_())


if __name__ == "__main__":
    main()
