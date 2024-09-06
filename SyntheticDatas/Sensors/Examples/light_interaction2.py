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
    x_range = (-250, 250, 100)
    y_range = (-250, 250, 100)
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

    light1 = Light(position=[45.800043, 8.952930, 6], power=9,
                  orientation_angle=290, diffusion_angle=60,
                  photometric_map=df, solid_angles=sAng, label="Light 2")

    light2 = Light(position=[45.800043, 8.952930, 6], power=9,
                  orientation_angle=290, diffusion_angle=60,
                  photometric_map=df, solid_angles=sAng, label="Light 3")

    light3 = Light(position=[45.800043, 8.952930, 6], power=9,
                  orientation_angle=290, diffusion_angle=60,
                  photometric_map=df, solid_angles=sAng, label="Light 4")

    light4 = Light(position=[45.800043, 8.952930, 6], power=9,
                  orientation_angle=290, diffusion_angle=60,
                  photometric_map=df, solid_angles=sAng, label="Light 5")







    light5 = Light(position=[45.800043, 8.952930, 2], power=9,
                  orientation_angle=290, diffusion_angle=60,
                  photometric_map=df, solid_angles=sAng, label="Light 1")

    light6 = Light(position=[45.800043, 8.952930, 2], power=9,
                  orientation_angle=290, diffusion_angle=60,
                  photometric_map=df, solid_angles=sAng, label="Light 2")

    light7 = Light(position=[45.800043, 8.952930, 2], power=9,
                  orientation_angle=290, diffusion_angle=60,
                  photometric_map=df, solid_angles=sAng, label="Light 3")

    light8 = Light(position=[45.800043, 8.952930, 2], power=9,
                  orientation_angle=290, diffusion_angle=60,
                  photometric_map=df, solid_angles=sAng, label="Light 4")

    light9 = Light(position=[45.800043, 8.952930, 2], power=9,
                  orientation_angle=290, diffusion_angle=60,
                  photometric_map=df, solid_angles=sAng, label="Light 5")






    light10 = Light(position=[45.800043, 8.952930, 2], power=9,
                  orientation_angle=290, diffusion_angle=60,
                  photometric_map=df, solid_angles=sAng, label="Light 1")

    light11 = Light(position=[45.800043, 8.952930, 2], power=9,
                  orientation_angle=290, diffusion_angle=60,
                  photometric_map=df, solid_angles=sAng, label="Light 2")

    light12 = Light(position=[45.800043, 8.952930, 2], power=9,
                  orientation_angle=290, diffusion_angle=60,
                  photometric_map=df, solid_angles=sAng, label="Light 3")

    light13 = Light(position=[45.800043, 8.952930, 2], power=9,
                  orientation_angle=290, diffusion_angle=60,
                  photometric_map=df, solid_angles=sAng, label="Light 4")

    light14 = Light(position=[45.800043, 8.952930, 2], power=9,
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
    light5_coords = [80, 50, light5.getHeight()]
    light6_coords = [-30, -40, light6.getHeight()]
    light7_coords = [-60, 45, light7.getHeight()]
    light8_coords = [210, 110, light8.getHeight()]
    light9_coords = [180, -180, light9.getHeight()]
    light10_coords = [150, 50, light10.getHeight()]
    light11_coords = [-10, -200, light11.getHeight()]
    light12_coords = [-200, 20, light12.getHeight()]
    light13_coords = [200, 200, light13.getHeight()]
    light14_coords = [100, -100, light14.getHeight()]
    
    #need to add direction to the values
    x_grid, y_grid, I_grid = light.SimGrid(x_range, y_range, light_coords, df) 
    x_grid1, y_grid1, I_grid1 = light1.SimGrid(x_range, y_range, light1_coords, df)
    x_grid2, y_grid2, I_grid2 = light2.SimGrid(x_range, y_range, light2_coords, df)
    x_grid3, y_grid3, I_grid3 = light3.SimGrid(x_range, y_range, light3_coords, df)
    x_grid4, y_grid4, I_grid4 = light4.SimGrid(x_range, y_range, light4_coords, df)
    x_grid5, y_grid5, I_grid5 = light5.SimGrid(x_range, y_range, light5_coords, df)
    x_grid6, y_grid6, I_grid6 = light6.SimGrid(x_range, y_range, light6_coords, df)
    x_grid7, y_grid7, I_grid7 = light7.SimGrid(x_range, y_range, light7_coords, df)
    x_grid8, y_grid8, I_grid8 = light8.SimGrid(x_range, y_range, light8_coords, df)
    x_grid9, y_grid9, I_grid9 = light9.SimGrid(x_range, y_range, light9_coords, df)
    x_grid10, y_grid10, I_grid10 = light10.SimGrid(x_range, y_range, light10_coords, df)
    x_grid11, y_grid11, I_grid11 = light11.SimGrid(x_range, y_range, light11_coords, df)
    x_grid12, y_grid12, I_grid12 = light12.SimGrid(x_range, y_range, light12_coords, df)
    x_grid13, y_grid13, I_grid13 = light13.SimGrid(x_range, y_range, light13_coords, df)
    x_grid14, y_grid14, I_grid14 = light14.SimGrid(x_range, y_range, light14_coords, df)
 



    multiple_lights=Light.SumMultipleGrid(*[1, I_grid,I_grid1,I_grid2,
                                            I_grid3,I_grid4,I_grid5,I_grid6,
                                            I_grid7,I_grid8,I_grid9,I_grid10,
                                            I_grid11,I_grid12,I_grid13,I_grid14])
    print("Sim multiple grids:", multiple_lights)
    
    
    
    # Crea una figura con le dimensioni specificate
    fig = plt.figure(figsize=(14, 4))

    # Usa Create2DProjection per creare l'asse e configurare la figura
    ax1 = Create2DProjection(fig, x_grid, y_grid, multiple_lights, light.getHeight())
    plt.tight_layout()
    plt.show()

    #add function to print multiple grids in one 
