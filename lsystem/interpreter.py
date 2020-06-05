from .graphics import GraphicsWriter
from .turtle import Turtle


class LSystemInterpeter:
    """Interpret L-System strings as turtle commands.

    Will need to carefully define the L-System language.
    Note that https://github.com/Objelisks/lsystembot uses placebo letters that don't correspond to
    drawing primitives, but still allow for grammatical transformations.

    Will need to carefully define the format handed off to GraphicsWriter objects. I think the
    interpreter should stream the line segments (along with associated metadata like thickness and
    color) to the graphics writers, and let the graphics writers decide if they stream to their
    respective outputs, or if they cache the results and perform additional operations.
    """
