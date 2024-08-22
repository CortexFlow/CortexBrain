import sys
import os
import json
import pandas as pd
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '../../'))) # access utils
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))) # access console
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
loginWindowPath = "./assets/uiElements/loginWindow.ui"

class SplashScreen(QMainWindow):
    def __init__(self):
        super(SplashScreen, self).__init__()
        uic.loadUi(UIPath, self)
        self.quit = self.findChild(QPushButton, 'Quit')
        self.status = self.findChild(QLabel, 'status')
        self.quit.clicked.connect(self.close)
        self.setWindowFlag(Qt.FramelessWindowHint)

        # Set the loading timer for 3 seconds
        self.loading_timer = QTimer(self)
        self.loading_timer.timeout.connect(self.finish_loading)
        self.loading_timer.start(6000)  # 6000 ms = 6 seconds

        self.show()

    def finish_loading(self):
        # Stop the timer
        self.loading_timer.stop()
        self.status.setText("Completed!")
        
        # Start the main application
        self.main_app = App()
        self.main_app.show()
        
        # Close the splash screen
        self.close()


class WebEngineBridge(QObject):
    def __init__(self, parent=None):
        super(WebEngineBridge, self).__init__(parent)

    @pyqtSlot(str)
    def handle_marker_click(self, poi_info):
        poi = json.loads(poi_info)
        name = poi.get('Name')
        if name in self.parent().poi_data:
            poi_details = self.parent().poi_data[name]
            coordinate = poi_details.get('Coordinate', '').replace('(', '').replace(')', '')
            lat, lng = coordinate.split(',') if coordinate else ('N/A', 'N/A')
            text = (f"{poi_details.get('Name', 'N/A')}\n"
                    f"Type: {poi_details.get('Type', 'N/A')}\n"
                    f"Distance from central point: {poi_details.get('Distance_from_POI', 'N/A')} km\n"
                    f"Coordinates: ({lat}, {lng})")
            self.parent().info_label.setText(text)
        else:
            self.parent().info_label.setText("Details not available for this POI.")
            
    
class Login():
    def LoginFunc(arg):
        pass

class App(QWidget):
    def __init__(self):
        super().__init__()
        self.setWindowTitle('CortexBrain')
        self.setGeometry(0, 0, 1800, 1080)
        self.setWindowIcon(QIcon("icon.png"))

        # Main layout
        main_layout = QVBoxLayout(self)

        # Main layout with sidebar and content
        content_layout = QHBoxLayout()

        # Sidebar
        self.sidebar = self.create_sidebar()
        content_layout.addWidget(self.sidebar)

        # Layout for the main content (configuration and map)
        main_content_layout = QVBoxLayout()

        # Frame for configuration data
        config_frame = QWidget()
        config_frame.setObjectName('configFrame')
        config_frame.setFixedHeight(200)
        config_layout = QVBoxLayout()

        # LineEdit for config file
        self.config_line_edit = QLineEdit(self)
        self.config_line_edit.setPlaceholderText("Select the configuration file...")
        self.config_line_edit.setObjectName('configLineEdit')

        config_btn = QPushButton('Browse', self)
        config_btn.setObjectName('configBtn')
        config_btn.clicked.connect(self.load_config_file)

        config_layout.addWidget(self.config_line_edit)
        config_layout.addWidget(config_btn)

        # Add a button to run the app
        run_btn = QPushButton('Generate Map', self)
        run_btn.setObjectName('runBtn')
        run_btn.clicked.connect(self.run_app)
        config_layout.addWidget(run_btn)

        config_frame.setLayout(config_layout)
        main_content_layout.addWidget(config_frame)

        # Map display
        self.map_view = QWebEngineView(self)
        self.map_view.setMinimumHeight(400)
        main_content_layout.addWidget(self.map_view)

        # Add the main content layout to the main layout
        content_layout.addLayout(main_content_layout)

        """ # Frame for POI information
        self.info_frame = QFrame()
        self.info_frame.setFixedHeight(400)  # Fixed height for the info window
        self.info_frame.setFrameShape(QFrame.StyledPanel)
        info_layout = QVBoxLayout()

        self.info_label = QLabel("Select a point of interest on the map", self)
        self.info_label.setWordWrap(True)  # Allow text to wrap
        info_layout.addWidget(self.info_label)

        self.info_frame.setLayout(info_layout)
        content_layout.addWidget(self.info_frame) """

        # Add the main layout to the main widget
        main_layout.addLayout(content_layout)
        self.setLayout(main_layout)

        # POI data
        self.poi_data = {}
        self.search_list = []
        self.config = None  # Add an attribute to store the configuration file

        # Bridge to connect Python and JavaScript
        self.bridge = WebEngineBridge(self)
        self.channel = QWebChannel(self.map_view.page())
        self.channel.registerObject("pyObj", self.bridge)
        self.map_view.page().setWebChannel(self.channel)

    def create_sidebar(self):
        sidebar = QFrame()
        sidebar.setFixedWidth(300)  # Set a fixed width for the sidebar
        sidebar.setFrameShape(QFrame.StyledPanel)

        sidebar_layout = QVBoxLayout()

        # Add a combobox to the sidebar
        self.category_combo_box = QComboBox(self)
        self.category_combo_box.addItems([
            "Select a category", "restaurant", "pub", "bar", "pharmacy", "school", "fountain", "fuel",
            "cafe", "veterinary", "place_of_worship", "bank", "drinking_water", "ice_cream",
            "food_court", "bicycle_rental", "theatre", "kindergarten", "post_office", "parking",
            "fast_food", "cinema", "car_rental", "boat_rental", "doctors", "clinic", "nightclub",
            "police", "hospital", "marketplace", "social_facility", "studio", "university", "prison", "courthouse"
        ])
        self.category_combo_box.currentIndexChanged.connect(self.on_category_changed)

        sidebar_layout.addWidget(self.category_combo_box)

        # Add a stretchable space at the end
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
            # Update the configuration file with the new category
            self.config["searchCategory"] = [selected_category]
            self.update_config_file()
            self.run_app()  # Regenerate the map with the new category

    def update_config_file(self):
        config_path = self.config_line_edit.text()
        if config_path and os.path.exists(config_path):
            try:
                with open(config_path, 'w') as file:
                    json.dump(self.config, file, indent=4)
            except Exception as e:
                QMessageBox.warning(self, "Error", f"Unable to update the configuration file: {e}")

    def load_config_file(self):
        config_file, _ = QFileDialog.getOpenFileName(
            self, "Select the configuration file", "", "JSON Files (*.json);;All Files (*)")
        if config_file:
            self.config_line_edit.setText(config_file)
            try:
                with open(config_file, 'r') as file:
                    self.config = json.load(file)
            except json.JSONDecodeError:
                QMessageBox.warning(self, "Error", "The configuration file is not valid JSON!")
            except Exception as e:
                QMessageBox.warning(self, "Error", f"Unable to read the configuration file: {e}")

    def run_app(self):
        config_path = self.config_line_edit.text()

        if not config_path or not os.path.exists(config_path):
            QMessageBox.warning(self, "Error", "Configuration file not found!")
            return

        if not self.config:
            QMessageBox.warning(self, "Error", "Load a valid configuration file first!")
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

            # Generate the map
            G = CreateNetworkData(df, poi=startingPlace, pos=coordsDuomo, poi_types=searchList, savePath=savePathPoi)
            self.display_map(G, savePathPoi)

        except KeyError as e:
            QMessageBox.warning(self, "Error", f"Missing key in configuration file: {e}")
        except Exception as e:
            QMessageBox.warning(self, "Error", f"Error generating the map: {e}")

    def display_map(self, G, save_path):
        map_path = f"{save_path}/mappa_punti_interesse_satellitare.html"
        if os.path.exists(map_path):
            self.map_view.setUrl(QUrl.fromLocalFile(os.path.abspath(map_path)))

            # Add code to initialize QWebChannel in JavaScript
            self.map_view.page().runJavaScript("""
                new QWebChannel(qt.webChannelTransport, function(channel) {
                    window.pyObj = channel.objects.pyObj;

                    function setupMarkers() {
                        let markers = [];
                        window.poiData = {};  // POI data for global access
                        for (let i = 0; i < poiList.length; i++) {
                            let poi = poiList[i];
                            let [lat, lng] = poi.Coordinate.replace(/[()]/g, '').split(',').map(Number);
                            let marker = new google.maps.Marker({
                                position: { lat: lat, lng: lng },
                                map: map,
                                title: poi.Name
                            });
                            marker.addListener('click', function() {
                                window.pyObj.handle_marker_click(JSON.stringify(poi));
                            });
                            markers.push(marker);
                            window.poiData[poi.Name] = poi;
                        }
                    }
                    setupMarkers();
                });
            """)
        else:
            QMessageBox.warning(self, "Error", "Map file not found!")


if __name__ == '__main__':
    app = QApplication(sys.argv)
    splash = SplashScreen()
    sys.exit(app.exec_())
