""" Copyright (C) 2024 CortexFlow - All Rights Reserved
* You may use, distribute and modify this code under the
* terms of the Apache2.0 license.
*
* You should have received a copy of the Apache2.0 license with
* this file. If not, please write to: lorenzotettamanti5@gmail.com 
"""
import sys
import os
sys.path.append(os.path.abspath(
    os.path.join(os.path.dirname(__file__), '../')))

import matplotlib.pyplot as plt
import cv2
import numpy as np
import pygame
from pygame.locals import *
from OpenGL.GL import *
from OpenGL.GLU import *
from GPSModel import GPS_Sensor

# create a class Environment to store all the info about the environment (Agents, Sensors, Obstacles)

class Environment:
    def __init__(self, grid_size=(800, 800), agents=None, sensors=None):
        self.grid_size = grid_size
        self.agents = agents if agents is not None else []
        self.sensors = sensors if sensors is not None else []
        # Initialize a grid with no obstacles
        self.grid = np.zeros((grid_size[0], grid_size[1]))
        self.cell_size = 1
        self.planimetry = self.loadPlanimetry(img_path='planimetria2.jpg')
        self.scaling_factor=1/self.cell_size
    
    def getGrid(self):
        return self.grid
    
    def getSensors(self):
        return self.sensors

    def getAgents(self):
        return self.agents

    def getGridSize(self):
        return self.grid_size

    def AddSensor(self, new_sensor):
        self.sensors.append(new_sensor)

    def addAgent(self, new_agent):
        self.agents.append(new_agent)

    def getProprierties(self):
        print(f"Agents: {self.getAgents()}")
        print(f"Sensors: {self.getSensors()}")
        print(f"Grid size: {self.getGridSize()}")
        print("-----------------------------")

    def AddObstacles(self, obstacles):
        """Obstacles should be a list of (x, y) tuples indicating the obstacle positions."""
        for obstacle in obstacles:
            x, y = obstacle
            self.grid[x, y] = 1  # Mark the obstacle position in the grid

    def DrawGrid(self, size):
        glBegin(GL_LINES)
        for i in range(size + 1):
            glVertex2d(i * self.cell_size, 0)
            glVertex2d(i * self.cell_size, size * self.cell_size)
            glVertex2d(0, i * self.cell_size)
            glVertex2d(size * self.cell_size, i * self.cell_size)
        glEnd()

    def DrawObstacles(self):
        """Draws the obstacles on the grid."""
        glColor3f(0, 0, 0)  # Black color for obstacles
        glBegin(GL_QUADS)
        for x in range(self.grid_size[0]):
            for y in range(self.grid_size[1]):
                if self.grid[x, y] == 1:  # Check if there's an obstacle
                    # Draw a black rectangle at the position of the obstacle
                    # takes two float arguments representing the x,y coordinates in the grid space
                    glVertex2d(x * self.cell_size - self.cell_size/2 ,
                               y * self.cell_size - self.cell_size/2 )
                    glVertex2d(x * self.cell_size + self.cell_size/2 ,
                               y * self.cell_size - self.cell_size/2 )
                    glVertex2d(x * self.cell_size + self.cell_size/2 ,
                               y * self.cell_size + self.cell_size/2 )
                    glVertex2d(x * self.cell_size - self.cell_size/2 ,
                               y * self.cell_size + self.cell_size/2 )

        glEnd()

    def DrawLine(self, point_coordinates=[(4, 4), (10, 4)], orientation="N"):
        directions = {
            'N': np.array([0, 1]),
            'S': np.array([0, -1]),
            'W': np.array([-1, 0]),
            'O': np.array([1, 0])
        }
        start = np.array(point_coordinates[0])
        end = np.array(point_coordinates[1])
        length = end-start
        line = []
        # Genere points from start to end 
        for i in range(int(np.linalg.norm(length)) + 1):  # euclid distance rounded
            # evaluate the next point
            point = start + i * directions[orientation]
            # add the point(converted to list) to the "line" list
            line.append(point.tolist())
        # generates a vector with the points and the given direction
        self.AddObstacles(line)  # generates a line in the given direction

    def DrawAgent(self, position):
        glColor3f(1.0, 0.0, 0.0)  # Red color
        glBegin(GL_QUADS)
        glVertex2d(position[0] * self.scaling_factor*self.cell_size - self.scaling_factor*self.cell_size / 2,
                   position[1] * self.scaling_factor*self.cell_size - self.scaling_factor*self.cell_size / 2)
        glVertex2d(position[0] * self.scaling_factor*self.cell_size + self.scaling_factor*self.cell_size / 2,
                   position[1] * self.scaling_factor*self.cell_size - self.scaling_factor*self.cell_size / 2)
        glVertex2d(position[0] * self.scaling_factor*self.cell_size + self.scaling_factor*self.cell_size / 2,
                   position[1] * self.scaling_factor*self.cell_size + self.scaling_factor*self.cell_size / 2)
        glVertex2d(position[0] * self.scaling_factor*self.cell_size - self.scaling_factor*self.cell_size / 2,
                   position[1] * self.scaling_factor*self.cell_size + self.scaling_factor*self.cell_size / 2)
        glEnd()

    def DrawSensors(self):
        glColor3f(0.0, 0.0, 1.0)  # Blue color for sensor
        glBegin(GL_QUADS)

        # Get the position of the first sensor
        # Assuming this returns (x, y) tuple
        for i in range(len(self.sensors)):
            sensor_position = self.sensors[i].getPosition()
            x, y = sensor_position[0], sensor_position[1]

            # Draw the sensor as a small square (quad) at its position
            glVertex2d(round(x * self.scaling_factor*self.cell_size - self.scaling_factor*self.cell_size/2, 1),
                       round(y * self.scaling_factor*self.cell_size - self.scaling_factor*self.cell_size/2, 1))
            glVertex2d(round(x * self.scaling_factor*self.cell_size + self.scaling_factor*self.cell_size/2, 1),
                       round(y * self.scaling_factor*self.cell_size  - self.scaling_factor*self.cell_size/2, 1))
            glVertex2d(round(x * self.scaling_factor*self.cell_size + self.scaling_factor*self.cell_size/2, 1),
                       round(y * self.scaling_factor*self.cell_size + self.scaling_factor*self.cell_size/2, 1))
            glVertex2d(round(x * self.scaling_factor*self.cell_size - self.scaling_factor*self.cell_size/2, 1),
                       round(y * self.scaling_factor*self.cell_size + self.scaling_factor*self.cell_size/2, 1))

        glEnd()

        """     
        NEED FIXING!
    def DrawFps(self, font, fps):
        # White color for the text
        fps_text = font.render(f"FPS: {fps:.2f}", True, (255, 255, 255))
        text_rect = fps_text.get_rect(topleft=(10, 10))

        # Define fixed dimensions for the rectangle
        rect_width = 100
        rect_height = 40

        # Draw a rectangle around the text
        pygame.draw.rect(screen, (0, 0, 0), (text_rect.x - 5, text_rect.y - 5,
                         rect_width, rect_height))  # Black rectangle for better contrast
        # Draw the text inside the rectangle
        screen.blit(fps_text, (text_rect.x, text_rect.y)) 
        """

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
            #print(np.unique(bordi_invertiti))
            zeros = np.where(bordi_invertiti == 0)
            # only store the 0 pixels corresponding to the edges---->returns a list
            zero_positions = list(zip(zeros[0], zeros[1]))
            #print(type(zero_positions))
            # print(zero_positions)
            # plt.imshow(bordi_invertiti, cmap='gray', vmin=0, vmax=255)  # Usa 'gray' per visualizzare in scala di grigi
            # plt.colorbar()  # Aggiunge una barra dei colori
            # plt.show()
            return bordi_invertiti, zero_positions
        else:
            return bordi_invertiti
