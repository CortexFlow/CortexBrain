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


from Environment import Environment

class Agent:
    def __init__(self, name, position, grid_size, speed=1):
        self.name = name
        self.position = np.array(position, dtype=float)
        self.grid_size = grid_size
        self.speed = speed
        self.grid = Environment.getGrid(environment)

    def move(self, direction,debug=False):
        directions = {
            'W': np.array([0, self.speed]),  # up
            'A': np.array([-self.speed, 0]),  # left
            'S': np.array([0, -self.speed]),   # down
            'D': np.array([self.speed, 0])    # right
        }

        if direction in directions:
            future_position = np.clip(
                self.position + directions[direction], 0, np.array(self.grid_size) - 1)

            if self.grid[int(future_position[0]), int(future_position[1])] == 1:
                if debug==True:
                    print(
                        f"Obstacle detected in position: {future_position}. You can't move there.")
            else:
                self.position = future_position
                if debug==True:
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




if __name__ == "__main__":
    # initialize pygame and a display
    pygame.init()
    display = (800, 800)
    screen = pygame.display.set_mode(display, DOUBLEBUF | OPENGL)
    #set render area
    gluOrtho2D(0, 100, 0, 100)
    glClearColor(1, 1, 1, 1)  # white background

    # create the environment and load the planimetry
    grid_size = 100
    environment = Environment(grid_size=(display[0], display[1]))
    environment.loadPlanimetry(img_path='planimetria4.jpg')
    planimetry, planimetry_edges = environment.ProcessPlanimetry(debug=True)

    # Create the agent 
    agent = Agent(name="Agent 1", position=[
                  60, 50], grid_size=grid_size, speed=1)

    # Create the sensors
    gps = GPS_Sensor(initial_position=[15, 15])
    gps2 = GPS_Sensor(initial_position=[20, 20])

    clock = pygame.time.Clock()
    running = True
    while running:
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                running = False

        # Pulisci lo schermo
        glClear(GL_COLOR_BUFFER_BIT)

        # Control the movement of the agent
        agent.control()  

        # create the grid
        glColor3f(0.8, 0.8, 0.8)  # grey color
        environment.DrawGrid(size=display[0])
        environment.AddObstacles(planimetry_edges)

        # draw the obstacles
        environment.DrawObstacles()

        # draw the agent
        environment.DrawAgent(agent.get_position())

        # draw the sensors
        environment.AddSensor(gps)
        environment.AddSensor(gps2)
        environment.DrawSensors()

        # visualize the FPS---> needs fix 
        #fps = clock.get_fps() 
        #environment.DrawFps(font, fps)

        # update the display
        pygame.display.flip()

        # 60fps limit
        clock.tick(60)

    pygame.quit()
    sys.exit()
