import unittest

import numpy as np
from numpy.testing import assert_allclose
from scipy.spatial.transform import Rotation

from ..turtle import Turtle


class TurtleTests(unittest.TestCase):
    def test_initial_position(self):
        turtle = Turtle()
        assert_allclose(turtle.position, (0, 0, 0))
        # Provide tuple, list, np.ndarray
        turtle = Turtle(position=(1, 1, 1))
        assert_allclose(turtle.position, (1, 1, 1))

        turtle = Turtle(position=[2, 2, 2])
        assert_allclose(turtle.position, (2, 2, 2))

        turtle = Turtle(position=np.array([3, 3, 3]))
        assert_allclose(turtle.position, (3, 3, 3))

    def test_initial_rotation(self):
        turtle = Turtle()
        assert_allclose(turtle.rotation.as_matrix(), np.eye(3))

        # A rotation 180 degrees about the global x axis should flip both the y and z axes.
        rotation = Rotation.from_euler("x", [np.pi])
        expected = [[1, 0, 0], [0, -1, 0], [0, 0, -1]]
        turtle = Turtle(rotation=rotation)
        # There's a little bit of numerical instability in the works.
        assert_allclose(turtle.rotation.as_matrix(), [expected], atol=1e-15)

    def test_forward(self):
        turtle = Turtle()
        assert_allclose(turtle.position, (0, 0, 0))
        turtle.forward()
        assert_allclose(turtle.position, (0, 0, 1))
        turtle.forward(2)
        assert_allclose(turtle.position, (0, 0, 3))

    def test_rotated_forward(self):
        # Apparently this rotates CW
        rotation = Rotation.from_euler("y", [np.pi / 2])
        turtle = Turtle(rotation=rotation)
        turtle.forward(2)
        assert_allclose(turtle.position, (2, 0, 0), atol=1e-15)

    def test_roll(self):
        turtle = Turtle()
        assert_allclose(turtle.position, (0, 0, 0))
        turtle.roll(45)
        # roll, pitch, yaw don't effect position.
        assert_allclose(turtle.position, (0, 0, 0))
        turtle.forward()
        assert_allclose(turtle.position, (0, -np.sqrt(2) / 2, np.sqrt(2) / 2))
        # Another roll will level out the turtle, leaving it to travel one unit in the -y direction
        turtle.roll(45)
        turtle.forward()
        assert_allclose(turtle.position, (0, -np.sqrt(2) / 2 - 1, np.sqrt(2) / 2))

    def test_pitch(self):
        turtle = Turtle()
        turtle.pitch(45)
        turtle.forward()
        assert_allclose(turtle.position, (np.sqrt(2) / 2, 0, np.sqrt(2) / 2))
        turtle.pitch(45)
        turtle.forward()
        assert_allclose(turtle.position, (1 + np.sqrt(2) / 2, 0, np.sqrt(2) / 2))

    def test_yaw(self):
        # The initial turtle orientation is (0, 0, 1), so we need to jump through hoops to test.
        turtle = Turtle()
        turtle.yaw(45)
        turtle.roll(45)
        turtle.forward()
        assert_allclose(turtle.position, (0.5, -0.5, np.sqrt(2) / 2))

    def test_initial_rotation_roll(self):
        rotation = Rotation.from_euler('x', [45], degrees=True)
        turtle = Turtle(rotation=rotation)
        turtle.forward()
        assert_allclose(turtle.position, (0, -np.sqrt(2) / 2, np.sqrt(2) / 2))
        turtle.roll(45)
        turtle.forward()
        assert_allclose(turtle.position, (0, -np.sqrt(2) / 2 - 1, np.sqrt(2) / 2))

    def test_initial_rotation_pitch(self):
        rotation = Rotation.from_euler('y', [45], degrees=True)
        turtle = Turtle(rotation=rotation)
        turtle.forward()
        assert_allclose(turtle.position, (np.sqrt(2) / 2, 0, np.sqrt(2) / 2))
        turtle.pitch(45)
        turtle.forward()
        assert_allclose(turtle.position, (1 + np.sqrt(2) / 2, 0, np.sqrt(2) / 2))
