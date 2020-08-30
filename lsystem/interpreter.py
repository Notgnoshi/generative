from .turtle import Turtle


class LSystemInterpeter:
    """Interpret L-System strings as turtle commands.

    Will need to carefully define the L-System language.
    Note that https://github.com/Objelisks/lsystembot uses placebo letters that don't correspond to
    drawing primitives, but still allow for grammatical transformations.
    """
