import os
import sys
sys.path.append(os.path.abspath(
    os.path.join(os.path.dirname(__file__), '../')))
from shared.enum import StrEnum

BASE_DIR = os.path.dirname(os.path.abspath(__file__))  # Ottieni la directory del file corrente
class GLOBAL_VAR(StrEnum):
    ICON = os.path.join(BASE_DIR, '../public/icon.png')
    TITLE = 'CortexBrain'
    SPLASH_SCREEN_UI = os.path.join(BASE_DIR, '../assets/UI_Components/SplashScreen.ui')
    LOGIN_SCREEN_UI = os.path.join(BASE_DIR, '../assets/UI_Components/loginWindow2.ui')
    APP_SCREEN_UI = os.path.join(BASE_DIR,'../assets/UI_Components/AppInterface.ui')
    CONNECTORS_SCREEN_UI = os.path.join(BASE_DIR, '../assets/UI_Components/Connectors.ui')
    CONNECTORS_SCREEN_TITLE = 'Connectors'
    CONNECTION_ESTABLISHED_UI = os.path.join(BASE_DIR,'../assets/UI_Components/ConnectionEstablished.ui')
    CONNECTION_ESTABLISHED_TITLE = 'Connection Panel'
    STOP_CONNECTION_SCREEN_UI = os.path.join(BASE_DIR, '../assets/UI_Components/StopConnectionPanel.ui')
    STOP_CONNECTION_SCREEN_TITLE = 'Stop Connection'

