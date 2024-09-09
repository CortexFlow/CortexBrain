""" 
ReadMe:
This function is not fully tested yet and can cause unexpected framework crashes or error while compiling
"""

import matplotlib.pyplot as plt
import time

def Create2DProjectionCUDA(fig, x_grid, y_grid, I_grid, h, center_x=0, center_y=0, max_distance=None):
    """
    The `Create2DProjection` function generates a 2D projection of the illuminance distribution on a street plane
    with isolux contours and radial distance labels.

    :param fig: The parameter `fig` is the figure object to which the graph will be added.
    :param x_grid: Represents the x coordinate grid in which the illuminance values are calculated.
    :param y_grid: Represents the grid of y coordinates in the 2D projection.
    :param I_grid: Represents the distribution of illuminance on the street plane at a certain height above the ground.
    :param h: The height at which the illuminance distribution on the street plane is plotted, specified in meters above ground level.
    :param center_x: The x-coordinate of the center point from which radial lines are drawn to calculate distances.
    :param center_y: The y-coordinate of the center point from which radial lines are drawn to calculate distances.
    :param max_distance: The maximum distance from the center point to be considered for drawing radial lines and adding distance labels.
    :return: The function returns the `ax` axis added to the `fig` object.
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
