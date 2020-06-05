import abc


class GraphicsWriter(abc.ABC):
    """A graphics abstraction layer for drawing L-Systems.

    TODO: Define the dimensionality in the constructor?
    TODO: Define the graphics abstraction (line segment?) (Dimension agnostic?)
    TODO: Consider using kd-trees or R-trees to simplify the drawings? (At least the SVG writer).
    This might have to be the responsibility of the interpreter? But then it can't be done on the fly?
    TODO: Provide an interface to serialize/deserialize binary data to avoid re-interpreting.
    """
