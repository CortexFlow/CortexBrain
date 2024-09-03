from BaseSensor import Sensor
from Map import LightMap
import math

import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
from luxpy import iolidfiles as iolid
from scipy.signal import argrelextrema


# The `Light` class represents a smart light sensor with properties such as position, power, lumen,
# height, diffusion angle, and orientation angle, along with methods to get and set these properties
# and compute the maximum range covered by the light sensor.

# Add Photometric Curves --->coming soon
# Improvements in the max range covered --> coming soon


class Light(Sensor):
    def __init__(self, position, power, diffusion_angle, orientation_angle, photometric_map, solid_angles, label="Smart Light"):
        super().__init__(SensorType="Light", value=[0.0, 0.0], label=label)
        self.lat = float(position[0])
        self.lon = float(position[1])
        self.power = power
        self.lumen, self.min_lumen, self.max_lumen, self.angular_range, self.lumen_lower_bound,self.lumen_upper_bound,self.mean_lumen = Light.evaluateLumen(self, photometric_map, solid_angles)
        self.label = label
        self.height = float(position[2])
        self.theta = diffusion_angle
        self.orientation = orientation_angle

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
        return [self.lumen_lower_bound, self.mean_lumen,self.lumen_upper_bound]

    def getPower(self):
        return self.power

    def getLightEfficiency(self):
        return round(self.light_efficiency,2)

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
        return round(np.max(self.lumen),2)

    def getStatus(self):
        """
        The `getStatus` function prints the current status of a light sensor with various attributes such as
        name, coordinates, power, lumen, height, diffusion angle, orientation angle, max range covered, and
        light efficiency.
        """
        """Prints the current status of the light sensor."""
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
        print("-----------------------------")

    # Function to calculate the intensity in candelas at a point (x, y, z)
    def evaluateE(self, x, y, z, df, debug="False"):
        """
        This Python function evaluates the illuminance at a given point based on the distance to a lamp and
        the intensity data from a CSV file.

        :param x: The `x` parameter represents the x-coordinate of the point where you want to evaluate the
        illuminance. If you have any more questions or need further assistance with the code, feel free to
        ask!
        :param y: The `y` parameter in the `evaluateE` function represents the y-coordinate of the point
        where you want to evaluate the illuminance. This function calculates the illuminance at a specific
        point in space based on the distance to a lamp and the intensity of the light emitted by the lamp at
        a given
        :param z: The parameter `z` in the `evaluateE` function represents the z-coordinate of the point
        where you want to evaluate the illuminance. This function calculates the illuminance at a specific
        point in space based on the distance to a lamp and the intensity of the light emitted by the lamp at
        a given
        :param x_lamp: The `x_lamp` parameter represents the x-coordinate of the lamp's position in 3D
        space. It is used in the function to calculate the distance from the point of interest to the lamp
        :param y_lamp: The `y_lamp` parameter in the `evaluateE` function represents the y-coordinate of the
        position of the lamp in a 3D space. This parameter is used to calculate the distance between the
        point of interest and the lamp in the y-direction
        :param z_lamp: The `z_lamp` parameter represents the vertical position of the lamp in the coordinate
        system. It is used in the function `evaluateE` to calculate the distance from a point to the lamp in
        3D space
        :param theta_lamp: Theta_lamp represents the angle at which the lamp is positioned. It is used to
        determine the intensity of light emitted by the lamp at that specific angle
        :param horizontal_angle: The `horizontal_angle` parameter in the `evaluateE` function represents the
        angle at which the lamp is positioned horizontally. This angle is used to look up the corresponding
        intensity value from the provided DataFrame (`df`) to calculate the illuminance at a specific point
        in space relative to the lamp's position
        :param df: The `df` parameter in the `evaluateE` function is likely a pandas DataFrame that contains
        intensity values for different horizontal angles. The function checks if the `horizontal_angle` is
        present in the columns of the DataFrame and retrieves the corresponding intensity value at the index
        specified by `theta_lamp`. If
        :return: The function `evaluateE` returns the illuminance in lux, which is calculated based on the
        distance from the point to the lamp and the corresponding intensity from the CSV file.
        """
        # Calculate the distance from the point to the lamp
        x_lamp = 0
        y_lamp = 0
        z_lamp = self.height
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

    # Function to create the 2D grid and calculate the intensity for each point
    def SimGrid(self, x_range, y_range, df):
        """
        The function SimGrid generates a grid of points and calculates illuminance values at each point
        based on given parameters and a specified evaluation function.

        :param x_range: The `x_range` parameter likely represents the range of x values for the grid
        :param y_range: The `y_range` parameter in the `SimGrid` function represents the range of values for
        the y-coordinate in the grid. It is used to create a linearly spaced array of y values within the
        specified range. The `np.linspace(*y_range)` function call generates an array of y values
        :param x_lamp: The `x_lamp` parameter in the `SimGrid` function represents the x-coordinate of the
        lamp position in the simulation grid. It is used in the calculation of illuminance at each point on
        the grid by passing it to the `evaluateE` function along with other parameters like `y_l
        :param y_lamp: The `y_lamp` parameter in the `SimGrid` function represents the y-coordinate of the
        lamp position in the simulated grid. It is used in the calculation of illuminance at each point on
        the grid by passing it along with other parameters to the `evaluateE` function
        :param z_lamp: The `z_lamp` parameter represents the height of the lamp above the road plane in the
        `SimGrid` function. It is used in the calculation of illuminance at each point on the grid relative
        to the lamp position
        :param theta_lamp: Theta_lamp represents the vertical angle of the lamp in degrees. It is used in
        the evaluateE function to calculate the illuminance at a specific point on the grid based on the
        lamp's position and orientation
        :param horizontal_angle: The `horizontal_angle` parameter in the `SimGrid` function likely
        represents the horizontal angle at which the light source (lamp) is positioned relative to the grid.
        This angle can be used to calculate the illuminance at different points on the grid based on the
        position and orientation of the lamp
        :param df: The `df` parameter in the `SimGrid` function seems to be a variable that is used in the
        `evaluateE` function to calculate the illuminance at a specific point on the grid. It is likely that
        `df` contains some data or parameters necessary for the calculation of illuminance,
        :return: The function `SimGrid` returns three arrays: `x_grid`, `y_grid`, and `I_grid`. `x_grid` and
        `y_grid` represent the grid points in the x and y directions, respectively, while `I_grid` stores
        the illuminance values calculated for each point on the grid.
        """

        x = np.linspace(*x_range)
        y = np.linspace(*y_range)
        z = 0  # Road plane

        x_grid, y_grid = np.meshgrid(x, y)
        # Create an empty matrix to store the illuminance
        I_grid = np.zeros_like(x_grid)

        # Iterate manually over each point in the grid
        for i in range(x_grid.shape[0]):
            for j in range(y_grid.shape[1]):
                I_grid[i, j] = Light.evaluateE(
                    self, x_grid[i, j], y_grid[i, j], z, df)

        return x_grid, y_grid, I_grid

    def evaluateLumen(self, df, solid_angles, debug="False"):
        """
        Evaluate the luminous flux (in lumens) for each column in the DataFrame using the corresponding solid angles.
        Additionally, calculate the angular range where the lumen values fall within one standard deviation from the mean.

        Args:
            df (pd.DataFrame): DataFrame of luminous intensities (in candelas).
            solid_angles (array-like): Array of corresponding solid angles (in steradians).
            debug (str): If "True", prints debugging information.

        Returns:
            tuple: A tuple containing the DataFrame of luminous flux (in lumens), 
                the minimum and maximum lumen values, 
                and the angular range within one standard deviation from the mean.
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
        lumen_values = np.array(mean_lumen_per_column[1:])
        mean_lumen = np.mean(lumen_values)
        std_lumen = np.std(lumen_values)

        lower_bound = mean_lumen - (std_lumen)#adding half of the standard deviation
        upper_bound = mean_lumen + (std_lumen) #adding half of the standard deviation

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
        return df_lumen, min_lumen, max_lumen, angular_range, round(lower_bound,2), round(upper_bound,2),round(mean_lumen,2)

# Function to read and prepare the data


def loadFromCSV(file_path, delimiter=";"):
    """
    The function `loadFromCSV` reads a CSV file, replaces commas with periods, converts the data to
    numeric format, and returns a pandas DataFrame.

    :param file_path: The `file_path` parameter in the `loadFromCSV` function is a string that
    represents the file path of the CSV file that you want to load and process. This parameter should be
    the location of the CSV file on your system
    :param delimiter: The `delimiter` parameter in the `loadFromCSV` function specifies the character
    used to separate fields in the CSV file. By default, the delimiter is set to ";", but you can
    specify a different delimiter if your CSV file uses a different character to separate values in each
    row, defaults to ; (optional)
    :return: The function `loadFromCSV` reads a CSV file located at the `file_path`, using the specified
    `delimiter` (default is ";"). It then replaces commas with periods in the data, converts the data to
    numeric format, and handles any conversion errors by coercing them. Finally, it returns the
    processed DataFrame `df`.
    """
    df = pd.read_csv(file_path, delimiter=delimiter)
    df.replace(',', '.', regex=True, inplace=True)
    df = df.apply(pd.to_numeric, errors='coerce')
    return df


# Function to create the polar plot
def CreatePolarGraph(df, angles):
    """
    The function `CreatePolarGraph` creates a polar graph with data from a DataFrame using specified
    angles.

    :param df: The `df` parameter in the `CreatePolarGraph` function is likely referring to a pandas
    DataFrame that contains the data to be plotted on the polar graph. The function seems to be designed
    to create a polar graph with the data provided in the DataFrame
    :param angles: The `angles` parameter in the `CreatePolarGraph` function represents the angles at
    which the data points will be plotted on the polar graph. These angles are typically in radians and
    determine the position of each data point around the center of the polar plot
    :return: The function `CreatePolarGraph` returns a tuple containing the `fig` (matplotlib figure)
    and `ax1` (matplotlib axes) objects that represent a polar graph created using the input DataFrame
    `df` and angles `angles`.
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
    The function `CreateHeatmap` creates a polar heatmap using the provided DataFrame and angles.

    :param fig: The `fig` parameter in the `CreateHeatmap` function is a matplotlib figure object where
    the polar heatmap will be added as a subplot. This figure object is typically created using
    `plt.figure()` in matplotlib before calling the `CreateHeatmap` function
    :param df: The `df` parameter in the `CreateHeatmap` function is likely a pandas DataFrame that
    contains the data to be visualized in the heatmap. It seems that the function is designed to create
    a polar heatmap using the data from this DataFrame
    :param angles: The `angles` parameter in the `CreateHeatmap` function represents the angles at which
    the data points are plotted on the polar heatmap. These angles are used to create the radial lines
    on the polar plot where the data values are mapped. The angles parameter is typically a list or
    array of angles in
    :return: The function `CreateHeatmap` returns the subplot `ax2` which is a polar heatmap added to
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
    This function creates a 2D projection of illuminance distribution on a road plane at a specified
    height from the ground, with added isolux contours and radial lines indicating distance from the center.

    :param fig: The figure object to which the subplot will be added.
    :param x_grid: The grid of x-coordinates in meters for the 2D projection.
    :param y_grid: The grid of y-coordinates in meters for the 2D projection.
    :param I_grid: The illuminance values at different points on the grid (2D array).
    :param h: The height at which the illuminance distribution is being projected (in meters).
    :param center_x: The x-coordinate of the center of the light source.
    :param center_y: The y-coordinate of the center of the light source.
    :param max_distance: The maximum distance to draw the radial lines (default is the maximum distance in the grid).
    :return: The subplot `ax3` displaying the 2D projection with isolux contours and radial lines.
    """
    ax3 = fig.add_subplot(133)

    # Plot the illuminance distribution as a colormap
    c = ax3.pcolormesh(x_grid, y_grid, I_grid, cmap='binary_r',
                       shading='auto', vmin=0, vmax=50)
    fig.colorbar(c, ax=ax3, label='Illuminance (lux)')

    # Add isolux contours
    if 0 < light.getHeight() <= 2:
        # Define levels for the contour lines
        contour_levels = np.arange(0, 30, 5)
    elif 2 < light.getHeight() <= 6:
        # Define levels for the contour lines
        contour_levels = np.arange(0, 30, 1)
    else:
        # Define levels for the contour lines
        contour_levels = np.arange(0, 30, 1)

    contours = ax3.contour(
        x_grid, y_grid, I_grid, levels=contour_levels, colors='yellow', linewidths=1.0)
    ax3.clabel(contours, inline=True, fontsize=8,
               fmt='%d lux', colors='yellow')

    # Draw radial lines from the center and add labels for distances
    if max_distance is None:
        max_distance = np.max(
            np.sqrt((x_grid - center_x)**2 + (y_grid - center_y)**2))

    # Calculate distances from the center to each point in the grid
    distance_grid = np.sqrt((x_grid - center_x)**2 + (y_grid - center_y)**2)

    # Add distance labels next to the isolux contours
    for level in contour_levels:
        contour = ax3.contour(x_grid, y_grid, I_grid, levels=[
                              level], colors='red', linewidths=1.0)
        # For each curve, find a point to label
        for collection in contour.collections:
            for path in collection.get_paths():
                # Find a point on the path (use the first point for simplicity)
                point = path.vertices[len(path.vertices)//2]
                # Calculate distance from the center
                distance = np.sqrt(
                    (point[0] - center_x)**2 + (point[1] - center_y)**2)
                ax3.text(point[0], point[1], f'{distance:.1f} m',
                         color='white', fontsize=9, ha='center', va='center')

    # Set labels and title
    ax3.set_xlabel('X (meters)')
    ax3.set_ylabel('Y (meters)')
    ax3.set_title(
        f"Illuminance Distribution on the Road Plane at height = {h} m from the ground")

    return ax3


def CalculateSolidAngle(df, threshold=180):
    """
    Calculate the solid angles in steradians based on local maxima within specified angular intervals.

    Args:
        df (pd.DataFrame): DataFrame with angles and columns of intensities.
        threshold (float): Angular interval in degrees for identifying local maxima.

    Returns:
        np.ndarray: Solid angles in steradians for each column.
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
    Calculate the solid angles in steradians using Monte Carlo sampling, with a fixed vertical angle.

    Args:
        df (pd.DataFrame): DataFrame with angles and columns of intensities.
        num_samples (int): Number of random samples to generate for Monte Carlo simulation.
        vertical_angle (float): The fixed vertical angle in degrees (e.g., zenith angle for a hemisphere).
        debug (bool): If True, prints debug information.

    Returns:
        np.ndarray: Solid angles in steradians for each column.
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
        theta = np.random.uniform(0, vertical_angle_rad, num_samples)  # Polar angle (constrained to vertical_angle)
        
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
        normalized_intensities = sampled_intensities / np.sum(sampled_intensities)
        
        # Calculate the solid angle using the given formula
        # Solid angle for a cone is 2π(1 - cos(θ/2)), where θ is the vertical angle
        solid_angle = 2 * np.pi * (1 - np.cos(vertical_angle_rad / 2))
        
        if debug:
            print(f"Column: {col}, Vertical Angle: {np.rad2deg(vertical_angle_rad)}, Solid Angle: {solid_angle}")
        
        
    return np.array(solid_angle)


# Main Function
if __name__ == "__main__":
    # Parameters and configurations

    x_range = (-30, 30, 100)
    y_range = (-30, 30, 100)

    val = np.arange(0, 181, 1)
    angles = np.radians(val)

    df = loadFromCSV("./Datasets/LED6W.csv")
    # List to store solid angles
    sAng = []

    # Calculate solid angle for each column (except the first one)
    for col in df.columns[1:]:
        solid_angle = CalculateSolidAngleMonteCarlo(df)
        sAng.append(solid_angle)

    # Convert list to NumPy array
    sAng = np.array(sAng)
    print(sAng)
    # Create a Light object

    light = Light(position=[45.800043, 8.952930, 8], power=9,
                  orientation_angle=290, diffusion_angle=60, photometric_map=df, solid_angles=sAng, label="Light 1")

    # Get the light sensor status
    light.getStatus()

    # Calculate the grid and illuminance
    x_grid, y_grid, I_grid = light.SimGrid(x_range, y_range, df)

    # Create the plots
    fig, ax1 = CreatePolarGraph(df, angles)
    ax2 = CreateHeatmap(fig, df, angles)
    ax3 = Create2DProjection(fig, x_grid, y_grid, I_grid, light.getHeight())

    plt.tight_layout()
    plt.show()

""" 
    # Create a LightMap object and add the light sensor
    map = LightMap()
    map.addSensor(light)
    map.CreateMap()
 """
