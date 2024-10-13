import sys
import os
sys.path.append(os.path.abspath(
    os.path.join(os.path.dirname(__file__), '../')))

from Globals.imports import *
from Login.login import Login
from Globals.constants import GLOBAL_VAR


class SplashScreen(QMainWindow):
    def __init__(self):
        super(SplashScreen, self).__init__()
        uic.loadUi(GLOBAL_VAR.SPLASH_SCREEN_UI, self)
        self.setWindowTitle(GLOBAL_VAR.TITLE)
        self.setWindowIcon(QIcon(GLOBAL_VAR.ICON))
        self.quit = self.findChild(QPushButton, 'Quit')
        self.status = self.findChild(QLabel, 'status')
        self.setWindowFlag(Qt.FramelessWindowHint)

        # Set the loading timer for 6 seconds
        self.loading_timer = QTimer(self)
        self.loading_timer.timeout.connect(self.finish_loading)
        self.loading_timer.start(6000)  # 6000 ms = 6 seconds

        self.show()

    def finish_loading(self):
        self.loading_timer.stop()
        self.status.setText("Completed!")
        self.login_window = Login()
        self.login_window.show()
        self.close()
