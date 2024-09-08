""" 
ReadMe:
This function is not fully tested yet and can cause unexpected framework crashes or error while compiling
"""


import cupy as cp
import matplotlib.pyplot as plt
import time

def Create2DProjectionCUDA(fig, x_grid, y_grid, I_grid, h, center_x=0, center_y=0, max_distance=None):
    """
    La funzione `Create2DProjection` genera una proiezione 2D della distribuzione di illuminamento su un piano stradale
    con contorni isolux e etichette di distanza radiale.

    :param fig: Il parametro `fig` è l'oggetto figura a cui verrà aggiunto il grafico.
    :param x_grid: Rappresenta la griglia delle coordinate x in cui vengono calcolati i valori di illuminamento.
    :param y_grid: Rappresenta la griglia delle coordinate y nella proiezione 2D.
    :param I_grid: Rappresenta la distribuzione dell'illuminamento sul piano stradale a una certa altezza da terra.
    :param h: L'altezza a cui viene tracciata la distribuzione di illuminamento sul piano stradale, specificata in metri dal livello del suolo.
    :param center_x: La coordinata x del punto centrale da cui vengono tracciate le linee radiali per calcolare le distanze.
    :param center_y: La coordinata y del punto centrale da cui vengono tracciate le linee radiali per calcolare le distanze.
    :param max_distance: La distanza massima dal punto centrale da considerare per tracciare le linee radiali e aggiungere etichette di distanza.
    :return: La funzione restituisce l'asse `ax` aggiunto all'oggetto `fig`.
    """
    start_time = time.time()
    # Creare un asse nella figura
    ax = fig.add_axes([0.3, 0.25, 0.5, 0.5])  # [sinistra, basso, larghezza, altezza]

    # Tracciare la distribuzione dell'illuminamento come mappa di colori
    c = ax.pcolormesh(cp.asnumpy(x_grid), cp.asnumpy(y_grid), cp.asnumpy(I_grid), cmap='binary_r',
                      shading='auto', vmin=0, vmax=50)
    fig.colorbar(c, ax=ax, label='Illuminance (lux)')

    contour_levels = cp.arange(0, 300, 5)

    contours = ax.contour(
        cp.asnumpy(x_grid), cp.asnumpy(y_grid), cp.asnumpy(I_grid), levels=cp.asnumpy(contour_levels), colors='yellow', linewidths=1.0)
    ax.clabel(contours, inline=True, fontsize=8,
              fmt='%d lux', colors='yellow')

    # Disegnare linee radiali dal centro e aggiungere etichette per le distanze
    if max_distance is None:
        max_distance = cp.max(
            cp.sqrt((x_grid - center_x)**2 + (y_grid - center_y)**2))

    # Calcolare le distanze dal centro a ciascun punto della griglia
    distance_grid = cp.sqrt((x_grid - center_x)**2 + (y_grid - center_y)**2)

    # Aggiungere etichette di distanza accanto ai contorni isolux
    for level in contour_levels:
        contour = ax.contour(cp.asnumpy(x_grid), cp.asnumpy(y_grid), cp.asnumpy(I_grid), levels=[
            level], colors='red', linewidths=1.0)
        # Per ogni curva, trovare un punto da etichettare
        for collection in contour.collections:
            for path in collection.get_paths():
                # Trovare un punto sul percorso (usare il primo punto per semplicità)
                point = path.vertices[len(path.vertices)//2]
                # Calcolare la distanza dal centro
                distance = cp.sqrt(
                    (point[0] - center_x)**2 + (point[1] - center_y)**2)
                ax.text(point[0], point[1], f'{cp.asnumpy(distance):.1f} m',
                        color='white', fontsize=9, ha='center', va='center')

    # Impostare etichette e titolo
    ax.set_xlabel('X (meters)')
    ax.set_ylabel('Y (meters)')
    ax.set_title(
        f"Illuminance Distribution on the Road Plane at height = {h} m from the ground")
    
    end_time = time.time()
    print(f"Elapsed time: {round(end_time - start_time, 2)} s")
    
    return ax
