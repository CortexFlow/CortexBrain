import sys
from PyQt5 import QtWidgets, uic
from PyQt5.QtWidgets import (QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout,
                             QPushButton, QLabel, QFrame, QLineEdit, QFileDialog, QMessageBox)
from PyQt5.QtGui import QIcon
from PyQt5.QtCore import QTimer, Qt, QUrl, pyqtSlot, QObject
from PyQt5.QtWebEngineWidgets import QWebEngineView
from PyQt5.QtWebChannel import QWebChannel
from PyQt5.QtWidgets import QComboBox



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

import time
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
        self.side_images = self.findChild(QLabel,"side_img")

        self.show()

    def handle_login(self):
        #next-->put login 
        
        # Once login is successful, open the main window
        self.main_app = MainWindow()
        self.main_app.show()
        
        # Close the login window
        self.close()

class MainWindow(QMainWindow):
    def __init__(self):
        super(MainWindow, self).__init__()
        # Specifica il percorso al tuo file .ui
        uic.loadUi('./AppInterface.ui', self)
        self.setWindowTitle('CortexBrain')
        self.setWindowIcon(QIcon("icon.png"))

        # Collega i pulsanti agli eventi
        self.btn_settings.clicked.connect(self.open_settings)
        self.customer_support.clicked.connect(self.custom_support)
        self.donate_btn.clicked.connect(self.donate)
        self.go_home_btn.clicked.connect(self.GoHome)  # Pulsante per la home
        self.go_sim_btn.clicked.connect(self.GoSim)    # Pulsante per la simulazione
        self.go_datas_btn.clicked.connect(self.GoDatas)
        self.go_progetta_btn.clicked.connect(self.GoProgetta)

        # Se desideri collegare il QFrame a un evento, ad esempio, il clic:
        self.frame_extra_menus.mousePressEvent = self.frame_clicked

        # Imposta il widget di partenza nello stackedWidget
        self.stackedWidget.setCurrentWidget(self.page_home)  # Imposta la pagina iniziale

    def open_settings(self):
        print("Settings button clicked")
        self.stackedWidget.setCurrentWidget(self.page_settings)  # Cambia alla pagina delle impostazioni

    def custom_support(self):
        print("Custom Support button clicked")

    def donate(self):
        print("Donate button clicked")

    def frame_clicked(self, event):
        print("Extra menus frame clicked")

    def GoHome(self):
        print("Home button clicked")
        self.stackedWidget.setCurrentWidget(self.page_home)  # Cambia alla pagina home

    def GoSim(self):
        print("Sim button clicked")
        self.stackedWidget.setCurrentWidget(self.page_sim)  # Cambia alla pagina simulazione

    def GoDatas(self):
        print("Datas button clicked")
        self.stackedWidget.setCurrentWidget(self.page_datas)  # Cambia alla pagina dati

    def GoProgetta(self):
        print("Datas button clicked")
        self.stackedWidget.setCurrentWidget(self.page_progetta)  # Cambia alla pagina dati
    
    
def main():
    app = QtWidgets.QApplication(sys.argv)
    # Inizia con lo splash screen
    splash = SplashScreen()
    
    # Esegui il loop dell'applicazione
    sys.exit(app.exec_())


if __name__ == "__main__":
    main()
