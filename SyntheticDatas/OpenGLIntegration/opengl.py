import glfw
from OpenGL.GL import *
import numpy as np

class Projection2D:
    def __init__(self, width=800, height=600, title="Illuminance Projection"):
        self.width = width
        self.height = height
        self.title = title
        self.window = None
        self.vao_rect = None
        self.vbo_rect = None
        self.texture = None

        # Inizializzazione della finestra e del contesto OpenGL
        if not glfw.init():
            raise Exception("GLFW non può essere inizializzato!")

        self.window = glfw.create_window(self.width, self.height, self.title, None, None)
        if not self.window:
            glfw.terminate()
            raise Exception("Impossibile creare la finestra GLFW!")

        glfw.make_context_current(self.window)
        glOrtho(0, 1, 0, 1, -1, 1)  # Setup ortogonale 2D

    def setup_buffers(self, x_grid, y_grid, I_grid):
        """
        Configura i buffer e la texture per il rendering del rettangolo con la texture della griglia.
        """
        # Scala I_grid per visualizzazione
        scale_factor = 1  # Fattore di scala per rendere visibili i valori
        scaled_I_grid = I_grid * scale_factor

        # Normalizza I_grid scalato per l'uso come texture
        max_I = np.max(scaled_I_grid)
        norm_I_grid = scaled_I_grid / max_I

        # Setup della texture
        self.texture = glGenTextures(1)
        glBindTexture(GL_TEXTURE_2D, self.texture)
        glTexImage2D(GL_TEXTURE_2D, 0, GL_R32F, norm_I_grid.shape[1], norm_I_grid.shape[0], 0, GL_RED, GL_FLOAT, norm_I_grid)
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE)
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE)
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR)
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR)

        # Vertici per il rettangolo (copre tutto il viewport)
        rect_vertices = np.array([
            0.0, 0.0, 0.0, 1.0,
            1.0, 0.0, 1.0, 1.0,
            1.0, 1.0, 1.0, 0.0,
            0.0, 1.0, 0.0, 0.0
        ], dtype=np.float32)

        self.vao_rect = glGenVertexArrays(1)
        self.vbo_rect = glGenBuffers(1)
        glBindVertexArray(self.vao_rect)
        glBindBuffer(GL_ARRAY_BUFFER, self.vbo_rect)
        glBufferData(GL_ARRAY_BUFFER, rect_vertices.nbytes, rect_vertices, GL_STATIC_DRAW)

        # Layout per i vertici e le coordinate della texture
        glVertexAttribPointer(0, 2, GL_FLOAT, GL_FALSE, 4 * sizeof(GLfloat), ctypes.c_void_p(0))
        glEnableVertexAttribArray(0)
        glVertexAttribPointer(1, 2, GL_FLOAT, GL_FALSE, 4 * sizeof(GLfloat), ctypes.c_void_p(2 * sizeof(GLfloat)))
        glEnableVertexAttribArray(1)

        glBindBuffer(GL_ARRAY_BUFFER, 0)
        glBindVertexArray(0)

    def render(self):
        """
        Ciclo principale di rendering.
        """
        while not glfw.window_should_close(self.window):
            # Imposta il colore di sfondo (scuro)
            glClear(GL_COLOR_BUFFER_BIT)
            glClearColor(0.0, 0.0, 0.0, 1.0)

            # Disegna il rettangolo con la texture
            glBindTexture(GL_TEXTURE_2D, self.texture)
            glBindVertexArray(self.vao_rect)
            glDrawArrays(GL_TRIANGLE_FAN, 0, 4)
            glBindVertexArray(0)

            # Scambia i buffer e gestisce gli eventi
            glfw.swap_buffers(self.window)
            glfw.poll_events()

        # Cleanup
        glDeleteTextures(1, [self.texture])
        glDeleteVertexArrays(1, [self.vao_rect])
        glDeleteBuffers(1, [self.vbo_rect])
        glfw.terminate()

def main():
    # Dimensioni della finestra
    width, height = 800, 600
    
    # Creazione di una griglia per la texture
    x_grid, y_grid = np.meshgrid(np.linspace(0, 1, width), np.linspace(0, 1, height))
    
    # Creazione di una distribuzione gaussiana centrata
    mean_x, mean_y = 0.5, 0.5  # Centro dello schermo
    sigma = 0.1  # Deviazione standard della gaussiana
    intensity = np.exp(-((x_grid - mean_x) ** 2 + (y_grid - mean_y) ** 2) / (2 * sigma ** 2))
    
    # Normalizza i valori di intensità tra 0 e 1
    intensity /= np.max(intensity)
    
    # Crea l'oggetto Projection2D e setup dei buffer
    projection = Projection2D(width=width, height=height)
    projection.setup_buffers(x_grid, y_grid, intensity)
    
    # Esegui il rendering
    projection.render()

if __name__ == "__main__":
    main()
