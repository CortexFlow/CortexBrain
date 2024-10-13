""" GENERAL IMPORT """
import sys
import io
import traceback

""" GRAPHICS IMPORT """
from PyQt5.QtWidgets import (QApplication, QMainWindow, QVBoxLayout,
                             QPushButton, QLabel, QTableWidgetItem, QFileDialog, QTextEdit)
from PyQt5 import uic
from PyQt5.QtGui import QIcon, QPixmap, QColor, QSyntaxHighlighter, QTextCharFormat
from PyQt5.QtCore import QTimer, QThreadPool, Qt, QRegularExpression
from PyQt5.QtCore import pyqtSignal, QObject

""" DATA VISUALIZATION IMPORT """
from matplotlib.backends.backend_qt5agg import FigureCanvasQTAgg as FigureCanvas
from matplotlib.figure import Figure
import numpy as np
import random
import time

""" SERVERS AND API """
import requests
from flask import Flask, request, jsonify

""" THREADING """
import threading

""" TEST IMPORT """
import unittest