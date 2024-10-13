from PyQt5.QtWidgets import (QApplication, QMainWindow, QVBoxLayout,
                             QPushButton, QLabel, QTableWidgetItem, QFileDialog, QTextEdit)
from PyQt5 import uic
from PyQt5.QtGui import QIcon, QPixmap, QColor, QSyntaxHighlighter, QTextCharFormat
from PyQt5.QtCore import QTimer, QThreadPool, Qt, QRegularExpression
from matplotlib.backends.backend_qt5agg import FigureCanvasQTAgg as FigureCanvas
from matplotlib.figure import Figure
from PyQt5.QtCore import pyqtSignal, QObject