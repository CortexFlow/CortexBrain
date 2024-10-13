
import sys
import os
sys.path.append(os.path.abspath(
    os.path.join(os.path.dirname(__file__), '../')))
sys.path.append(os.path.abspath(
    os.path.join(os.path.dirname(__file__), './')))


from Globals.imports import *
from components.LoadingScreen.splash_screen import SplashScreen



if __name__ == "__main__":
    app = QApplication(sys.argv)
    splash = SplashScreen()
    sys.exit(app.exec_())
