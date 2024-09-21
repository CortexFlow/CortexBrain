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



if __name__ == "__main__":
    pygame.init()
    # Colori
    white = (255, 255, 255)
    black = (0, 0, 0)
    yellow_light = (228, 235, 10)
    blue_light = (10, 100, 235)

    pygame.display.set_caption("Light Render")
    display = pygame.display.set_mode((800, 800), pygame.DOUBLEBUF)
    clock, fps = pygame.time.Clock(), 240

    # Prima luce (gialla)
    light1 = Graphics.LIGHT(200, Graphics.pixel_shader(200, (yellow_light), 1, False))

    # Seconda luce 
    light2 = Graphics.LIGHT(150, Graphics.pixel_shader(150, (yellow_light), 1, False))
    light3 = Graphics.LIGHT(150, Graphics.pixel_shader(150, (yellow_light), 1, False))
    light4 = Graphics.LIGHT(150, Graphics.pixel_shader(150, (yellow_light), 1, False))

    gps = GPS_Sensor(initial_position=[15, 15])
    gps2 = GPS_Sensor(initial_position=[20, 20])

    # Dimensioni e posizioni dei rettangoli (muri della stanza)
    border_thickness = 5  # Spessore dei muri
    room_width = 300
    room_height = 150
    padding = 50

    # Definizione dei rettangoli per i muri
    shadow_objects = [
        # PRIMA STANZA (Superiore sinistra)
        pygame.Rect(padding, padding, room_width, border_thickness),  # Muro superiore
        pygame.Rect(padding, padding + room_height, room_width, border_thickness),  # Muro inferiore
        pygame.Rect(padding, padding, border_thickness, room_height),  # Muro sinistro
        pygame.Rect(padding + room_width, padding, border_thickness, room_height),  # Muro destro
]
    
    # Stampa le coordinate dei rettangoli in shadow_objects
    for rect in shadow_objects:
        print(f"Rect: Left: {rect.left}, Top: {rect.top}, Right: {rect.right}, Bottom: {rect.bottom}")


    # Inizializza l'ambiente e l'agente una sola volta
    environment = Environment(grid_size=(800, 800))
    agent = Agent(name="Agent 1", position=[70, 60], environment=environment, speed=1)  # Passa l'ambiente all'agente
    environment.addAgent(agent)  # Aggiungi l'agente all'ambiente
    environment.addSensor(gps)
    environment.addSensor(gps2)
    #environment.addObstacles(shadow_objects)
    while True:
        clock.tick(fps)
        display.fill(white)

        # Environment -------------------------------------------------------
        environment.DrawGrid(display, shadow_objects)
        # Sensors -----------------------------------------------------------
        environment.DrawSensors(display)
        environment.DrawSensors(display)
        # Agent --------------------------------------------------------------
        agent.control()  # Controlla il movimento dell'agente
        environment.DrawAgent(display)

        # Lighting ------------------------------------------------------------
        lights_display = pygame.Surface((display.get_size()))

        # Aggiungi luce ambientale globale
        lights_display.blit(Graphics.global_light(display.get_size(), 25), (0, 0))

        # Prima luce gialla
        light1.main(shadow_objects, lights_display, 100, 100)

        # Seconda luce blu
        light2.main(shadow_objects, lights_display, 150, 100)
        light3.main(shadow_objects, lights_display, 250, 150)
        light4.main(shadow_objects, lights_display, 200, 150)

        # Applicazione delle luci
        display.blit(lights_display, (0, 0), special_flags=pygame.BLEND_RGBA_MULT)

        # Gestione degli eventi -----------------------------------------------
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                pygame.quit()
                exit()

        pygame.display.set_caption(str(round(clock.get_fps(), 0)))
        pygame.display.update()
