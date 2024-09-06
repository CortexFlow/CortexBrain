import os
import sys
sys.path.append(os.path.abspath(
    os.path.join(os.path.dirname(__file__), '../')))

import time
import numpy as np
import matplotlib.pyplot as plt


from lightModel import Light, loadFromCSV, CalculateSolidAngleMonteCarloParallel
from lightModel import Create2DProjection
if __name__ == "__main__":
    
    # Parameters and configurations
    x_range = (-30, 30, 1000)
    y_range = (-30, 30, 1000)
    val = np.arange(0, 181, 1)
    angles = np.radians(val)

    # Load photometric data
    df = loadFromCSV("../Datasets/LED9W.csv")

    # List to store solid angles
    sAng = []

    # Calculate solid angle
    start_time = time.time()
    solid_angle = CalculateSolidAngleMonteCarloParallel(df)
    sAng.append(solid_angle)
    end_time = time.time()
    print(f"Elapsed time: {round(end_time - start_time, 2)} s")

    # Initialize the Light objects
    light = Light(position=[45.800043, 8.952930, 6], power=9,
                  orientation_angle=290, diffusion_angle=60,
                  photometric_map=df, solid_angles=sAng, label="Light 1")

    light1 = Light(position=[45.800043, 8.952930, 2], power=9,
                  orientation_angle=290, diffusion_angle=60,
                  photometric_map=df, solid_angles=sAng, label="Light 2")

    light2 = Light(position=[45.800043, 8.952930, 2], power=9,
                  orientation_angle=290, diffusion_angle=60,
                  photometric_map=df, solid_angles=sAng, label="Light 3")

    light3 = Light(position=[45.800043, 8.952930, 6], power=9,
                  orientation_angle=290, diffusion_angle=60,
                  photometric_map=df, solid_angles=sAng, label="Light 4")

    light4 = Light(position=[45.800043, 8.952930, 6], power=9,
                  orientation_angle=290, diffusion_angle=60,
                  photometric_map=df, solid_angles=sAng, label="Light 5")








    
    
    
    
    
    
    # Get the light sensor status
    light.getStatus()


    # Calculate grid and illuminance
    light_coords = [0, 0, light.getHeight()]
    light1_coords = [-10, -20, light1.getHeight()]
    light2_coords = [-20, 20, light2.getHeight()]
    light3_coords = [20, 20, light3.getHeight()]
    light4_coords = [10, -10, light4.getHeight()]

    
    #need to add direction to the values
    x_grid, y_grid, I_grid = light.SimGrid(x_range, y_range, light_coords, df) 
    x_grid1, y_grid1, I_grid1 = light1.SimGrid(x_range, y_range, light1_coords, df)
    x_grid2, y_grid2, I_grid2 = light2.SimGrid(x_range, y_range, light2_coords, df)
    x_grid3, y_grid3, I_grid3 = light3.SimGrid(x_range, y_range, light3_coords, df)
    x_grid4, y_grid4, I_grid4 = light4.SimGrid(x_range, y_range, light4_coords, df)
 



    multiple_lights=Light.SimMultipleGrid(*[1, I_grid,I_grid1,I_grid2,I_grid3,I_grid4])
    print("Sim multiple grids:", multiple_lights)
    
    
    
    # Crea una figura con le dimensioni specificate
    fig = plt.figure(figsize=(14, 4))

    # Usa Create2DProjection per creare l'asse e configurare la figura
    ax1 = Create2DProjection(fig, x_grid, y_grid, multiple_lights, light.getHeight())
    plt.tight_layout()
    plt.show()

    #add function to print multiple grids in one 
