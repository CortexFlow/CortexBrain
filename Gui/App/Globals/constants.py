import os
import unittest
import sys
sys.path.append(os.path.abspath(
    os.path.join(os.path.dirname(__file__), '../')))
from shared.enum import StrEnum


class GLOBAL_VAR(StrEnum):
    BASE_DIR = os.path.dirname(os.path.abspath(__file__))  # Ottieni la directory del file corrente
    ICON = os.path.join(BASE_DIR, '../public/icon.png')
    TITLE = 'CortexBrain'
    SPLASH_SCREEN_UI = os.path.join(BASE_DIR, '../assets/UI_Components/SplashScreen.ui')
    LOGIN_SCREEN_UI = os.path.join(BASE_DIR, '../assets/UI_Components/loginWindow2.ui')
    APP_SCREEN_UI = os.path.join(BASE_DIR,'../assets/UI_Components/AppInterface.ui')
    CONNECTORS_SCREEN_UI = os.path.join(BASE_DIR, '../assets/UI_Components/Connectors.ui')
    CONNECTORS_SCREEN_TITLE = 'Connectors'
    STOP_CONNECTION_SCREEN_UI = os.path.join(BASE_DIR, '../assets/UI_Components/StopConnectionPanel.ui')
    STOP_CONNECTION_SCREEN_TITLE = 'Stop Connection'

class TestGlobalVar(unittest.TestCase):
    def test_file_paths_exist(self):
        """Test that all file paths defined in GLOBAL_VAR exist."""
        for var in GLOBAL_VAR:
            # Ottieni il percorso assoluto
            path = os.path.abspath(var.value)
            # Verifica che il file esista
            self.assertTrue(os.path.exists(path), f"File non trovato: {path}")

if __name__ == '__main__':
    unittest.main()
