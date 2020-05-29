import numpy as np
from scipy.spatial.transform import Rotation


class Turtle:
    """A turtle object that keeps track of its position and rotation in 3D space.

    All angles given in degrees.
    """

    def __init__(self, position: np.ndarray = None, rotation: Rotation = None):
        """Initialize the turtle with the given position and rotation.

        Use a traditional RHS coordinate system with Z pointing up.
        Transformations are performed using intrinsic Euler angles, so rotations about an axis are
        relative to the turtle's local reference frame.

        :param position: The starting position of the turtle. Defaults to (0, 0, 0).
        :param rotation: The starting rotation of the turtle, applied to the vector (0, 0, 1).
        """
        if rotation is not None and not isinstance(rotation, Rotation):
            raise TypeError("Rotation must be a scipy.spatial.transform.Rotation")

        self._position = np.array(position) if position is not None else np.array([[0, 0, 0]])
        self.rotation = rotation if rotation is not None else Rotation.from_matrix(np.eye(3))

    @property
    def position(self):
        """Ensure the position is externally always treated as (3,), not (1, 3)."""
        return self._position.reshape((3,))

    @position.setter
    def position(self, value):
        self._position = np.array(value)

    def forward(self, stepsize=1):
        """Move the turtle forward by the given stepsize."""
        orientation = (0, 0, 1)
        orientation = self.rotation.apply(orientation)
        self.position = self.position + stepsize * orientation

    def yaw(self, angle):
        """Yaw the turtle around its local Z axis."""
        # NOTE: Capital axes indicate intrinsic Euler angles.
        self.rotation = self.rotation * Rotation.from_euler("Z", [angle], degrees=True)

    def pitch(self, angle):
        """Pitch the turtle around its local Y axis."""
        self.rotation = self.rotation * Rotation.from_euler("Y", [angle], degrees=True)

    def roll(self, angle):
        """Roll the turtle around its local X axis."""
        self.rotation = self.rotation * Rotation.from_euler("X", [angle], degrees=True)
