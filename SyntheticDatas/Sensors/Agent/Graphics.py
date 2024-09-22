import math as meth
import numpy as np
import pygame
from pygame.locals import *

pygame.init()

class LIGHT:
    def __init__(self, size, pixel_shader):
        self.size = size
        self.radius = size * 0.5
        self.render_surface = pygame.Surface((size, size))
        self.pixel_shader_surf = pixel_shader.copy()
        self.baked_pixel_shader_surf = pixel_shader.copy()
        self.render_surface.set_colorkey((0, 0, 0))

    def baked_lighting(self, tiles, x, y, reset_surface):
        if reset_surface:
            self.baked_pixel_shader_surf = self.pixel_shader_surf.copy()
        
        dx, dy = x - self.radius, y - self.radius

        for point in self.get_tiles(tiles, x, y):
            corners = self.get_corners(point, x, y)
            corners = [(corner[0] - dx, corner[1] - dy) for corner in corners]
            self.fill_shadows(self.baked_pixel_shader_surf, [
                corners[0],
                corners[1],
                self.get_intersection([self.radius] * 2, corners[1]),
                self.get_intersection([self.radius] * 2, corners[0]),
                corners[2]
            ])

    def get_intersection(self, p1, p2):
        dx = p2[0] - p1[0]
        dy = p2[1] - p1[1]

        if dx == 0:  # Vertical line
            return [p2[0], 0 if dy <= 0 else self.size]
        if dy == 0:  # Horizontal line
            return [0 if dx <= 0 else self.size, p2[1]]

        # Check for intersection with vertical edges
        y_gradient = dy / dx
        y_intercept = p1[1] - (p1[0] * y_gradient)
        y_line = 0 if dx <= 0 else self.size
        y_intersection = [y_line, (y_gradient * y_line) + y_intercept]
        if 0 <= y_intersection[1] <= self.size:
            return y_intersection

        # Check for intersection with horizontal edges
        x_gradient = dx / dy
        x_intercept = p1[0] - (p1[1] * x_gradient)
        x_line = 0 if dy <= 0 else self.size
        x_intersection = [(x_gradient * x_line) + x_intercept, x_line]
        if 0 <= x_intersection[0] <= self.size:
            return x_intersection

        return None  # No intersection found

    def fill_shadows(self, render_surface, points):
        render_points = [points[0], points[4], points[1], points[2], points[3]]

        # Use only necessary calculations
        if points[2][0] + points[3][0] not in (1000, 0) and points[2][1] + points[3][1] not in (1000, 0):
            if abs(points[2][0] - points[3][0]) == self.size:
                if self.radius < points[2][1]:
                    render_points = [points[0], points[4], points[1], points[2], [0, self.size], [self.size, self.size], points[3]]
                elif self.radius > points[2][1]:
                    render_points = [points[0], points[4], points[1], points[2], [self.size, 0], [0, 0], points[3]]
            elif abs(points[2][1] - points[3][1]) == self.size:
                if self.radius < points[2][0]:
                    render_points = [points[0], points[4], points[1], points[2], [self.size, self.size], [self.size, 0], points[3]]
                elif self.radius > points[2][0]:
                    render_points = [points[0], points[4], points[1], points[2], [0, self.size], [0, 0], points[3]]
            else:
                if points[2][0] not in (self.size, 0):
                    render_points = [points[0], points[4], points[1], points[2], [points[3][0], points[2][1]], points[3]]
                else:
                    render_points = [points[0], points[4], points[1], points[2], [points[2][0], points[3][1]], points[3]]

        pygame.draw.polygon(render_surface, (0, 0, 0), render_points)

    def get_corners(self, points, x, y):
        corners = [points[0], points[2], points[2]]

        if points[1][0] <= x <= points[0][0]:
            corners = [points[0], points[1], points[1]] if y < points[1][1] else [points[2], points[3], points[3]]
        if points[0][1] <= y <= points[2][1]:
            corners = [points[1], points[2], points[2]] if x < points[1][0] else [points[0], points[3], points[3]]

        if x < points[1][0] and y < points[1][1]:
            corners = [points[0], points[2], points[1]]
        elif x > points[0][0] and y > points[2][1]:
            corners = [points[0], points[2], points[3]]
        if x > points[0][0] and y < points[1][1]:
            corners = [points[1], points[3], points[0]]
        elif x < points[1][0] and y > points[2][1]:
            corners = [points[1], points[3], points[2]]

        return corners

    def get_tiles(self, tiles, x, y):
        points = []
        for rect in tiles:
            if (-self.radius - rect.width <= rect.x - x <= self.radius) and (-self.radius - rect.height <= rect.y - y <= self.radius):
                points.append([
                    [rect.x + rect.width, rect.y],
                    [rect.x, rect.y],
                    [rect.x, rect.y + rect.height],
                    [rect.x + rect.width, rect.y + rect.height]
                ])
        return points

    def check_cast(self, points, dx, dy):
        for point in points:
            x_index = int(point[0] - dx)
            y_index = int(point[1] - dy)
            
            if not (0 <= x_index < self.pixel_shader_surf.get_width() and
                    0 <= y_index < self.pixel_shader_surf.get_height()):
                #print(f"Out of bounds: ({x_index}, {y_index})")
                continue
            
            if self.pixel_shader_surf.get_at((x_index, y_index)) != (0, 0, 0, 255):
                return True
        return False


    def main(self, tiles, display, x, y):
        self.render_surface.fill((0, 0, 0))
        self.render_surface.blit(self.baked_pixel_shader_surf, (0, 0))

        dx, dy = x - self.radius, y - self.radius

        for point in self.get_tiles(tiles, x, y):
            if self.check_cast(point, dx, dy):
                corners = self.get_corners(point, x, y)
                corners = [(corner[0] - dx, corner[1] - dy) for corner in corners]
                self.fill_shadows(self.render_surface, [
                    corners[0],
                    corners[1],
                    self.get_intersection([self.radius] * 2, corners[1]),
                    self.get_intersection([self.radius] * 2, corners[0]),
                    corners[2]
                ])

        display.blit(self.render_surface, (x - self.radius, y - self.radius), special_flags=BLEND_RGBA_ADD)
        return display

def global_light(size, intensity):
    dark = pygame.Surface(size).convert_alpha()
    dark.fill((255, 255, 255, intensity))
    return dark

def pixel_shader(size, color, intensity, point, angle=0, angle_width=360):
    final_array = np.full((size, size, 3), color, dtype=np.float64)
    radius = size * 0.5

    x, y = np.meshgrid(np.arange(size), np.arange(size))
    distance = np.sqrt((x - radius) ** 2 + (y - radius) ** 2)
    radial_falloff = np.clip((radius - distance) / radius, 0, 1)

    if point:
        point_angle = (180 / np.pi) * -np.arctan2((radius - x), (radius - y)) + 180
        diff_angle = np.abs(((angle - point_angle) + 180) % 360 - 180)
        angular_falloff = np.clip(((angle_width / 2) - diff_angle) / angle_width, 0, 1)
    else:
        angular_falloff = 1

    final_intensity = radial_falloff * angular_falloff * intensity
    final_array *= final_intensity[..., np.newaxis]

    return pygame.surfarray.make_surface(final_array.astype(np.uint8))
