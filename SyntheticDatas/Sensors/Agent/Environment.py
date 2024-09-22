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
    -Sistemare il codice per disegnare una linea
    -Aggiungere muri agli ostacoli
    -convertire in OpenGL per fare 
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
        #self.planimetry = self.loadPlanimetry(img_path='planimetria2.jpg')
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
            
