import sys
from PyQt5 import QtWidgets, uic


class MainWindow(QtWidgets.QMainWindow):
    def __init__(self):
        super(MainWindow, self).__init__()
        # Specifica il percorso al tuo file .ui
        uic.loadUi('./my_interface.ui', self)

        # Collega i pulsanti agli eventi
        self.btn_settings.clicked.connect(self.open_settings)
        self.custome_support.clicked.connect(self.custom_support)
        self.donate_btn.clicked.connect(self.donate)
        self.go_home_btn.clicked.connect(self.GoHome)  # Pulsante per la home
        self.go_sim_btn.clicked.connect(self.GoSim)    # Pulsante per la simulazione
        self.go_datas_btn.clicked.connect(self.GoDatas)

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
        self.stackedWidget.setCurrentWidget(self.page_settings)  # Cambia alla pagina settings

    def GoDatas(self):
        print("Datas button clicked")


def main():
    app = QtWidgets.QApplication(sys.argv)
    window = MainWindow()
    window.show()
    sys.exit(app.exec_())


if __name__ == "__main__":
    main()
