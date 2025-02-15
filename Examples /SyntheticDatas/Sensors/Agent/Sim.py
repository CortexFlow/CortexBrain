import pygame
import numpy as np
from BaseAgent import Agent
from Environment import Environment
import Graphics as Graphics
from GPSModel import GPS_Sensor
import pygame_chart as pyc
import sys


if __name__ == "__main__":
    # Inizializza Pygame
    pygame.init()

    # Colori
    white = (255, 255, 255)
    black = (0, 0, 0)
    yellow_light = (228, 235, 10)

    # Finestra di visualizzazione: 1000x800 per includere spazio per i grafici
    pygame.display.set_caption("Light Render")
    display = pygame.display.set_mode((1500, 800), pygame.DOUBLEBUF )

    clock, fps = pygame.time.Clock(), 60

    # Definizione delle luci
    light1 = Graphics.LIGHT(200, Graphics.pixel_shader(200, (yellow_light), 1, False))

    # Definizione dei rettangoli per i muri
    border_thickness = 5
    room_width = 300
    room_height = 150
    padding = 50
    shadow_objects = [
        pygame.Rect(padding, padding, room_width, border_thickness),
        pygame.Rect(padding, padding + room_height, room_width, border_thickness),
        pygame.Rect(padding, padding, border_thickness, room_height),
        pygame.Rect(padding + room_width, padding, border_thickness, room_height),
    ]

    gps1 = GPS_Sensor(initial_position=[150, 100])
    gps2 = GPS_Sensor(initial_position=[250, 100])

    # Inizializza l'ambiente e l'agente
    environment = Environment(grid_size=(800, 800))
    agent = Agent(name="Agent 1", position=[70, 60], environment=environment, speed=1)
    environment.addAgent(agent)
    environment.addSensor(gps1)
    environment.addSensor(gps2)
    # Setup per i grafici
    figure = pyc.Figure(display, 800, 0, 700, 800)  # Posizione e dimensioni della figura

    gps1_data = []
    gps2_data = []

    # Variabile per il conteggio dei frame
    frame_count = 0

    while True:
        clock.tick(fps)
        display.fill(white)

        # Draw Environment
        environment.DrawGrid(display, shadow_objects)
        environment.DrawSensors(display)
        environment.DrawAgent(display)
        agent.control()

        # Lighting
        lights_display = pygame.Surface((800, 800))
        lights_display.blit(Graphics.global_light((800, 800), 25), (0, 0))
        light1.main(shadow_objects, lights_display, 100, 100)
        display.blit(lights_display, (0, 0), special_flags=pygame.BLEND_RGBA_MULT)

        # Event handling
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                pygame.quit()
                sys.exit()

        # Update GPS data
        position1 = gps1.getPosition()
        position2 = gps2.getPosition()

        # Assicurati che i dati siano di tipo numerico
        if isinstance(position1, list) and position1:  # Controlla che sia una lista non vuota
            gps1_data.append(position1[0])  # Aggiungi il primo valore della lista
            #print(gps1_data)
        if isinstance(position2, list) and position2:  # Controlla che sia una lista non vuota
            gps2_data.append(position2[0])  # Aggiungi il primo valore della lista
            #print(gps2_data)

        # Esegui calcoli su GPU solo ogni 15 frame
        if frame_count % 15 == 0:
            # Aggiungi grafici solo se ci sono dati
            if len(gps1_data) > 1 and len(gps2_data) > 1:  # Controlla che ci siano abbastanza dati
                figure.line('GPS Sensor 1', list(range(len(gps1_data))), gps1_data)
                figure.line('GPS Sensor 2', list(range(len(gps2_data))), gps2_data)

        frame_count += 1

        # Disegna la figura solo se ci sono dati sufficienti
        if len(gps1_data) > 1 and len(gps2_data) > 1:  # Assicurati di avere più di un dato
            try:
                figure.draw()
            except ValueError as e:
                pass

        # Show FPS
        pygame.display.set_caption(f"FPS: {round(clock.get_fps(), 0)}")
        pygame.display.update()
