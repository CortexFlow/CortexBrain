""" Copyright (C) 2024 CortexFlow - All Rights Reserved
* You may use, distribute and modify this code under the
* terms of the Apache2.0 license.
*
* You should have received a copy of the Apache2.0 license with
* this file. If not, please write to: lorenzotettamanti5@gmail.com 
"""


"""  
    MIGLIORAMENTI:
    -Eliminare il codice inutile
    -Sistemare codice per fare la linea
    -Sistemare il codice per disegnare una linea
    -Aggiungere muri agli ostacoli
"""

from GPSModel import GPS_Sensor
from OpenGL.GLU import *
from OpenGL.GL import *
from pygame.locals import *
import pygame
import numpy as np
import cv2
import matplotlib.pyplot as plt
import sys
import os
sys.path.append(os.path.abspath(
    os.path.join(os.path.dirname(__file__), '../')))


# create a class Environment to store all the info about the environment (Agents, Sensors, Obstacles)


class Environment:
    def __init__(self, grid_size=(800, 800), agents=None, sensors=None):
        self.grid_size = grid_size
        self.agents = agents if agents is not None else []
        self.sensors = sensors if sensors is not None else []
        self.grid = np.zeros((grid_size[0], grid_size[1]))  # Inizializza la griglia senza ostacoli
        self.cell_size = 1
        self.planimetry = self.loadPlanimetry(img_path='planimetria2.jpg')
        self.scaling_factor = 1 / self.cell_size
        self.obstacles=[]

    def getGrid(self):
        return self.grid

    def getSensors(self):
        return self.sensors

    def getAgents(self):
        return self.agents

    def getGridSize(self):
        return self.grid_size

    def addSensor(self, new_sensor):
        self.sensors.append(new_sensor)

    def addAgent(self, new_agent):
        self.agents.append(new_agent)

    def getProperties(self):
        print(f"Agents: {self.getAgents()}")
        print(f"Sensors: {self.getSensors()}")
        print(f"Grid size: {self.getGridSize()}")
        print("-----------------------------")

    def addObstacles(self, shadow_objects):
        for rect in shadow_objects:
            self.obstacles.append(rect)



    def DrawGrid(self, display, shadow_objects):
        black = (0, 0, 0)
        for rect in shadow_objects:
            pygame.draw.rect(display, black, rect)

    
    def DrawAgent(self, display):
        red = (255, 0, 0)
        for agent in self.agents:
            agent_rect = pygame.Rect(agent.get_position(), (10, 10))  # Imposta la dimensione dell'agente
            pygame.draw.rect(display, red, agent_rect)

    def DrawSensors(self,display):
        blue = (0, 0, 255)
        for sensor in self.sensors:
            sensor_rect = pygame.Rect(sensor.getPosition(), (10, 10))  # Imposta la dimensione dell'agente
            pygame.draw.rect(display, blue, sensor_rect)
            

    def loadPlanimetry(self, img_path, debug=False):

        # 1. load the image
        img = cv2.imread(img_path)

        # resize the image
        resized = cv2.resize(img, (100, 100))
        if debug == True:
            self.planimetry = resized
            cv2.imshow('Resized Image', resized)
            return self.planimetry
        else:
            self.planimetry = resized
            return self.planimetry

    def ProcessPlanimetry(self, debug=False):
        immagine_hsv = cv2.cvtColor(self.planimetry, cv2.COLOR_BGR2HSV)
        # 3. Define color min and color max to detect
        colore_min = np.array([0, 0, 0])     # black
        colore_max = np.array([0, 0, 0])

        # create a mask for the filter the black color
        maschera = cv2.inRange(immagine_hsv, colore_min, colore_max)

        # invert the mask
        maschera_invertita = cv2.bitwise_not(maschera)

        # apply the mask
        immagine_filtrata = cv2.bitwise_and(
            self.planimetry, self.planimetry, mask=maschera_invertita)

        # 4. convert in grey scale
        grigia = cv2.cvtColor(immagine_filtrata, cv2.COLOR_BGR2GRAY)

        # 5. detect the edges using the canny algorithm
        bordi = cv2.Canny(grigia, 100, 100)

        # 6. invert the color of the edges
        bordi_invertiti = cv2.bitwise_not(bordi)

        if debug == True:
            # cv2.imshow('Inverted edges \lanimetria', bordi_invertiti)
            # print(np.unique(bordi_invertiti))
            zeros = np.where(bordi_invertiti == 0)
            # only store the 0 pixels corresponding to the edges---->returns a list
            zero_positions = list(zip(zeros[0], zeros[1]))
            # print(type(zero_positions))
            # print(zero_positions)
            # plt.imshow(bordi_invertiti, cmap='gray', vmin=0, vmax=255)  # Usa 'gray' per visualizzare in scala di grigi
            # plt.colorbar()  # Aggiunge una barra dei colori
            # plt.show()
            return bordi_invertiti, zero_positions
        else:
            return bordi_invertiti

    