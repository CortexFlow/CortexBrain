import sys
import os
import numpy as np

sys.path.append(os.path.abspath(
    os.path.join(os.path.dirname(__file__), '../')))
import time
from mqttConnector import MQTTClient, ConnectionEstablished
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
from PyQt5.QtCore import QTimer
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
        uic.loadUi("./SplashScreen.ui", self)
        self.setWindowTitle('CortexBrain')
        self.setWindowIcon(QIcon("icon.png"))
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
        uic.loadUi("./loginWindow2.ui", self)

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
        uic.loadUi("./Connectors.ui", self)

        # Salva un riferimento alla finestra principale
        self.main_window = main_window

        self.btn_connect.clicked.connect(self.connectMqtt)

        
        
        self.main_window.btn_stopconn.clicked.connect(self.stopServerConnection)
        self.show()

        # Inizializza i dati per il grafico
        self.x_data = np.arange(0, 10, 0.1)
        self.y_data = np.zeros_like(self.x_data)  # Inizializza i dati y con zeri
        self.figure = None
        self.canvas = None
        self.ax = None
        self.line = None  # Inizializza 'line' qui

        # Inizializza il timer per l'aggiornamento del grafico
        self.timer = QTimer()
        self.timer.timeout.connect(self.update_plot)
        
    def handle_server_status(self):
        if self.mqtt_client.conn_status is False:
            self.pixmap = QPixmap('./play-blue.png')
        elif self.mqtt_client.conn_status is True:
            self.pixmap = QPixmap('./stop-red.png')
        else:
            self.pixmap = QPixmap('./stop-red.png')
        
        self.main_window.server_status_icon.setPixmap(self.pixmap)
        

    def connectMqtt(self):
        client_id = f'python-mqtt-{random.randint(0, 1000)}'
        self.mqtt_client = MQTTClient(self.broker_text.toPlainText(), int(
            self.port_text.toPlainText()), client_id)

        # Connetti il segnale status_changed al metodo updateStatus
        self.mqtt_client.status_changed.connect(self.updateStatus)

        self.mqtt_client.connect_mqtt()
        self.mqtt_client.subscribe(self.topic_text.toPlainText())
        self.connection_established = ConnectionEstablished()
        #regola per l'icona di stato del server:
        self.handle_server_status()
        
        # adds a timer to close the window
        self.loading_timer = QTimer(self)
        self.loading_timer.start(2000)
        self.loading_timer.timeout.connect(self.on_timeout)  # Collega il segnale timeout

    def on_timeout(self):
        self.connection_established.close()

    def updateStatus(self, status):
        print("Response: ", status)
        self.main_window.compiler_.append(status)

        # Aggiungi il dato nella tabella
        self.add_to_table(status)

        # Aggiorna y_data con il nuovo stato
        if self.figure is None:  # Se il grafico non è ancora stato creato
            self.create_plot()  # Crea il grafico
        self.y_data = np.append(self.y_data[1:], status)  # Aggiorna i dati y (inserendo il nuovo stato)
        self.update_plot()  # Chiama il metodo per aggiornare il grafico

    def create_plot(self):
        # Crea una figura Matplotlib e un canvas
        self.figure = Figure()
        self.canvas = FigureCanvas(self.figure)

        # Ottieni il layout del QWidget
        self.plot_widget_layout = self.main_window.sim_1_widget.layout()

        # Se non esiste, crea un QVBoxLayout e impostalo nel QWidget
        if self.plot_widget_layout is None:
            print("Layout non impostato, creazione di un QVBoxLayout...")
            self.plot_widget_layout = QVBoxLayout(self.main_window.sim_1_widget)
            self.main_window.sim_1_widget.setLayout(self.plot_widget_layout)

        # Rimuovi eventuali widget esistenti dal layout
        for i in reversed(range(self.plot_widget_layout.count())): 
            widget = self.plot_widget_layout.itemAt(i).widget()
            if widget is not None:
                widget.deleteLater()  # Rimuovi il widget esistente

        # Aggiungi il nuovo canvas al layout
        self.plot_widget_layout.addWidget(self.canvas)

        # Disegna il grafico iniziale
        self.ax = self.figure.add_subplot(111)
        self.line, = self.ax.plot(self.x_data, self.y_data)
        self.ax.set_ylim(0, 1000)
        self.ax.set_title('Status over Time')  # Aggiungi un titolo al grafico
        self.ax.set_xlabel('Time')  # Etichetta asse x
        self.ax.set_ylabel('Status')  # Etichetta asse y

        # Avvia il timer per aggiornare il grafico
        self.timer.start(1000)
        
        
    def add_to_table(self, status):
        print(f"Aggiungendo status alla tabella: {status}")
        self.main_window.data_table.setColumnCount(1)  # Imposta almeno 1 colonna

        # Ottieni il numero di righe attuali
        row_count = self.main_window.data_table.rowCount()
        
        # Aggiungi una nuova riga
        self.main_window.data_table.insertRow(row_count)
        
        # Inserisci i dati di status nella prima colonna della nuova riga
        self.main_window.data_table.setItem(row_count, 0, QTableWidgetItem(str(status))) 



    def update_plot(self):
        # Filtro dei dati numerici
        try:
            # Converte i dati in float e ignora eventuali stringhe
            numeric_y_data = [float(val) for val in self.y_data if self.is_float(val)]
        except ValueError:
            print("Errore nella conversione dei dati in float")

        # Aggiorna i dati del grafico solo se esistono valori numerici
        if numeric_y_data:
            self.line.set_ydata(numeric_y_data)
            self.canvas.draw()  # Ridisegna il canvas

    def is_float(self, value):
        # Funzione di utilità per verificare se un valore può essere convertito in float
        try:
            float(value)
            return True
        except ValueError:
            return False
        
    
    def stopServerConnection(self):
        self.main_window.compiler_.append("Stopping server connetion..")
        # close the server connection
        self.mqtt_client.stop_mqtt()
        #regola per l'icona di stato del server:
        self.handle_server_status() #bug: non viene aggiornata l'icona: SOL-->COLLEGARE A SEGNALE
        




class SyntaxHighlighter(QSyntaxHighlighter):
    def __init__(self, parent=None):
        super(SyntaxHighlighter, self).__init__(parent)

        # Definisci il formato per le parole chiave
        self.highlightingRules = []


        # Colore blue
        blue_format = QTextCharFormat()
        blue_format.setForeground(QColor(116,151,178))
        # Colore blue
        yellow_format = QTextCharFormat()
        yellow_format.setForeground(QColor(255,255,51))

        # Aggiungi parole chiave da evidenziare
        blue_keywords = ["def", "class", "import", "from", "as", "if", "else", "elif", "return", "while", "for", "in", "break", "continue", "try", "except", "with", "lambda"]
        yellow_keywords = ["\[", "\]", "\(", "\)", "\[\]", "\(\)"]
        
        for keyword in blue_keywords:
            pattern = QRegularExpression(r'\b' + keyword + r'\b')
            self.highlightingRules.append((pattern, blue_format))
            
        for keyword in yellow_keywords:
            pattern_y = QRegularExpression(keyword)
            self.highlightingRules.append((pattern_y, yellow_format))
                
                
    def highlightBlock(self, text):
        # Applica tutte le regole di evidenziazione
        for pattern, format in self.highlightingRules:
            match_iterator = pattern.globalMatch(text)
            while match_iterator.hasNext():
                match = match_iterator.next()
                self.setFormat(match.capturedStart(), match.capturedLength(), format)





class MainWindow(QMainWindow):
    def __init__(self):
        super(MainWindow, self).__init__()
        uic.loadUi('./AppInterface.ui', self)
        self.setWindowTitle('CortexBrain')
        self.setWindowIcon(QIcon("icon.png"))

        # Collega i pulsanti agli eventi
        self.btn_settings.clicked.connect(self.open_settings)
        self.customer_support.clicked.connect(self.custom_support)
        self.donate_btn.clicked.connect(self.donate)
        self.go_home_btn.clicked.connect(self.GoHome)
        self.go_sim_btn.clicked.connect(self.GoSim)
        self.go_datas_btn.clicked.connect(self.GoDatas)
        self.go_progetta_btn.clicked.connect(self.GoProgetta)

        # ------------------------------------------------------------

        # Inizializza l'editor di testo
        self.text_editor.setText("Benvenuto nel text editor!")

        # Inizializza il syntax highlighter
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
        # CONNECTORS
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
        # Cambia alla pagina delle impostazioni
        self.stackedWidget.setCurrentWidget(self.page_settings)

    def custom_support(self):
        return None

    def donate(self):
        return None

    def GoHome(self):
        print("Home button clicked")
        self.stackedWidget.setCurrentWidget(
            self.page_home)  # Cambia alla pagina home

    def GoSim(self):
        self.stackedWidget.setCurrentWidget(
            self.page_sim)  # Cambia alla pagina simulazione

    def GoDatas(self):
        self.stackedWidget.setCurrentWidget(
            self.page_datas)  # Cambia alla pagina dati

    def GoProgetta(self):
        # Cambia alla pagina progettazione
        self.stackedWidget.setCurrentWidget(self.page_progetta)

    def newFile(self):
        pass

    def saveFile(self):
        if self.current_path is not None:
            filetext = self.text_editor.toPlainText()
            with open(self.current_path, 'w') as f:
                f.write(filetext)
        else:
            self.saveFileAs()

    def saveFileAs(self):
        pathname = QFileDialog.getSaveFileName(
            self, 'Save file', 'D:\codefirst.io\PyQt5 Text Editor', 'Text files(*.txt)')
        filetext = self.text_editor.toPlainText()
        with open(pathname[0], 'w') as f:
            f.write(filetext)
        self.current_path = pathname[0]
        self.setWindowTitle(pathname[0])

    def openFile(self):
        fname = QFileDialog.getOpenFileName(
            self, 'Open file', 'D:\codefirst.io\PyQt5 Text Editor', 'Text files (*.txt)')
        self.setWindowTitle(fname[0])
        with open(fname[0], 'r') as f:
            filetext = f.read()
            self.text_editor.setText(filetext)
        self.current_path = fname[0]

    def undo(self):
        self.text_editor.undo()

    def redo(self):
        self.text_editor.redo()

    def copy(self):
        self.text_editor.copy()

    def paste(self):
        self.text_editor.paste()

    def compile_code(self):
        code = self.text_editor.toPlainText()
        self.compiler_.clear()  # Pulisci l'output precedente

        output, error = self.compile_code_internal(code)
        if error:
            self.compiler_.append(error)
        else:
            self.compiler_.append("Compiled with no errors")

    def compile_code_internal(self, code):
        try:
            compiled_code = compile(code, '<string>', 'exec')
            exec_output = {}
            exec(compiled_code, exec_output)
            return None, None  # Restituisce nessun output e nessun errore
        except SyntaxError as e:
            return None, f"Errore di sintassi: {e}"
        except Exception as e:
            error_message = traceback.format_exc()
            return None, f"Errore di esecuzione:\n{error_message}"

    def run_code(self):
        code = self.text_editor.toPlainText()
        self.compiler_.clear()  # Pulisci l'output precedente

        # Compila il codice e mostra il risultato
        self.compile_code()

        # Cattura l'output in un buffer
        buffer = io.StringIO()
        original_stdout = sys.stdout  # Salva l'originale stdout
        sys.stdout = buffer  # Reindirizza stdout

        try:
            local_vars = {}
            exec(code, {}, local_vars)
            output = buffer.getvalue()
            if output:
                # Mostra l'output nel QTextEdit
                self.compiler_side_window.append(output)
        except Exception as e:
            error_message = traceback.format_exc()
            self.compiler_.append(f"Errore:\n{error_message}")
        finally:
            sys.stdout = original_stdout  # Ripristina l'originale stdout
            buffer.close()  # Chiudi il buffer

    def open_connectors_window(self):
            # Se la finestra Connectors è già aperta, portala in primo piano
            if self.connectors_window is None or not self.connectors_window.isVisible():
                self.connectors_window = Connectors(self)  # Crea una nuova finestra Connectors
            else:
                self.connectors_window.raise_()  # Porta la finestra già aperta in primo piano
                self.connectors_window.activateWindow()  # Attiva la finestra

            print("Connection established")
    
    def on_close(self, event):
        self.connectors_window = None  # Imposta a None quando la finestra viene chiusa
        event.accept()  # Accetta l'evento di chiusura


def main():
    app = QtWidgets.QApplication(sys.argv)
    # Inizia con lo splash screen
    splash = SplashScreen()

    # Esegui il loop dell'applicazione
    sys.exit(app.exec_())


if __name__ == "__main__":
    main()
