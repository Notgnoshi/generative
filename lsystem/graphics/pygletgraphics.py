import pyglet

from .graphics import GraphicsWriter


# TODO: 2D _and_ 3D, automatically detect and configure the camera accordingly
# TODO: GUI, non-GUI. Nongui should save final PNG renderings, creation, and rotation animation
class PygletWriter(GraphicsWriter):
    """An OpenGL L-System renderer.

    TODO: Handle 2D _and_ 3D systems.
    TODO: In the case of 2D systems, adjust the camera angle automagically.
    TODO: Provide option to save renderings. Where to place the camera?
    TODO: Provide option to animate the L-System creation.
    TODO: Provide option to save animation of the system growth and creation.
    Pyglet doesn't support this other than saving each frame and stiching together after the fact.
    TODO: Provide option to save animation of the final product rotating about an axis.
    TODO: Provide GUI and headless modes.
    TODO: GUI should be interactive, pan, scan, zoom.
    TODO: Does Pyglet support GUI forms?
    TODO: GUI should (optionally) show the growth animation.
    TODO: GUI should (optionally) show the rotation animation.
    """
