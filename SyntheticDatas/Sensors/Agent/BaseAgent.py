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
from GPSModel import GPS_Sensor
from OpenGL.GLU import *
from OpenGL.GL import *
from pygame.locals import *
import pygame
import numpy as np
import cv2
import matplotlib.pyplot as plt




class Agent:
    def __init__(self, name, position, grid_size, speed=1):
        self.name = name
        self.position = np.array(position, dtype=float)
        self.grid_size = grid_size
        self.speed = speed

    def move(self, direction):
        directions = {
            '+1_x': np.array([self.speed, 0]),
            '-1_x': np.array([-self.speed, 0]),
            '+1_y': np.array([0, self.speed]),
            '-1_y': np.array([0, -self.speed])
        }
        self.position += directions[direction]
        self.position = np.clip(self.position, 0, self.grid_size - 1)

    def RandomMove(self):
        directions = ['+1_x', '-1_x', '+1_y', '-1_y']
        direction = np.random.choice(directions)
        self.move(direction)

    def get_position(self):
        return self.position

# create a class Environment to store all the info about the environment (Agents, Sensors, Obstacles)


class Environment:
    def __init__(self, grid_size=(800, 800), agents=None, sensors=None):
        self.grid_size = grid_size
        self.agents = agents if agents is not None else []
        self.sensors = sensors if sensors is not None else []
        # Initialize a grid with no obstacles
        self.grid = np.zeros((grid_size[0], grid_size[1]))
        self.cell_size = 1/8
        self.planimetry=self.loadPlanimetry(img_path='planimetria4.jpg')

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
            glVertex2f(i * self.cell_size, 0)
            glVertex2f(i * self.cell_size, size * self.cell_size)
            glVertex2f(0, i * self.cell_size)
            glVertex2f(size * self.cell_size, i * self.cell_size)
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
                    glVertex2f(x * self.cell_size, y * self.cell_size)
                    glVertex2f(x * self.cell_size +
                               self.cell_size, y * self.cell_size)
                    glVertex2f(x * self.cell_size + self.cell_size,
                               y * self.cell_size + self.cell_size)
                    glVertex2f(x * self.cell_size, y *
                               self.cell_size + self.cell_size)
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
        # Genera i punti tra start e end
        for i in range(int(np.linalg.norm(length)) + 1):  # Distanza euclidea arrotondata
            # Calcola il punto successivo
            point = start + i * directions[orientation]
            # Aggiungi il punto convertito in lista
            line.append(point.tolist())
        # generates a vector with the points and the given direction
        self.AddObstacles(line)  # generates a line in the given direction

    def DrawAgent(self, position):
        glColor3f(1.0, 0.0, 0.0)  # Red color
        glBegin(GL_QUADS)
        glVertex2f(position[0] * 8*self.cell_size - 8*self.cell_size / 2,
                   position[1] * 8*self.cell_size - 8*self.cell_size / 2)
        glVertex2f(position[0] * 8*self.cell_size + 8*self.cell_size / 2,
                   position[1] * 8*self.cell_size - 8*self.cell_size / 2)
        glVertex2f(position[0] * 8*self.cell_size + 8*self.cell_size / 2,
                   position[1] * 8*self.cell_size + 8*self.cell_size / 2)
        glVertex2f(position[0] * 8*self.cell_size - 8*self.cell_size / 2,
                   position[1] * 8*self.cell_size + 8*self.cell_size / 2)
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
            glVertex2f(round(x * 8*self.cell_size/2 - 8*self.cell_size/2,1),
                    round(y * 8*self.cell_size/2 - 8*self.cell_size/2,1))
            glVertex2f(round(x * 8*self.cell_size/2 + 8*self.cell_size/2,1),
                    round(y * 8*self.cell_size/2 - 8*self.cell_size/2,1))
            glVertex2f(round(x * 8*self.cell_size/2 + 8*self.cell_size/2,1),
                    round(y * 8*self.cell_size/2 + 8*self.cell_size/2,1))
            glVertex2f(round(x * 8*self.cell_size/2 - 8*self.cell_size/2,1),
                    round(y * 8*self.cell_size/2 + 8*self.cell_size/2,1))

        glEnd()

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

    def loadPlanimetry(self,img_path,debug=False):

        # 1. Carica l'immagine
        img = cv2.imread(img_path)

        # Ridimensiona l'immagine
        resized = cv2.resize(img, (800, 800))
        if debug==True:
            self.planimetry=resized
            cv2.imshow('Resized Image', resized)
            return self.planimetry
        else:
            self.planimetry=resized
            return self.planimetry
        
    def ProcessPlanimetry(self,debug=False):
        immagine_hsv = cv2.cvtColor(self.planimetry, cv2.COLOR_BGR2HSV)
        # 3. Definisci l'intervallo di colore per il nero
        colore_min = np.array([0, 0, 0])     # Nero
        colore_max = np.array([3, 3, 3])     # Nero con saturazione e valore bassi

        # Crea una maschera per il colore nero
        maschera = cv2.inRange(immagine_hsv, colore_min, colore_max)

        # Inverti la maschera per escludere il colore nero
        maschera_invertita = cv2.bitwise_not(maschera)

        # Applica la maschera invertita all'immagine originale
        immagine_filtrata = cv2.bitwise_and(self.planimetry, self.planimetry, mask=maschera_invertita)

        # 4. Converti l'immagine filtrata in scala di grigi
        grigia = cv2.cvtColor(immagine_filtrata, cv2.COLOR_BGR2GRAY)

        # 5. Rileva i bordi con l'algoritmo Canny
        bordi = cv2.Canny(grigia, 100, 100)

        # 6. Inverti i colori dell'immagine dei bordi
        bordi_invertiti = cv2.bitwise_not(bordi)

        # Mostra i bordi invertiti
        if debug==True:
            cv2.imshow('Bordi Invertiti della Planimetria', bordi_invertiti)
            print(np.unique(bordi_invertiti))
            zeros=np.where(bordi_invertiti==0)
            zero_positions=list(zip(zeros[0],zeros[1])) # only store the 0 pixels corresponding to the edges---->returns a list
            print(type(zero_positions))
            #print(zero_positions)
            #plt.imshow(bordi_invertiti, cmap='gray', vmin=0, vmax=255)  # Usa 'gray' per visualizzare in scala di grigi
            #plt.colorbar()  # Aggiunge una barra dei colori
            #plt.show()
            return bordi_invertiti,zero_positions
        else:
            return bordi_invertiti


if __name__ == "__main__":
    # Initialize pygame and set up the display
    pygame.init()
    global screen
    display = (800, 800)
    screen = pygame.display.set_mode(display, DOUBLEBUF | OPENGL)
    gluOrtho2D(0, 100, 0, 100)  # Adjust to fit your grid range
    glClearColor(1, 1, 1, 1)  # White background

    # Create an environment and add obstacles
    grid_size = 100
    environment = Environment(grid_size=(display[0], display[1]))
    environment.loadPlanimetry(img_path='planimetria2.jpg')
    planimetry,planimetry_edges=environment.ProcessPlanimetry(debug=True)
    """     
    obstacles = [(20, 40), (21, 21), (22, 30), (50, 50),
                 (60, 80)]  # obstacles
    environment.AddObstacles(obstacles) 
    """
    """     
    environment.DrawLine([(4, 4), (4, 80)], orientation="N")
    environment.DrawLine([(4, 4), (80, 4)], orientation="O")
    environment.DrawLine([(80, 4), (80, 80)], orientation="N")
    environment.DrawLine([(80, 80), (4, 80)], orientation="W") 
    """

    # Create an agent
    agent = Agent(name="Agent 1", position=[
                  50, 50], grid_size=grid_size, speed=0.1)
    gps = GPS_Sensor(initial_position=[15, 15])
    gps2 = GPS_Sensor(initial_position=[20, 20])

    # Initialize font for FPS display
    pygame.font.init()
    font = pygame.font.SysFont("Arial", 36)

    clock = pygame.time.Clock()
    running = True
    while running:
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                running = False

        glClear(GL_COLOR_BUFFER_BIT)

        # Draw grid
        glColor3f(255, 255, 255)  # Gray color for grid lines
        environment.DrawGrid(size=display[0])
        environment.AddObstacles(planimetry_edges)

        # Draw obstacles
        environment.DrawObstacles()

        # Randomly move the agent in the grid
        agent.RandomMove()

        # Draw agent
        environment.DrawAgent(agent.get_position())
        environment.AddSensor(gps)
        environment.AddSensor(gps2)
        environment.DrawSensors()

        # Draw FPS
        fps = clock.get_fps()
        environment.DrawFps(font, fps)

        pygame.display.flip()
        clock.tick(60)  # 10 frames per second

    pygame.quit()
