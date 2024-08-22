import sys
import os
import json
import pandas as pd
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '../../'))) #accedere a utils
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))) #accede a console
from PyQt5.QtWidgets import (QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout,
                             QPushButton, QLabel, QFrame, QLineEdit, QFileDialog, QMessageBox)
from PyQt5.QtGui import QIcon
from PyQt5 import uic
from PyQt5.QtCore import QTimer, Qt, QUrl, pyqtSlot, QObject
from PyQt5.QtWebEngineWidgets import QWebEngineView
from PyQt5.QtWebChannel import QWebChannel
from Console.RWDatas import RealWorld
from utils.PlotDatas import CreateNetworkData
from PyQt5.QtWidgets import QComboBox

UIPath = "./assets/uiElements/SplashScreen.ui"

class SplashScreen(QMainWindow):
    def __init__(self):
        super(SplashScreen, self).__init__()
        uic.loadUi(UIPath, self)
        self.quit = self.findChild(QPushButton, 'Quit')
        self.status = self.findChild(QLabel, 'status')
        self.quit.clicked.connect(self.close)
        self.setWindowFlag(Qt.FramelessWindowHint)

        # Imposta il timer di caricamento per 3 secondi
        self.loading_timer = QTimer(self)
        self.loading_timer.timeout.connect(self.finish_loading)
        self.loading_timer.start(3000)  # 3000 ms = 3 secondi

        self.show()

    def finish_loading(self):
        # Ferma il timer
        self.loading_timer.stop()
        self.status.setText("Completed!")
        
        # Avvia l'applicazione principale
        self.main_app = App()
        self.main_app.show()
        
        # Chiudi lo splash screen
        self.close()


class WebEngineBridge(QObject):
    def __init__(self, parent=None):
        super(WebEngineBridge, self).__init__(parent)

    @pyqtSlot(str)
    def handle_marker_click(self, poi_info):
        poi = json.loads(poi_info)
        name = poi.get('Nome')
        if name in self.parent().poi_data:
            poi_details = self.parent().poi_data[name]
            coordinate = poi_details.get('Coordinate', '').replace('(', '').replace(')', '')
            lat, lng = coordinate.split(',') if coordinate else ('N/A', 'N/A')
            text = (f"{poi_details.get('Nome', 'N/A')}\n"
                    f"Tipo: {poi_details.get('Tipo', 'N/A')}\n"
                    f"Distanza dal punto centrale: {poi_details.get('Distance_from_POI', 'N/A')} km\n"
                    f"Coordinate: ({lat}, {lng})")
            self.parent().info_label.setText(text)
        else:
            self.parent().info_label.setText("Dettagli non disponibili per questo POI.")
            
    
class Login():
    def LoginFunc(arg):
        pass


class App(QWidget):
    def __init__(self):
        super().__init__()
        self.setWindowTitle('POI Mapper')
        self.setGeometry(0, 0, 1800, 1080)
        self.setWindowIcon(QIcon("icon.png"))

        # Layout principale
        main_layout = QVBoxLayout(self)

        # Layout principale con sidebar e contenuto
        content_layout = QHBoxLayout()

        # Sidebar
        self.sidebar = self.create_sidebar()
        content_layout.addWidget(self.sidebar)

        # Layout per il contenuto principale (configurazione e mappa)
        main_content_layout = QVBoxLayout()

        # Frame per i dati di configurazione
        config_frame = QWidget()
        config_frame.setObjectName('configFrame')
        config_frame.setFixedHeight(200)
        config_layout = QVBoxLayout()

        # LineEdit per il file config
        self.config_line_edit = QLineEdit(self)
        self.config_line_edit.setPlaceholderText("Seleziona il file di configurazione...")
        self.config_line_edit.setObjectName('configLineEdit')

        config_btn = QPushButton('Sfoglia', self)
        config_btn.setObjectName('configBtn')
        config_btn.clicked.connect(self.load_config_file)

        config_layout.addWidget(self.config_line_edit)
        config_layout.addWidget(config_btn)

        # Aggiungere un bottone per eseguire l'app
        run_btn = QPushButton('Genera Mappa', self)
        run_btn.setObjectName('runBtn')
        run_btn.clicked.connect(self.run_app)
        config_layout.addWidget(run_btn)

        config_frame.setLayout(config_layout)
        main_content_layout.addWidget(config_frame)

        # Visualizzazione mappa
        self.map_view = QWebEngineView(self)
        self.map_view.setMinimumHeight(400)
        main_content_layout.addWidget(self.map_view)

        # Aggiungi il layout del contenuto principale al layout principale
        content_layout.addLayout(main_content_layout)

        # Finestra per le informazioni del POI
        self.info_frame = QFrame()
        self.info_frame.setFixedHeight(400)  # Altezza fissa per la finestra delle informazioni
        self.info_frame.setFrameShape(QFrame.StyledPanel)
        info_layout = QVBoxLayout()

        self.info_label = QLabel("Seleziona un punto di interesse sulla mappa", self)
        self.info_label.setWordWrap(True)  # Permette al testo di andare a capo
        info_layout.addWidget(self.info_label)

        self.info_frame.setLayout(info_layout)
        content_layout.addWidget(self.info_frame)

        # Aggiungi il layout principale al widget principale
        main_layout.addLayout(content_layout)
        self.setLayout(main_layout)

        # Dati POI
        self.poi_data = {}
        self.search_list = []
        self.config = None  # Aggiungi un attributo per memorizzare il file di configurazione

        # Bridge per collegare Python e JavaScript
        self.bridge = WebEngineBridge(self)
        self.channel = QWebChannel(self.map_view.page())
        self.channel.registerObject("pyObj", self.bridge)
        self.map_view.page().setWebChannel(self.channel)

    def create_sidebar(self):
        sidebar = QFrame()
        sidebar.setFixedWidth(300)  # Imposta una larghezza fissa per la sidebar
        sidebar.setFrameShape(QFrame.StyledPanel)

        sidebar_layout = QVBoxLayout()

        # Aggiungi un combobox alla sidebar
        self.category_combo_box = QComboBox(self)
        self.category_combo_box.addItems([
            "Seleziona una categoria", "restaurant", "pub", "bar", "pharmacy", "school", "fountain", "fuel",
            "cafe", "veterinary", "place_of_worship", "bank", "drinking_water", "ice_cream",
            "food_court", "bicycle_rental", "theatre", "kindergarten", "post_office", "parking",
            "fast_food", "cinema", "car_rental", "boat_rental", "doctors", "clinic", "nightclub",
            "police", "hospital", "marketplace", "social_facility", "studio", "university", "prison", "courthouse"
        ])
        self.category_combo_box.currentIndexChanged.connect(self.on_category_changed)

        sidebar_layout.addWidget(self.category_combo_box)

        # Aggiungi uno spazio flessibile alla fine
        sidebar_layout.addStretch(1)

        sidebar.setLayout(sidebar_layout)
        return sidebar

    def on_category_changed(self, index):
        categories = [
            "", "restaurant", "pub", "bar", "pharmacy", "school", "fountain", "fuel",
            "cafe", "veterinary", "place_of_worship", "bank", "drinking_water", "ice_cream",
            "food_court", "bicycle_rental", "theatre", "kindergarten", "post_office", "parking",
            "fast_food", "cinema", "car_rental", "boat_rental", "doctors", "clinic", "nightclub",
            "police", "hospital", "marketplace", "social_facility", "studio", "university", "prison", "courthouse"
        ]
        selected_category = categories[index]
        if selected_category and self.config:
            # Aggiorna il file di configurazione con la nuova categoria
            self.config["searchCategory"] = [selected_category]
            self.update_config_file()
            self.run_app()  # Rigenera la mappa con la nuova categoria

    def update_config_file(self):
        config_path = self.config_line_edit.text()
        if config_path and os.path.exists(config_path):
            try:
                with open(config_path, 'w') as file:
                    json.dump(self.config, file, indent=4)
            except Exception as e:
                QMessageBox.warning(self, "Errore", f"Impossibile aggiornare il file di configurazione: {e}")

    def load_config_file(self):
        config_file, _ = QFileDialog.getOpenFileName(
            self, "Seleziona il file di configurazione", "", "JSON Files (*.json);;All Files (*)")
        if config_file:
            self.config_line_edit.setText(config_file)
            try:
                with open(config_file, 'r') as file:
                    self.config = json.load(file)
            except json.JSONDecodeError:
                QMessageBox.warning(self, "Errore", "Il file di configurazione non è valido JSON!")
            except Exception as e:
                QMessageBox.warning(self, "Errore", f"Impossibile leggere il file di configurazione: {e}")

    def run_app(self):
        config_path = self.config_line_edit.text()

        if not config_path or not os.path.exists(config_path):
            QMessageBox.warning(self, "Errore", "File di configurazione non trovato!")
            return

        if not self.config:
            QMessageBox.warning(self, "Errore", "Carica prima un file di configurazione valido!")
            return

        try:
            coordsDuomo = tuple(map(float, self.config["coordinates"]))
            cityName = self.config["city"]
            startingPlace = self.config["startingPlace"]
            searchList = self.config["searchCategory"]
            savePathPoi = self.config["savePOI"]

            if not os.path.exists(f"{savePathPoi}/POI.csv"):
                df = RealWorld.GetPlaceInfo(cityName)
                df.to_csv(f"{savePathPoi}/POI.csv", index=False)
                df = RealWorld.EvaluateDistance(df, coordsDuomo)
                df.to_csv(f"{savePathPoi}/DistanceFromPoi.csv", index=False)
            else:
                df = pd.read_csv(f"{savePathPoi}/POI.csv")
                df = RealWorld.EvaluateDistance(df, coordsDuomo)
                df.to_csv(f"{savePathPoi}/DistanceFromPoi.csv", index=False)

            # Genera la mappa
            G = CreateNetworkData(df, poi=startingPlace, pos=coordsDuomo, poi_types=searchList, savePath=savePathPoi)
            self.display_map(G, savePathPoi)

        except KeyError as e:
            QMessageBox.warning(self, "Errore", f"Chiave mancante nel file di configurazione: {e}")
        except Exception as e:
            QMessageBox.warning(self, "Errore", f"Errore durante la generazione della mappa: {e}")

    def display_map(self, G, save_path):
        map_path = f"{save_path}/mappa_punti_interesse_satellitare.html"
        if os.path.exists(map_path):
            self.map_view.setUrl(QUrl.fromLocalFile(os.path.abspath(map_path)))

            # Aggiungi il codice per inizializzare QWebChannel in JavaScript
            self.map_view.page().runJavaScript("""
                new QWebChannel(qt.webChannelTransport, function(channel) {
                    window.pyObj = channel.objects.pyObj;

                    function setupMarkers() {
                        let markers = [];
                        window.poiData = {};  // Dati POI per accesso globale
                        for (let i = 0; i < poiList.length; i++) {
                            let poi = poiList[i];
                            let [lat, lng] = poi.Coordinate.replace(/[()]/g, '').split(',').map(Number);
                            let marker = new google.maps.Marker({
                                position: { lat: lat, lng: lng },
                                map: map,
                                title: poi.Nome
                            });
                            marker.addListener('click', function() {
                                window.pyObj.handle_marker_click(JSON.stringify(poi));
                            });
                            markers.push(marker);
                            window.poiData[poi.Nome] = poi;
                        }
                    }
                    setupMarkers();
                });
            """)
        else:
            QMessageBox.warning(self, "Errore", "Il file della mappa non è stato trovato!")


if __name__ == '__main__':
    app = QApplication(sys.argv)
    splash = SplashScreen()
    sys.exit(app.exec_())
