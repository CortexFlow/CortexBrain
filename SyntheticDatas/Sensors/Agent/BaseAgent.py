""" Copyright (C) 2024 CortexFlow - All Rights Reserved
* You may use, distribute and modify this code under the
* terms of the Apache2.0 license.
*
* You should have received a copy of the Apache2.0 license with
* this file. If not, please write to: lorenzotettamanti5@gmail.com 
"""
import Graphics as Graphics
import sys
import os
sys.path.append(os.path.abspath(
    os.path.join(os.path.dirname(__file__), '../')))
import matplotlib.pyplot as plt
import numpy as np
import pygame
from pygame.locals import *
from OpenGL.GL import *
from OpenGL.GLU import *
from GPSModel import GPS_Sensor
from Environment import Environment
from lightModel import Light, loadFromCSV, CalculateSolidAngleMonteCarloParallel
from lightModel import Create2DProjection
from lightModel import Simulate

import pygame_chart as pyc
class Agent:
    def __init__(self, name, position, environment, speed=1):
        self.name = name
        self.position = np.array(position, dtype=float)
        self.environment = environment  # Riferimento all'ambiente
        self.speed = speed
        
    def move(self, direction, debug=False):
        directions = {
            'W': np.array([0, -self.speed]),  # su
            'A': np.array([-self.speed, 0]),  # sinistra
            'S': np.array([0, self.speed]),    # giù
            'D': np.array([self.speed, 0])     # destra
        }

        if direction in directions:
            future_position = np.clip(
                self.position + directions[direction], 0, np.array(self.environment.getGridSize()) - 1)
            
            # Controlla le collisioni con gli ostacoli
            collision = False
            for rect in self.environment.obstacles:
                if self.environment.agent.colliderect(rect):
                    collision = True
                    break

            # Se non ci sono collisioni, aggiorna la posizione
            if not collision:
                self.position = future_position
                if debug:
                    print(f"Position updated: {self.position}")


    def control(self):
        keys = pygame.key.get_pressed()
        if keys[pygame.K_w]:
            self.move('W')
        elif keys[pygame.K_a]:
            self.move('A')
        elif keys[pygame.K_s]:
            self.move('S')
        elif keys[pygame.K_d]:
            self.move('D')

    def get_position(self):
        return self.position

