import sys
import os
import numpy as np

sys.path.append(os.path.abspath(
    os.path.join(os.path.dirname(__file__), '../')))
import time
from mqttConnector import MQTTClient, ConnectionEstablished
from Connectors.httpConnector import HTTPClient
import io
import random
import traceback
from PyQt5.QtWidgets import QComboBox
from PyQt5.QtWebChannel import QWebChannel
from PyQt5.QtWebEngineWidgets import QWebEngineView
from PyQt5.QtCore import QTimer, Qt, QUrl, pyqtSlot, QObject
from PyQt5.QtGui import QIcon,QColor
from PyQt5.QtWidgets import (QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout,
                             QPushButton, QLabel, QFrame, QLineEdit, QTableWidgetItem,QFileDialog, QMessageBox,  QPlainTextEdit,QTextEdit)
from PyQt5 import QtWidgets, uic
from PyQt5.QtCore import QTimer,QThreadPool
from matplotlib.backends.backend_qt5agg import FigureCanvasQTAgg as FigureCanvas
from matplotlib.figure import Figure
from PyQt5.QtWidgets import QFileDialog
from PyQt5.QtGui import QIcon, QSyntaxHighlighter, QTextCharFormat, QTextCursor,QPixmap
from PyQt5.QtCore import Qt, QRegularExpression
from PyQt5.QtGui import QPainter, QColor, QTextFormat, QIcon
from PyQt5.QtCore import QRect, QSize, Qt



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

        # main_window--> reference the main window application
        self.main_window = main_window
        self.connection_established = None 

        #self.btn_connect.clicked.connect(self.connectMqtt) #click--->connect to mqtt  
        self.btn_connect.clicked.connect(self.connectHttp)
        
        
        self.main_window.btn_stopconn.clicked.connect(self.stopServerConnection) #stop server connection
        self.show()

        """         
        NEED FIX: UPGRADE THIS PART TO HANDLE MORE COMPLEX CASES
        
        # plotting logic-->inizialize xdata range and an array of 0 elements for the y axis
        self.x_data = np.arange(0, 10, 0.1)
        self.y_data = np.zeros_like(self.x_data) 
        self.figure = None
        self.canvas = None
        self.ax = None
        self.line = None  #inizialize the dynamic line

        # inizialize the timer. the timer is connect to the auto chart update
        self.timer = QTimer()
        self.timer.timeout.connect(self.update_plot) 
        """
        
        
        #handles server status and display the status icon 
    def handle_server_status(self):
        if self.mqtt_client.conn_status is False:
            self.pixmap = QPixmap('./play-blue.png')
        elif self.mqtt_client.conn_status is True:
            self.pixmap = QPixmap('./stop-red.png')
        else:
            self.pixmap = QPixmap('./stop-red.png')
        
        self.main_window.server_status_icon.setPixmap(self.pixmap) #assign status icon
        
        
        
    def handle_http_server_status(self):
        """ 
        WORKING ON THIS INTEGRATON
        FEATURE: HTTP CLIENT STATUS CHECKER
        USAGE: CHECK THE HTTP CONNECTION STATUS AND CHANGE THE SERVER STATUS ICON
        """
        if self.http_client.conn_status is False:
            self.pixmap = QPixmap('./play-blue.png')
        elif self.http_client.conn_status is True:
            self.pixmap = QPixmap('./stop-red.png')
        else:
            self.pixmap = QPixmap('./stop-red.png')
        
        self.main_window.server_status_icon.setPixmap(self.pixmap) #assign status icon
        

    def connectMqtt(self):
        client_id = f'python-mqtt-{random.randint(0, 1000)}'
        self.mqtt_client = MQTTClient(self.broker_text.toPlainText(), int(
            self.port_text.toPlainText()), client_id)

        # Connetti il segnale status_changed al metodo updateStatus
        self.mqtt_client.status_changed.connect(self.updateStatus)

        self.mqtt_client.connect_mqtt()
        self.mqtt_client.subscribe(self.topic_text.toPlainText()) #subscribes to the topic
        self.connection_established = ConnectionEstablished() #inizialize the connectionEstablished window
        #call the handle server status icon 
        self.handle_server_status()
        
        # adds a timer to close the window
        self.loading_timer = QTimer(self)
        self.loading_timer.start(2000)
        self.loading_timer.timeout.connect(self.on_timeout)  # connect the timeout signal to the on_timout function 

    def connectHttp(self):
        """ WORKING ON THIS INTEGRATION 
            FEATURE: HTTP CONNECTOR
            USAGE: CONNECTS TO A HTTP SERVER
        """
        # Inizializza l'oggetto HTTPClient con l'URL e la porta specificati
        self.http_client = HTTPClient(url=self.broker_text.toPlainText(), port=int(self.port_text.toPlainText()))
        
        # Connetti il segnale per lo stato della connessione
        self.http_client.status_changed.connect(self.updateStatus)
        
        # Esegui la connessione HTTP
        self.http_client.connect_http()

        # Gestisci lo stato della connessione direttamente qui
        print(f"HTTP CONNECTION STATUS: {self.http_client.conn_status}")
        
        if self.http_client.conn_status:  # Se la connessione è stata stabilita
            # Inizializza la finestra della connessione stabilita
            self.connection_established = ConnectionEstablished()  
            
            # Mostra esplicitamente la finestra
            self.connection_established.show()

            # Forza l'aggiornamento della UI per evitare che rimanga bloccata
            QApplication.processEvents()

            # Inizia a ricevere i messaggi in un thread separato
            self.loading_timer = QTimer(self)
            self.loading_timer.setSingleShot(True)  # Assicurati che il timer scatti una sola volta
            self.loading_timer.timeout.connect(self.on_timeout)  # Collega il timeout alla funzione on_timeout
            
            # Inizia a ricevere i messaggi in background
            QThreadPool.globalInstance().start(self.receive_messages)  # Utilizza QThreadPool per gestire le operazioni in background

            # Avvia il timer per chiudere la finestra dopo 2 secondi
            self.loading_timer.start(2000)  # Imposta il timer per 2 secondi

        else:
            print("HTTP connection failed.")

    def receive_messages(self):
        """ Funzione per ricevere messaggi in background. """
        try:
            self.http_client.get_received_messages(endpoint=self.topic_text.toPlainText())
        except Exception as e:
            print(f"Error receiving messages: {e}")



    
            
    #on_timeout function--->automatically close the connectionEstablished window after 2 seconds
    def on_timeout(self):
        """Chiamata quando il timer scade."""
        # Verifica se 'connection_established' è stato creato
        if hasattr(self, 'connection_established'):
            print("Closing connection window.")
            self.connection_established.close()
        else:
            print("Connection window not initialized.")

    def updateStatus(self, status):
        #status stores the upcoming data 
        print("Response: ", status)
        self.main_window.compiler_.append(status) #insert the status

        """ 
        WORKING ON THIS INTEGRATION
        
        NEED FIX: UPGRADE THIS PART TO HANDLE MORE COMPLEX CASES
        
        # Add the data in the table 
        self.add_to_table(status)

        # update the y axis with the new status data
        if self.figure is None:  
            self.create_plot()  
        self.y_data = np.append(self.y_data[1:], status)  # append the upcoming status 
        self.update_plot()  # update the plot 
        """
    def create_plot(self):
        # inizialize a new matplotlib figure and a canvas
        self.figure = Figure()
        self.canvas = FigureCanvas(self.figure)

        
        self.plot_widget_layout = self.main_window.sim_1_widget.layout() #store the plot widget in the plot_widget_layout variable

        #handles some exceptions
        #if the widget doesn't exist create a new QVBoxLayout
        if self.plot_widget_layout is None:
            print("Layout non impostato, creazione di un QVBoxLayout...")
            self.plot_widget_layout = QVBoxLayout(self.main_window.sim_1_widget)
            self.main_window.sim_1_widget.setLayout(self.plot_widget_layout)

        # removes widgets unecessary elements in the plot layout (rare event)
        for i in reversed(range(self.plot_widget_layout.count())): 
            widget = self.plot_widget_layout.itemAt(i).widget()
            if widget is not None:
                widget.deleteLater()  

        # add the new canvas to the layout
        self.plot_widget_layout.addWidget(self.canvas)

        # draw the plot
        self.ax = self.figure.add_subplot(111)
        self.line, = self.ax.plot(self.x_data, self.y_data)
        self.ax.set_ylim(0, 1000)
        self.ax.set_title('Status over Time')  #title
        self.ax.set_xlabel('Time')  # x axis title
        self.ax.set_ylabel('Status')  # y axis title

        # this timer controls the auto update of the plot
        self.timer.start(1000)
        
    #add datas to the table in the simulation environment
    def add_to_table(self, status):
        print(f"Aggiungendo status alla tabella: {status}")
        self.main_window.data_table.setColumnCount(1)  #set at least 1 colum

        # return the current row count
        row_count = self.main_window.data_table.rowCount()
        
        # insert a new row
        self.main_window.data_table.insertRow(row_count)
        
        # Insert the status data in the current row, line 0
        self.main_window.data_table.setItem(row_count, 0, QTableWidgetItem(str(status))) 



    def update_plot(self):
        
        try:
            # convert the data in float and ignores the string values
            numeric_y_data = [float(val) for val in self.y_data if self.is_float(val)]
        except ValueError:
            print("Errore nella conversione dei dati in float")

        # update the data values only if there are numeric values
        if numeric_y_data:
            self.line.set_ydata(numeric_y_data)
            self.canvas.draw()  # draw the updated canvas

    def is_float(self, value):
        # check if the is a float type
        try:
            float(value)
            return True
        except ValueError:
            return False
        
    
    def stopServerConnection(self):
        """ 
        WORKING ON THIS INTEGRATION:
        FEATURE: STOP SERVER CONNECTION. 
        
        """
        self.main_window.compiler_.append("Stopping server connetion..")
        # close the server connection
        #self.mqtt_client.stop_mqtt()
        self.http_client.stop_http()
        #handle server status--->display the connection status icon
        #self.handle_server_status()
        self.handle_http_server_status()



#highlights the words
class SyntaxHighlighter(QSyntaxHighlighter):
    def __init__(self, parent=None):
        super(SyntaxHighlighter, self).__init__(parent)

        
        self.highlightingWords = [] #list for the highlithed words


        # Colore blue
        blue_format = QTextCharFormat()
        blue_format.setForeground(QColor(116,151,178))
        # Colore yellow
        yellow_format = QTextCharFormat()
        yellow_format.setForeground(QColor(255,255,51))

        #blue keywords
        blue_keywords = ["def", "class", "import", "from", "as", "if", "else", "elif", "return", "while", "for", "in", "break", "continue", "try", "except", "with", "lambda"]
        #yellow keywords
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
                self.setFormat(match.capturedStart(), match.capturedLength(), format)




#main window
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
        #print("Home button clicked")
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

    #"save"  file function (TEXT EDITOR FEATURE)
    def saveFile(self):
        if self.current_path is not None:
            filetext = self.text_editor.toPlainText()
            with open(self.current_path, 'w') as f:
                f.write(filetext)
        else:
            self.saveFileAs()

    #"save as" file function (TEXT EDITOR FEATURE)
    def saveFileAs(self):
        pathname = QFileDialog.getSaveFileName(
            self, 'Save file', 'D:\codefirst.io\PyQt5 Text Editor', 'Text files(*.txt)')
        filetext = self.text_editor.toPlainText()
        with open(pathname[0], 'w') as f:
            f.write(filetext)
        self.current_path = pathname[0]
        self.setWindowTitle(pathname[0])

    #"open" file function (TEXT EDITOR FEATURE)
    def openFile(self):
        fname = QFileDialog.getOpenFileName(
            self, 'Open file', 'D:\codefirst.io\PyQt5 Text Editor', 'Text files (*.txt)')
        self.setWindowTitle(fname[0])
        with open(fname[0], 'r') as f:
            filetext = f.read()
            self.text_editor.setText(filetext)
        self.current_path = fname[0]
        
        
    #"undo" file function (TEXT EDITOR FEATURE)
    def undo(self):
        self.text_editor.undo()
    
    #"redo" file function (TEXT EDITOR FEATURE)
    def redo(self):
        self.text_editor.redo()
    
    #"copy" file function (TEXT EDITOR FEATURE)
    def copy(self):
        self.text_editor.copy()
   
    #"paste" file function (TEXT EDITOR FEATURE)
    def paste(self):
        self.text_editor.paste()

    #"compile" code function (TEXT EDITOR FEATURE)
    def compile_code(self):
        code = self.text_editor.toPlainText()
        self.compiler_.clear()  # clear previous output

        output, error = self.compile_code_internal(code)
        if error:
            self.compiler_.append(error)
        else:
            self.compiler_.append("Compiled with no errors")

    #compile code internal--->return no output only for the "compile function" associated with the compile button
    def compile_code_internal(self, code):
        try:
            compiled_code = compile(code, '<string>', 'exec')
            exec_output = {}
            exec(compiled_code, exec_output)
            return None, None  # Return no output and no errors
        except SyntaxError as e:
            return None, f"Errore di sintassi: {e}" #error handler
        except Exception as e:
            error_message = traceback.format_exc()
            return None, f"Errore di esecuzione:\n{error_message}" #error message

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
        original_stdout = sys.stdout  # Store the current stdout (console output)
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

    #open the connector window
    def open_connectors_window(self):
            # If the Connectors window is already open, bring it to the foreground
            if self.connectors_window is None or not self.connectors_window.isVisible():
                self.connectors_window = Connectors(self)  # Create a new Connectors window
            else:
                self.connectors_window.raise_()  # Bring the already open window to the foreground
                self.connectors_window.activateWindow()  # Activates the window

            print("Connection established")
    
    def on_close(self, event):
        self.connectors_window = None  # set the connect window to none when the connectionEstablished window is closed
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
