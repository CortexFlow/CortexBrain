import pygame
import numpy as np
from BaseAgent import Agent
from Environment import Environment
import Graphics as Graphics
from GPSModel import GPS_Sensor
import sys

# Funzione per disegnare una tabella
def draw_table(display, gps1_data, gps2_data):
    font = pygame.font.Font(None, 15)
    # Intestazione della tabella
    header = font.render("GPS Sensor Data", True, (0, 0, 0))
    display.blit(header, (850, 20))
    
    # Disegna intestazioni delle colonne
    col1 = font.render("GPS 1", True, (0, 0, 0))
    col2 = font.render("GPS 2", True, (0, 0, 0))
    display.blit(col1, (850, 60))
    display.blit(col2, (950, 60))
    
    # Disegna i dati della tabella
    for i, (val1, val2) in enumerate(zip(gps1_data, gps2_data)):
        row = font.render(f"{val1:.2f}", True, (0, 0, 0))
        display.blit(row, (850, 100 + i * 30))
        row = font.render(f"{val2:.2f}", True, (0, 0, 0))
        display.blit(row, (950, 100 + i * 30))

if __name__ == "__main__":
    # Inizializza Pygame
    pygame.init()

    # Colori
    white = (255, 255, 255)
    black = (0, 0, 0)
    yellow_light = (228, 235, 10)

    # Finestra di visualizzazione: 1000x800 per includere spazio per i grafici
    pygame.display.set_caption("Light Render")
    display = pygame.display.set_mode((1500, 800), pygame.DOUBLEBUF)

    clock, fps = pygame.time.Clock(), 240

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

    gps1_data = []
    gps2_data = []
    
    last_position1 = None
    last_position2 = None

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

        # Assicurati che i dati siano di tipo numerico e diversi dagli ultimi valori
        if isinstance(position1, list) and position1 and (last_position1 is None or position1[0] != last_position1):
            gps1_data.append(position1[0])
            last_position1 = position1[0]
        
        if isinstance(position2, list) and position2 and (last_position2 is None or position2[0] != last_position2):
            gps2_data.append(position2[0])
            last_position2 = position2[0]

        frame_count += 1

        # Disegna la tabella
        if len(gps1_data) > 0 and len(gps2_data) > 0:  # Controlla che ci siano dati
            draw_table(display, gps1_data, gps2_data)

        # Show FPS
        pygame.display.set_caption(f"FPS: {round(clock.get_fps(), 0)}")
        pygame.display.update()
