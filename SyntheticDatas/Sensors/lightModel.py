""" Copyright (C) 2024 CortexFlow - All Rights Reserved
* You may use, distribute and modify this code under the
* terms of the Apache2.0 license.
*
* You should have received a copy of the Apache2.0 license with
* this file. If not, please write to:lorenzotettamanti5@gmail.com 
"""
""" 
Suggestions for future updates:
    1.Improvements in the max range covered --> coming soon
    2.add power consumption metrics 
    3.add functions to load datas from .LDT files 
    4.improve interaction between lights
"""


# The `Light` class represents a smart light sensor with properties such as position, power, lumen,
# height, diffusion angle, and orientation angle, along with methods to get and set these properties
# and compute the maximum range covered by the light sensor.


from BaseSensor import Sensor
import math
import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
from scipy.signal import argrelextrema
import time
from joblib import Parallel, delayed
import os
os.environ['TF_CPP_MIN_LOG_LEVEL'] = '2' #disable all the tensorflow messages expect critical errors/crashes
import tensorflow as tf
import altair as alt
import sys
import warnings
sys.path.append(
    os.path.abspath(os.path.join(os.path.dirname(__file__), '../')))

warnings.filterwarnings("ignore")
class Light(Sensor):
    def __init__(self, position, power, diffusion_angle, orientation_angle, photometric_map, solid_angles, label="Smart Light", photometric_map_path="./Datasets"):
        super().__init__(SensorType="Light", value=[0.0, 0.0], label=label)
        self.lat = float(position[0])
        self.lon = float(position[1])
        self.power = power
        self.lumen, self.min_lumen, self.max_lumen, self.angular_range, self.lumen_lower_bound, self.lumen_upper_bound, self.mean_lumen = Light.evaluateLumenParallel(
            self, photometric_map, solid_angles)
        self.label = label
        self.height = float(position[2])
        self.theta = diffusion_angle
        self.orientation = orientation_angle
        self.photometric_map = photometric_map
        self.photometric_map_path = photometric_map_path

        self.light_efficiency = round(
            ((self.lumen_lower_bound+self.lumen_upper_bound)/2)/self.power, 1)

    def SetPosition(self, position):
        self.lat = float(position[0])
        self.lon = float(position[1])
        self.height = float(position[2])

    def getPosition(self):
        return (self.lat, self.lon, self.height)

    def getLumen(self):
        return self.lumen

    def getLumenRange(self):
        return [self.lumen_lower_bound, self.mean_lumen, self.lumen_upper_bound]

    def getPower(self):
        return self.power

    def getLightEfficiency(self):
        return round(self.light_efficiency, 2)

    def getHeight(self):
        return self.height

    def getDiffusionAngle(self):
        return self.theta

    def setAngle(self, new_angle):
        self.orientation = new_angle
        return self.orientation

    def getAngle(self):
        return self.orientation

    def getAngularRange(self):
        return self.angular_range

    def computeMaxRange(self):
        return round(self.height*(math.tan(math.radians(90-self.theta))), 3)

    def getPeakLumen(self):
        return round(np.max(self.lumen), 2)

    def getFilePath(self):
        return self.photometric_map_path

    def getStatus(self):
        """
        The `getStatus` function prints out various sensor status information in a formatted manner.
        """
        print("-----------------------------")
        print("Sensor Status:")
        print(f"Name: {self.name}")
        print(f"Coordinates: {self.getPosition()}")
        print(f"Power: {self.getPower()} W")
        print(f"Lumen (LB-M-UB): {self.getLumenRange()} lm")
        print(f"Lumen Peak: {self.getPeakLumen()} lm")
        print(f"Angular Range (Horizontal): {self.getAngularRange()} ")
        print(f"Height: {self.getHeight()} m")
        print(f"Diffusion Angle: {self.getDiffusionAngle()}° ")
        print(f"Orientation Angle: {self.getAngle()}° ")
        print(f"Max Range Covered: {self.computeMaxRange()} m ")
        print(f"Light Efficiency: {self.getLightEfficiency()} lm/W")
        print(f"Photometric Map Path: {self.getFilePath()}")
        print("-----------------------------")

    # Function to calculate the intensity in candelas at a point (x, y, z)
    def evaluateE(self, x, y, z, lamp_coords, df, debug="False"):
        """
        This Python function calculates the illuminance at a point from a lamp based on distance and
        intensity data.
        
        :param x: x is the x-coordinate of the point where you want to evaluate the illuminance
        :param y: y is the y-coordinate of the point where you want to evaluate the illuminance
        :param z: the variable `z` is being set to 0, which represents the ground level.
        :param lamp_coords: Represents the coordinates of the lamp in 3D space. 
        It is expected to be a tuple or list containing the x, y, z coordinates of the lamp in that order.
        :param df: DataFrame object that contains photometric map data used to determine the intensity 
        of light at a specific angle. 
        :param debug: The `debug` parameter in the `evaluateE` function is used to control whether debug
        information should be printed during the execution of the function.
        :return: The function `evaluateE` returns the illuminance value `E` in lux.
        """
        # Calculate the distance from the point to the lamp
        x_lamp = lamp_coords[0]
        y_lamp = lamp_coords[1]
        z_lamp = lamp_coords[2]
        horizontal_angle = 0
        theta_lamp = self.theta

        phi_angle = "C"+str(horizontal_angle)

        z = 0  # ground level
        d = np.sqrt((x - x_lamp)**2 + (y - y_lamp)**2 + (z - z_lamp)**2)
        # Find the corresponding intensity from the CSV file
        if phi_angle in df.columns:
            I_theta = (df[phi_angle].iloc[int(theta_lamp)])
        else:
            I_theta = 0  # If the angle is out of range, the intensity is zero

        E = I_theta / d**2  # Illuminance (lux)
        if debug == "True":
            print(f"E: {E} cd")
            return E
        else:
            return E  # Returns the illuminance in lux
        
        
    #optimization of evaluateE using TensorFlow:
    def compute_illuminance_tensor(self,x, y, z, lamp_coords, df_tensor):
        """
        The function `compute_illuminance_tensor` calculates illuminance based on lamp coordinates,
        distance, and intensity values using TensorFlow tensors.
        
        :return: The function `compute_illuminance_tensor` returns the illuminance value calculated
        based on the input parameters `x`, `y`, `z`, `lamp_coords`, and `df_tensor`.
        """
        x_lamp = lamp_coords[0]
        y_lamp = lamp_coords[1]
        z_lamp = lamp_coords[2]
        horizontal_angle = 0
        theta_lamp = self.theta

        phi_angle = "C"+str(horizontal_angle)

        # Convert to TensorFlow tensors
        x_tensor = tf.convert_to_tensor(x, dtype=tf.float32)
        y_tensor = tf.convert_to_tensor(y, dtype=tf.float32)
        z_tensor = tf.convert_to_tensor(z, dtype=tf.float32)
        x_lamp_tensor = tf.convert_to_tensor(x_lamp, dtype=tf.float32)
        y_lamp_tensor = tf.convert_to_tensor(y_lamp, dtype=tf.float32)
        z_lamp_tensor = tf.convert_to_tensor(z_lamp, dtype=tf.float32)

        # Calculate distance
        d = tf.sqrt(tf.square(x_tensor - x_lamp_tensor) + tf.square(y_tensor - y_lamp_tensor) + tf.square(z_tensor - z_lamp_tensor))

        # Extract intensity value
        I_theta = tf.gather(df_tensor[phi_angle], int(theta_lamp), axis=0)

        # Calculate illuminance
        E = I_theta / tf.square(d)
        return E

    # Function to create the 2D grid and calculate the intensity for each point
    def compute_illuminance(x, y, z, lamp_coords, df, self):
        return Light.evaluateE(self, x, y, z, lamp_coords, df)

    def SimGrid(self, x_range, y_range, lamp_coords, df):
        """
        The function SimGrid generates a grid of points and calculates illuminance values at each point
        based on given parameters and a specified evaluation function.
        """

        x = np.linspace(*x_range)
        y = np.linspace(*y_range)
        z = 0  # Road plane

        x_grid, y_grid = np.meshgrid(x, y)
        
        # Create an empty matrix to store the illuminance
        I_grid = np.zeros_like(x_grid)

        # Parallelize the computation
        results = Parallel(n_jobs=-1)(
            delayed(Light.compute_illuminance)(x_grid[i, j], y_grid[i, j], z, lamp_coords, df, self)
            for i in range(x_grid.shape[0])
            for j in range(y_grid.shape[1])
        )
        
        # Reshape the results into the grid shape
        I_grid = np.array(results).reshape(x_grid.shape)
        
        return x_grid, y_grid, I_grid
    
    #optimization using tensorflow
    def SimGridTensorFlow(self, x_range, y_range, lamp_coords, df):
        """
        Generates a grid of points and calculates illuminance values at each point using TensorFlow.

        :param x_range: Range for x-coordinates
        :param y_range: Range for y-coordinates
        :param lamp_coords: Coordinates of the lamp
        :param df: DataFrame containing intensity values
        :return: x_grid, y_grid, I_grid where I_grid is the calculated illuminance
        """

        x = np.linspace(*x_range)
        y = np.linspace(*y_range)
        z = 0  # Ground level

        # Create meshgrid
        x_grid, y_grid = np.meshgrid(x, y)

        # Convert DataFrame to TensorFlow tensor
        df_tensor = {col: tf.convert_to_tensor(df[col].values, dtype=tf.float32) for col in df.columns}

        # Compute illuminance using TensorFlow
        I_grid = Light.compute_illuminance_tensor(self,x_grid, y_grid, z, lamp_coords, df_tensor)

        # Convert result back to numpy array
        I_grid = I_grid.numpy()

        return x_grid, y_grid, I_grid

    # sum multiple particles
    def SumMultipleGrid(self, *args):
        """
        The `SumMultipleGrid` function takes multiple matrices as input, sums them element-wise, and returns
        the total sum matrix.
        :return: returns the sum of all the input grids passed as arguments.
        """
        if not args:
            raise ValueError("No matrix passed to the function")

        # Converts all the elements in a numpy array
        args = [np.array(grid) for grid in args]

        # inizialize I_grid_total 
        I_grid_total = np.zeros_like(args[0])

        # compute the sum of all the grids passed
        for grid in args:
            I_grid_total += grid

        return I_grid_total

    def evaluateLumen(self, df, solid_angles, debug="False"):
        """
        The function `evaluateLumen` calculates various lumen-related metrics from a DataFrame using
        solid angles and provides optional debug information.

        :param df: The `evaluateLumen` function takes a DataFrame `df`, a list of solid angles, and an
        optional `debug` parameter to enable debugging output. The function performs several
        calculations related to luminous flux and returns the modified DataFrame along with some
        relevant data
        :param solid_angles: Solid angles is a list of values representing the solid angles for each
        column in the DataFrame. These values are used to calculate the luminous flux for each column in
        the DataFrame
        :param debug: When `debug` is set to "True",additional information is printed during the 
        execution of the function to help with debugging, defaults to False (optional)
        :return: returns the following values in this order:
        1. `df_lumen`: DataFrame with calculated lumen values
        2. `min_lumen`: Minimum lumen value across all columns
        3. `max_lumen`: Maximum lumen value across all columns
        4. `angular_range`: Tuple representing the range of angles where lumen values fall within one
        standard deviation
        """

        df_lumen = df.copy()

        # Step 1: Calculate luminous flux (in lumens)
        for idx, col in enumerate(df.columns[1:]):
            df_lumen[col] = df[col] * solid_angles[idx]
            if debug == "True":
                print(f"col: {col} , idx value: {solid_angles[idx]}")  # debug

        # Step 2: Calculate mean and max lumen per column
        mean_lumen_per_column = df_lumen.max()
        if debug == "True":
            print(f"Mean lumen per column: {mean_lumen_per_column}")
            print(
                f"Average lumen for first 23 columns: {mean_lumen_per_column[1:23].mean()}")

        # Step 3: Find the max and min of the lumen values across all columns
        max_lumen_per_col = [df_lumen[col].max()
                             for col in df_lumen.columns[1:]]
        max_lumen = round(max(max_lumen_per_col), 2)

        min_lumen_per_col = [df_lumen[col].min()
                             for col in df_lumen.columns[1:]]
        min_lumen = round(min(min_lumen_per_col), 2)

        if debug == "True":
            print(f"Lumen range: {min_lumen} lm - {max_lumen} lm")

        # Step 4: Calculate the angular range within one standard deviation of the lumen distribution
        # Exclude the first column which contains angles
        lumen_values = np.array(mean_lumen_per_column[1:])  # CUDA OPTIMIZATION
        mean_lumen = np.mean(lumen_values)
        std_lumen = np.std(lumen_values)

        # adding half of the standard deviation
        lower_bound = mean_lumen - (std_lumen)
        # adding half of the standard deviation
        upper_bound = mean_lumen + (std_lumen)

        if debug == "True":
            print(f"Mean lumen: {mean_lumen}")
            print(f"Standard deviation of lumen: {std_lumen}")
            print(
                f"Lumen range within one standard deviation: ({lower_bound}, {upper_bound})")

        # Find the range of angles where the lumen values fall within the first deviation
        selected_angles = []
        for i, lumen_value in enumerate(lumen_values):
            if lower_bound <= lumen_value <= upper_bound:
                # Extract the angle from column name (CXX)
                angle = int(df.columns[i + 1][1:])
                selected_angles.append(angle)

        if selected_angles:
            min_angle = min(selected_angles)
            max_angle = max(selected_angles)
            angular_range = (min_angle, max_angle)
        else:
            angular_range = (None, None)

        if debug == "True" and angular_range != (None, None):
            print(
                f"Angular range within one standard deviation: {angular_range[0]} - {angular_range[1]} degrees")

        # Step 5: Return the DataFrame and relevant data
        return df_lumen, min_lumen, max_lumen, angular_range, round(lower_bound, 2), round(upper_bound, 2), round(mean_lumen, 2)

    def evaluateLumenParallel(self, df, solid_angles, debug="False"):
        """
        The function `evaluateLumenParallel` takes a DataFrame, solid angles, and a debug flag as input,
        calculates various lumen-related metrics, and returns the modified DataFrame along with min/max
        lumen values, angular range within one standard deviation, and mean lumen.
        
        """

        df_lumen = df.copy()
        solid_angles = list(zip(*solid_angles))
        solid_angles = np.array(solid_angles)
        # Check if the length of `solid_angles` matches the number of columns (excluding the first column)
        # num_columns = len(df.columns) - 1  # Exclude the angle column

        # Step 1: Calculate luminous flux (in lumens)
        for idx, col in enumerate(df.columns[1:]):
            df_lumen[col] = df[col] * solid_angles[idx]
            if debug == "True":
                print(f"col: {col} , idx value: {solid_angles[idx]}")  # debug

        # Step 2: Calculate mean and max lumen per column
        mean_lumen_per_column = df_lumen.max()
        if debug == "True":
            print(f"Mean lumen per column: {mean_lumen_per_column}")
            print(
                f"Average lumen for first 23 columns: {mean_lumen_per_column[1:23].mean()}")

        # Step 3: Find the max and min of the lumen values across all columns
        max_lumen_per_col = [df_lumen[col].max()
                             for col in df_lumen.columns[1:]]
        max_lumen = round(max(max_lumen_per_col), 2)

        min_lumen_per_col = [df_lumen[col].min()
                             for col in df_lumen.columns[1:]]
        min_lumen = round(min(min_lumen_per_col), 2)

        if debug == "True":
            print(f"Lumen range: {min_lumen} lm - {max_lumen} lm")

        # Step 4: Calculate the angular range within one standard deviation of the lumen distribution
        # Exclude the first column which contains angles
        lumen_values = np.array(mean_lumen_per_column[1:])  # CUDA OPTIMIZATION
        mean_lumen = np.mean(lumen_values)
        std_lumen = np.std(lumen_values)

        lower_bound = mean_lumen - (std_lumen)
        upper_bound = mean_lumen + (std_lumen)

        if debug == "True":
            print(f"Mean lumen: {mean_lumen}")
            print(f"Standard deviation of lumen: {std_lumen}")
            print(
                f"Lumen range within one standard deviation: ({lower_bound}, {upper_bound})")

        # Find the range of angles where the lumen values fall within the first deviation
        selected_angles = []
        for i, lumen_value in enumerate(lumen_values):
            if lower_bound <= lumen_value <= upper_bound:
                # Extract the angle from column name (CXX)
                angle = int(df.columns[i + 1][1:])
                selected_angles.append(angle)

        if selected_angles:
            min_angle = min(selected_angles)
            max_angle = max(selected_angles)
            angular_range = (min_angle, max_angle)
        else:
            angular_range = (None, None)

        if debug == "True" and angular_range != (None, None):
            print(
                f"Angular range within one standard deviation: {angular_range[0]} - {angular_range[1]} degrees")

        # Return the DataFrame and relevant data
        return df_lumen, min_lumen, max_lumen, angular_range, round(lower_bound, 2), round(upper_bound, 2), round(mean_lumen, 2)


class Simulate:
    def __init__(self, gridParams=[(-30, 30, 101), (-30, 30, 101)], label="Simulation", lights=None, light_coords=None, render_mode="matplotlib"):
        """
        This Python class defines a simulation object with methods for setting grid parameters, running
        the simulation with different render modes, and displaying simulation results.
        
        :param gridParams: The `gridParams` parameter in the `__init__` method is a list containing two
        tuples. Each tuple represents the parameters for defining a grid. The first tuple contains the
        parameters for the x-grid, and the second tuple contains the parameters for the y-grid. Each tuple
        has three elements:
        :param label: The `label` parameter in the `__init__` method of the class is used to set the name
        of the simulation. It is a string parameter that defaults to "Simulation" if not provided when
        creating an instance of the class, defaults to Simulation (optional)
        :param lights: a list of Light objects
        :param light_coords: The `light_coords` parameter in the `__init__` method of the class is used to
        store the coordinates of the lights in the simulation. These coordinates are used in the
        `RunSimulation` method to simulate the light distribution on the grid.
        :param render_mode: The `render_mode` parameter in the code snippet you provided is a parameter
        that specifies the mode in which the simulation results will be rendered. The possible values for
        `render_mode` in this code are:
        1.matplotlib
        2.CUDA
        3.Altair
        """

        self.xgrid = gridParams[0]  # x_grid
        self.ygrid = gridParams[1]  # y grid
        self.grid_divisions = gridParams[0][2]
        self.name = label
        self.angles = np.radians(np.arange(0, 181, 1))
        self.lights = lights if lights is not None else []
        self.lights_labels=[light.label for light in self.lights]
        self.light_coords = light_coords if light_coords is not None else {}
        self.light_path_names = [
            light.photometric_map_path for light in self.lights]
        self.lightFlag = np.all(self.light_path_names ==
                                self.light_path_names[0])
        self.render_mode = render_mode

    def __del__(self):
        print(f"Simulation object deleted")

    def SetXgrid(self, new_xgrid):
        self.xgrid = new_xgrid

    def SetYgrid(self, new_ygrid):
        self.ygrid = new_ygrid
        
    def SetGridDivisions(self,grid_divisions):
        self.grid_divisions = grid_divisions
    
    def getXgrid(self):
        return self.xgrid

    def getYgrid(self):
        return self.ygrid

    def getLightsObject(self):
        return self.lights

    def getLightPathNames(self):
        return self.light_path_names

    def getLightFlag(self):
        return self.lightFlag

    def getGridDivisions(self):
        return self.grid_divisions

    def getGridPace(self):
        return round((self.xgrid[1] - self.xgrid[0])/(self.getGridDivisions()-1),2)

    def SetRenderMode(self, new_render_mode):
        self.render_mode = new_render_mode

    def getRenderMode(self):
        return self.render_mode

    def getProprierties(self):
        print(f"Objects: {self.lights_labels}")
        print(f"Path Names {self.getLightPathNames()}")
        print(f"Light Flag {self.getLightFlag()}")
        print(f"Grid: {self.getXgrid(),self.getYgrid()} m^2")
        print(f"Grid Divisions {self.getGridDivisions()} points")
        print(f"Grid Pace: {self.getGridPace()} m ")
        print(f"Render Mode: {self.getRenderMode()}")
    
    def RunSimulation(self):
        start_time = time.time()
        results = []
        for light in self.lights:
            coords = self.light_coords.get(light, [0, 0, light.getHeight()])
            x_grid, y_grid, I_grid = light.SimGridTensorFlow(
                self.getXgrid(), self.getYgrid(), coords, light.photometric_map)
            results.append((x_grid, y_grid, I_grid))

        # Process results
        x_grid, y_grid, I_grid = results[0]

        # Combine results from multiple lights
        multiple_lights = Light.SumMultipleGrid(
            *[1] + [result[2] for result in results])
        print("Sim multiple grids:", multiple_lights)

        if self.getRenderMode() == "matplotlib":
            # Create a figure with specified dimensions
            fig = plt.figure(figsize=(20, 20))
            ax1 = Create2DProjection(
                fig, x_grid, y_grid, multiple_lights, self.getLightsObject()[0].getHeight())
            plt.tight_layout()
            plt.show()

        # integration with openGL
        elif self.getRenderMode() == "CUDA":
            try:
                import cupy
                print("The library 'cupy' is installed.")
                
                from GPUIntegration.CUDA import Create2DProjectionCUDA
                
                # Create a figure with specified dimensions
                fig = plt.figure(figsize=(20, 20))
                ax1 = Create2DProjectionCUDA(
                    fig, x_grid, y_grid, multiple_lights, self.getLightsObject[0].getHeight())
                plt.tight_layout()
                plt.show()
                
            except ImportError:
                print("CUDA is not supported. Please choose a different method.")
                exit()

            
        elif self.getRenderMode() == "Altair":

            chart = Create2DProjectionAltair(
                x_grid, y_grid, multiple_lights, self.getLightsObject[0].getHeight())
            chart.show()

        else:
            print("Visualization disabled")
            
        end_time = time.time()
        print(f"Simulation time: {round(end_time - start_time, 2)} s")
        return results

# Function to read and prepare the data


def loadFromCSV(file_path, delimiter=";"):
    
    # The code snippet is reading a CSV file located at `file_path` using pandas, replacing commas with
    # periods in the dataframe, converting the values in the dataframe to numeric data types, and handling
    # any errors by coercing them to NaN values. Finally, it returns the modified dataframe.
        

    df = pd.read_csv(file_path, delimiter=delimiter)
    df.replace(',', '.', regex=True, inplace=True)
    df = df.apply(pd.to_numeric, errors='coerce')
    return df


# Function to create the polar plot
def CreatePolarGraph(df, angles):
    """
    The function `CreatePolarGraph` creates a polar graph with data from a DataFrame using specified
    angles.

    :param df: a DataFrame containing the data to be plotted on the polar graph.
    :param angles: represents the angles at which the data points will be plotted on the polar graph. 
    These angles are typically in radians and determine the position of each data point around 
    the circle in the polar plot
    :return: returns a matplotlib figure (`fig`) and an axis object
    (`ax1`) with a polar projection that displays a polar graph based on the input data frame (`df`) and
    angles provided. The graph represents the photometric map with multiple columns plotted on the same
    polar axis.
    """
    fig = plt.figure(figsize=(14, 4))

    ax1 = fig.add_subplot(131, projection='polar')
    for col in df.columns[1:]:
        ax1.plot(angles, df[col].values, label=col)
        ax1.fill(angles, df[col].values, alpha=0.1)

    ax1.set_theta_zero_location('S')
    ax1.set_theta_direction(-1)
    ax1.set_xticks(np.radians(
        [0, 30, 60, 90, 120, 150, 180, 210, 240, 270, 300, 330, 360]))
    ax1.set_xticklabels(['0°', '30°', '60°', '90°', '120°',
                        '150°', '180°', '150°', '120°', '90°', '60°', '30°', '0°'])
    ax1.set_yticks(np.arange(0, 651, 130))
    ax1.set_yticklabels([f'{i}' for i in np.arange(0, 651, 130)])
    ax1.set_rlabel_position(0)
    ax1.set_title("Photometric Map")

    return fig, ax1

# Function to create the polar heatmap


def CreateHeatmap(fig, df, angles):
    
    """
    The function `CreateHeatmap` creates a polar heatmap using the provided DataFrame and angles on a
    given figure.

    :param fig: a matplotlib figure object where the polar heatmap will be added as a subplot. 
    :param df: a DataFrame containing the data that will be used to create the heatmap
    :param angles: represents the angles at which the data points are plotted on the polar heatmap. 
    These angles are used to create the radial lines on the polar plot. 
    :return: returns the subplot `ax2` which is a polar heatmap added to
    the provided figure `fig`.
    """

    ax2 = fig.add_subplot(132, projection='polar')

    matrix = df[df.columns[1:]].T.values
    angle_matrix, r_matrix = np.meshgrid(angles, np.arange(matrix.shape[0]))

    c = ax2.pcolormesh(angle_matrix, r_matrix, matrix,
                       cmap='plasma', shading='auto')
    fig.colorbar(c, ax=ax2, orientation='horizontal',
                 label="luminous intensity (cd)")

    ax2.set_theta_zero_location('S')
    ax2.set_theta_direction(-1)
    ax2.set_xticks(np.radians(
        [0, 30, 60, 90, 120, 150, 180, 210, 240, 270, 300, 330, 360]))
    ax2.set_xticklabels(['0°', '30°', '60°', '90°', '120°',
                        '150°', '180°', '150°', '120°', '90°', '60°', '30°', '0°'])
    ax2.set_yticks(np.arange(0, matrix.shape[0], int(matrix.shape[0]/5)))
    ax2.set_yticklabels([f'{i}' for i in np.arange(
        0, matrix.shape[0], int(matrix.shape[0]/5))])
    ax2.set_rlabel_position(0)
    ax2.set_title("Polar Heatmap")

    return ax2

# Function to create the 2D illuminance plot


def Create2DProjection(fig, x_grid, y_grid, I_grid, h, center_x=0, center_y=0, max_distance=None):
    """
    The function `Create2DProjection` generates a 2D projection of illuminance distribution on a road
    plane with isolux contours and radial distance labels.

    :param fig: figure object that the plot will be added to.
    :param x_grid: The `x_grid` parameter in the `Create2DProjection` function represents the grid of
    x-coordinates where the illuminance values are calculated and plotted.
    :param y_grid: The `y_grid` parameter in the `Create2DProjection` function represents the grid of
    y-coordinates in the 2D projection. 
    :param I_grid: I_grid represents the illuminance distribution on the road plane at a certain height
    from the ground.
    :param h: The parameter `h` represents the height at which the illuminance distribution is being
    plotted on the road plane. It is specified in meters from the ground level
    :param center_x: The `center_x` parameter in the `Create2DProjection` function represents the
    x-coordinate of the center point from which radial lines are drawn to calculate distances to each
    point in the grid. This center point is used as the reference point for calculating distances and
    displaying labels on the plot, defaults is 0 (optional)
    :param center_y: represents the y-coordinate of the center point from which radial lines are 
    drawn to calculate distances in the 2D projection. 
    :param max_distance: The `max_distance` parameter in the `Create2DProjection` function is used to
    specify the maximum distance from the center point (defined by `center_x` and `center_y`) to
    consider when drawing radial lines and adding distance labels on the plot.
    :return: returns the plot `ax` that is added to the provided
    `fig` object after plotting the illuminance distribution, isolux contours, radial lines from the
    center, and distance labels on the plot.
    """
    start_time = time.time()
    # Create an axis in the figure
    ax = fig.add_axes([0.3, 0.25, 0.5, 0.5])  # [left, bottom, width, height]

    # Plot the illuminance distribution as a colormap
    c = ax.pcolormesh(x_grid, y_grid, I_grid, cmap='binary_r',
                      shading='auto', vmin=0, vmax=50)
    fig.colorbar(c, ax=ax, label='Illuminance (lux)')

    contour_levels = np.arange(0, 300, 5)

    contours = ax.contour(
        x_grid, y_grid, I_grid, levels=contour_levels, colors='yellow', linewidths=1.0)
    ax.clabel(contours, inline=True, fontsize=8,
              fmt='%d lux', colors='yellow')

    # Draw radial lines from the center and add labels for distances
    if max_distance is None:
        max_distance = np.max(
            np.sqrt((x_grid - center_x)**2 + (y_grid - center_y)**2))

    # Calculate distances from the center to each point in the grid
    distance_grid = np.sqrt((x_grid - center_x)**2 + (y_grid - center_y)**2)

    # Add distance labels next to the isolux contours
    for level in contour_levels:
        contour = ax.contour(x_grid, y_grid, I_grid, levels=[
            level], colors='red', linewidths=1.0)
        # For each curve, find a point to label
        for collection in contour.collections:
            for path in collection.get_paths():
                # Find a point on the path (use the first point for simplicity)
                point = path.vertices[len(path.vertices)//2]
                # Calculate distance from the center
                distance = np.sqrt(
                    (point[0] - center_x)**2 + (point[1] - center_y)**2)
                ax.text(point[0], point[1], f'{distance:.1f} m',
                        color='white', fontsize=9, ha='center', va='center')

    # Set labels and title
    ax.set_xlabel('X (meters)')
    ax.set_ylabel('Y (meters)')
    ax.set_title(
        f"Illuminance Distribution on the Road Plane at height = {h} m from the ground")

    
    end_time = time.time()
    print(f"Selected Plotting Mode: Matplotlib")
    print(f"Elapsed time: {round(end_time - start_time, 2)} s")
    return ax

#plotting with altair:
def Create2DProjectionAltair(x_grid, y_grid, I_grid, h, center_x=0, center_y=0, max_distance=None):
    """
    The `Create2DProjection` function generates a 2D projection of the lighting distribution on a street plane
    with isolux contours and radial distance labels. It uses Altair for plotting.

    :param x_grid: x coordinate grid.
    :param y_grid: y coordinate grid.
    :param I_grid: Illuminance grid.
    :param h: Height at which the illuminance distribution is plotted.
    :param center_x: x coordinate of the center (default: 0)
    :param center_y: Y coordinate of the center (default: 0)
    :param max_distance: Maximum distance for radial lines (default: None)
    :return: Altair graph of illuminance distribution.
    """
    start_time = time.time()

    # converts the datas in a pandas dataframe
    data = pd.DataFrame({
        'x': x_grid.ravel(),
        'y': y_grid.ravel(),
        'illuminance': I_grid.ravel()
    })

    # Create the heatmap using Altair
    heatmap = alt.Chart(data).mark_rect().encode(
        x=alt.X('x:Q', title='X (meters)'),
        y=alt.Y('y:Q', title='Y (meters)'),
        color=alt.Color('illuminance:Q', scale=alt.Scale(scheme='greys'), title='Illuminance (lux)')
    ).properties(
        width=600,
        height=600,
        title=f"Illuminance Distribution on the Road Plane at height = {h} m from the ground"
    )

    # saves the heatmap
    file_name = 'illuminance_chart.html'
    heatmap.save(file_name)

    end_time = time.time()
    print(f"Selected Plotting Mode: Altair")
    print(f"Elapsed time: {round(end_time - start_time, 2)} s")
    print(f"Grafico salvato come '{file_name}'")

    return heatmap


# overload 2D Projection for making only 1 plot

def CalculateSolidAngle(df, threshold=180):
    """
    The function CalculateSolidAngle calculates the solid angle for local maximum angles in a DataFrame
    based on intensity values and a specified threshold.

    :param df: a DataFrame `df` as input, which contains intensity values and corresponding angles. 
    The function calculates the solid angle for each unique local maximum angle in the intensity values 
    based on a specified threshold
    :param threshold: The `threshold` parameter in the `CalculateSolidAngle` function is used to
    determine the minimum angle difference between two points to consider them as separate peaks. It is
    specified in degrees and is used to calculate the `order` parameter for finding local maxima in the
    intensity values. The `order`, defaults to 180 (optional)
    :return: The function `CalculateSolidAngle` is returning the last calculated solid angle for the
    unique local maximum angle in the input DataFrame `df`.
    """

    # Initialize a list to store solid angles
    solid_angles = []
    # Get the column names excluding 'val'
    intensity_columns = df.columns[1:]

    # Iterate over each column of intensities in the DataFrame
    for col in intensity_columns:
        # Get the intensity values for the current column
        intensities = df[col].values
        angles = df['val'].values

        # Find local maxima
        # `order` is the number of points on each side to consider for a peak
        order = int(threshold / (angles[1] - angles[0]))
        # print(f"Order for local maxima (based on threshold {threshold} degrees): {order}")

        local_maxima_indices = argrelextrema(
            intensities, np.greater, order=order)[0]
        # print(f"Local maxima indices: {local_maxima_indices}")

        # Filter unique local maxima angles
        unique_local_maxima_angles = np.unique(angles[local_maxima_indices])
        # print(f"Unique local maxima angles: {unique_local_maxima_angles}")

        # Calculate solid angle for each local maximum angle
        for angle in unique_local_maxima_angles:
            # Convert the angle to radians
            angle_rad = np.deg2rad(angle)

            # Calculate the solid angle for the angle
            solid_angle = 2 * np.pi * (1 - np.cos(angle_rad))
            # print(f"Angle: {angle} degrees, Angle (radians): {angle_rad}, Solid angle: {solid_angle}")

    return solid_angle


def CalculateSolidAngleMonteCarlo(df, num_samples=1000000, vertical_angle=100, debug=False):
    """
    The function CalculateSolidAngleMonteCarlo calculates the solid angle for a cone based on Monte
    Carlo sampling of intensity values within a specified vertical angle range.

    :param df: takes a DataFrame `df` as input, which
    contains intensity values and corresponding angles. 
    :param num_samples: the number of random points to generate on a sphere for Monte Carlo integration.
    The default value is set to 1000000
    :param vertical_angle: parameter represents the vertical angle in degrees
    within which the Monte Carlo simulation will be performed. 
    :param debug: The `debug` parameter in the `CalculateSolidAngleMonteCarlo` function is a boolean
    flag that controls whether debug information is printed during the calculation process. If
    `debug=True`, the function will print out information such as the column being processed.
    :return: The function `CalculateSolidAngleMonteCarlo` is returning the calculated solid angle based 
    on the given vertical angle in radians.
    """

    # Initialize a list to store solid angles
    solid_angles = []

    # Convert the fixed vertical angle to radians
    vertical_angle_rad = np.deg2rad(vertical_angle)

    # Get the column names excluding 'val'
    intensity_columns = df.columns[1:]

    # Iterate over each column of intensities in the DataFrame
    for col in intensity_columns:
        # Generate random points on a sphere (using spherical coordinates)
        phi = np.random.uniform(0, 2 * np.pi, num_samples)  # Azimuthal angle
        # Polar angle (constrained to vertical_angle)
        theta = np.random.uniform(0, vertical_angle_rad, num_samples)

        # Convert spherical coordinates to Cartesian coordinates
        x = np.sin(theta) * np.cos(phi)
        y = np.sin(theta) * np.sin(phi)
        z = np.cos(theta)

        # Calculate the fraction of points that fall within the desired angular range
        intensities = df[col].values
        angles = df['val'].values

        # Find the corresponding intensity values for the sampled angles
        sampled_intensities = np.interp(theta, np.deg2rad(angles), intensities)

        # Normalize intensities to create a probability distribution
        normalized_intensities = sampled_intensities / \
            np.sum(sampled_intensities)

        # Calculate the solid angle using the given formula
        # Solid angle for a cone is 2π(1 - cos(θ/2)), where θ is the vertical angle
        solid_angle = 2 * np.pi * (1 - np.cos(vertical_angle_rad / 2))

        if debug:
            print(
                f"Column: {col}, Vertical Angle: {np.rad2deg(vertical_angle_rad)}, Solid Angle: {solid_angle}")

    return solid_angle


####################################################################################################################################################
####################################################################################################################################################
####################### PARALLELIZATION ############################################################################################################
####################################################################################################################################################
####################################################################################################################################################

def CalculateSolidAngleForColum(col, df, num_samples, vertical_angle_rad, debug=False):
    """
    The function calculates the solid angle for a given column in a DataFrame using Monte Carlo sampling
    on a sphere.
    :param df: takes a DataFrame `df` as input, which contains intensity values and corresponding angles. 
    :param num_samples: the number of random points to generate on a sphere for Monte Carlo integration.
    The default value is set to 1000000
    :param vertical_angle: parameter represents the vertical angle in degrees
    within which the Monte Carlo simulation will be performed. 
    :param debug: The `debug` parameter in the `CalculateSolidAngleMonteCarlo` function is a boolean
    flag that controls whether debug information is printed during the calculation process. If
    `debug=True`, the function will print out information such as the column being processed.
    :return: The function `CalculateSolidAngleMonteCarlo` is returning the calculated solid angle based 
    on the given vertical angle in radians.
    """

    # Generate random points on the sphere (spherical coordinates)
    phi = np.random.uniform(0, 2 * np.pi, num_samples)  # Azimuthal angle
    theta = np.random.uniform(0, vertical_angle_rad,
                              num_samples)  # Polar angle

    # Convert spherical coordinates to Cartesian coordinates
    x = np.sin(theta) * np.cos(phi)
    y = np.sin(theta) * np.sin(phi)
    z = np.cos(theta)

    # Extract intensity and angle values from the DataFrame
    intensities = df[col].values
    angles = df['val'].values

    # Interpolate intensities for sampled angles
    sampled_intensities = np.interp(theta, np.deg2rad(angles), intensities)

    # Normalize intensities to create a probability distribution
    normalized_intensities = sampled_intensities / np.sum(sampled_intensities)

    # Calculate the solid angle using the given formula
    solid_angle = 2 * np.pi * (1 - np.cos(vertical_angle_rad / 2))

    if debug:
        print(
            f"Column: {col}, Vertical Angle: {np.rad2deg(vertical_angle_rad)}, Solid Angle: {solid_angle}")

    return solid_angle


def CalculateSolidAngleMonteCarloParallel(df, num_samples=1000000, vertical_angle=100, debug=False, n_jobs=-1):
    """
    The function CalculateSolidAngleMonteCarloParallel calculates solid angles for each column in a
    DataFrame using Monte Carlo simulation in parallel.
    
    :param df: takes a DataFrame `df` as input, which contains intensity values and corresponding angles. 
    :param num_samples: the number of random points to generate on a sphere for Monte Carlo integration.
    The default value is set to 1000000
    :param vertical_angle: parameter represents the vertical angle in degrees
    :param debug: If `debug=True`, you may see more detailed output or logging to help with
    troubleshooting or understanding the calculation steps.
    :param n_jobs: The `n_jobs` parameter in the `CalculateSolidAngleMonteCarloParallel` function
    specifies the number of parallel jobs to run. Setting `n_jobs=-1` will use all available CPU cores
    for parallel processing. This can help speed up the calculation process by distributing the workload
    across multiple cores
    :return: The function `CalculateSolidAngleMonteCarloParallel` returns a list of solid angles
    calculated for each column in the input DataFrame `df`, based on the specified parameters
    """

    # Convert the fixed vertical angle to radians
    vertical_angle_rad = np.deg2rad(vertical_angle)

    # Get column names excluding 'val'
    intensity_columns = df.columns[1:]

    # Parallelize the calculation using joblib
    solid_angles = Parallel(n_jobs=n_jobs)(
        delayed(CalculateSolidAngleForColum)(
            col, df, num_samples, vertical_angle_rad, debug)
        for col in intensity_columns
    )
    # Convert every element in a list if not
    solid_angles = [item if isinstance(item, list) else [
        item] for item in solid_angles]

    # flatten list
    solid_angles = [item for sublist in solid_angles for item in sublist]
    return solid_angles


# Main Function
if __name__ == "__main__":

    # Load photometric data
    df = loadFromCSV("./Datasets/LED9W.csv")

    # List to store solid angles
    sAng = []

    # Calculate solid angle
    start_time = time.time()
    solid_angle = CalculateSolidAngleMonteCarloParallel(df)
    sAng.append(solid_angle)
    #print(sAng)
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
              photometric_map=df, solid_angles=sAng, label="Light 3"),

        Light(position=[45.800043, 8.952930, 4], power=9,
              orientation_angle=290, diffusion_angle=60,
              photometric_map=df, solid_angles=sAng, label="Light 4"),

        Light(position=[45.800043, 8.952930, 4], power=9,
              orientation_angle=290, diffusion_angle=60,
              photometric_map=df, solid_angles=sAng, label="Light 5"),

        Light(position=[45.800043, 8.952930, 4], power=9,
              orientation_angle=290, diffusion_angle=60,
              photometric_map=df, solid_angles=sAng, label="Light 6"),

        Light(position=[45.800043, 8.952930, 4], power=9,
              orientation_angle=290, diffusion_angle=60,
              photometric_map=df, solid_angles=sAng, label="Light 7"),

        Light(position=[45.800043, 8.952930, 4], power=9,
              orientation_angle=290, diffusion_angle=60,
              photometric_map=df, solid_angles=sAng, label="Light 8"),

        Light(position=[45.800043, 8.952930, 4], power=9,
              orientation_angle=290, diffusion_angle=60,
              photometric_map=df, solid_angles=sAng, label="Light 9"),

        Light(position=[45.800043, 8.952930, 4], power=9,
              orientation_angle=290, diffusion_angle=60,
              photometric_map=df, solid_angles=sAng, label="Light 10"),

        Light(position=[45.800043, 8.952930, 4], power=9,
              orientation_angle=290, diffusion_angle=60,
              photometric_map=df, solid_angles=sAng, label="Light 11"),

        Light(position=[45.800043, 8.952930, 4], power=9,
              orientation_angle=290, diffusion_angle=60,
              photometric_map=df, solid_angles=sAng, label="Light 12"),

        Light(position=[45.800043, 8.952930, 4], power=9,
              orientation_angle=290, diffusion_angle=60,
              photometric_map=df, solid_angles=sAng, label="Light 13"),

        Light(position=[45.800043, 8.952930, 4], power=9,
              orientation_angle=290, diffusion_angle=60,
              photometric_map=df, solid_angles=sAng, label="Light 14"),

        Light(position=[45.800043, 8.952930, 4], power=9,
              orientation_angle=290, diffusion_angle=60,
              photometric_map=df, solid_angles=sAng, label="Light 15"),

        Light(position=[45.800043, 8.952930, 4], power=9,
              orientation_angle=290, diffusion_angle=60,
              photometric_map=df, solid_angles=sAng, label="Light 16"),
        Light(position=[45.800043, 8.952930, 4], power=9,
              orientation_angle=290, diffusion_angle=60,
              photometric_map=df, solid_angles=sAng, label="Light 17")
    ]

    # for light in lights:
    #   light.getStatus()

    # Define coordinates for each light
    light_coords = {
        lights[0]: [0, 0, lights[0].getHeight()],
        lights[1]: [10, 0, lights[1].getHeight()],
        lights[2]: [20, 0, lights[2].getHeight()],
        lights[3]: [30, 0, lights[3].getHeight()],
        lights[4]: [40, 0, lights[4].getHeight()],
        lights[5]: [-10, 0, lights[5].getHeight()],
        lights[6]: [-20, 0, lights[6].getHeight()],
        lights[7]: [-30, 0, lights[7].getHeight()],
        lights[8]: [-40, 0, lights[8].getHeight()],
        lights[9]: [0, 20, lights[9].getHeight()],
        lights[10]: [10, 20, lights[10].getHeight()],
        lights[11]: [20, 20, lights[11].getHeight()],
        lights[12]: [30, 20, lights[12].getHeight()],
        lights[13]: [40, 20, lights[13].getHeight()],
        lights[14]: [-10, 20, lights[14].getHeight()],
        lights[15]: [-20, 20, lights[15].getHeight()],
        lights[16]: [-30, 20, lights[16].getHeight()],
        lights[17]: [-40, 20, lights[17].getHeight()]

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
