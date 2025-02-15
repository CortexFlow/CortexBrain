import os
import sys
sys.path.append(os.path.abspath(
    os.path.join(os.path.dirname(__file__), '../')))

import time
import numpy as np
import matplotlib.pyplot as plt


from lightModel import Light, loadFromCSV, CalculateSolidAngleMonteCarloParallel
from lightModel import Create2DProjection
from lightModel import Simulate


if __name__ == "__main__":
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
      lights = [

            Light(position=[45.800043, 8.952930, 4], power=9,
                  orientation_angle=290, diffusion_angle=60,
                  photometric_map=df, solid_angles=sAng, label="Light 0"),

            Light(position=[45.800043, 8.952930, 4], power=9,
                  orientation_angle=290, diffusion_angle=60,
                  photometric_map=df, solid_angles=sAng, label="Light 1"),

            Light(position=[45.800043, 8.952930, 4], power=9,
                  orientation_angle=290, diffusion_angle=60,
                  photometric_map=df, solid_angles=sAng, label="Light 2"),

            Light(position=[45.800043, 8.952930, 4], power=9,
                  orientation_angle=290, diffusion_angle=60,
                  photometric_map=df, solid_angles=sAng, label="Light 3")
      ]

      for light in lights:
            light.getStatus()

      # Define coordinates for each light
      light_coords = {
            lights[0]: [0, 0, lights[0].getHeight()],
            lights[1]: [10, 0, lights[1].getHeight()],
            lights[2]: [20, 0, lights[2].getHeight()],
            lights[3]: [30, 0, lights[3].getHeight()]
      }

      # Create Simulate object with the list of Light objects and their coordinates
      simulation = Simulate(label="Lights Simulation", lights=lights,
                              light_coords=light_coords, render_mode="matplotlib")
      simulation.SetXgrid((-60, 60, 500))
      simulation.SetYgrid((-60, 60, 500))
      simulation.SetGridDivisions(500)
      simulation.getProprierties()
      # Run the simulation

      results = simulation.RunSimulation()
