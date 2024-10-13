import sys
from Globals.imports import *
from components.LoadingScreen.splash_screen import SplashScreen



if __name__ == "__main__":
    app = QApplication(sys.argv)
    splash = SplashScreen()
    sys.exit(app.exec_())
