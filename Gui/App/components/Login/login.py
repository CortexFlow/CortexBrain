from Globals.imports import *
from MainWindow.main_window import MainWindow
from Globals.constants import GLOBAL_VAR



class Login(QMainWindow):
    def __init__(self):
        super(Login, self).__init__()
        self.setWindowTitle(GLOBAL_VAR.TITLE)
        self.setWindowIcon(QIcon(GLOBAL_VAR.ICON))
        uic.loadUi(GLOBAL_VAR.LOGIN_SCREEN_UI, self)

        # Find the login button and connect it to the login function
        self.loginButton = self.findChild(QPushButton, "login")
        self.loginButton.clicked.connect(self.handle_login)
        self.side_images = self.findChild(QLabel, "side_img")

        self.show()

    def handle_login(self):
        # Once login is successful, open the main window
        self.main_app = MainWindow()
        self.main_app.show()

        # Close the login window
        self.close()
