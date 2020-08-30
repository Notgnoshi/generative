import io
from typing import Iterable, NewType

from shapely.geometry import LineString

from .turtle import Turtle

Token = NewType("Token", str)
Tokens = Iterable[Token]
Lines = Iterable[LineString]


class LSystemInterpeter:
    """Interpret L-System strings as turtle commands.

    Will need to carefully define the L-System language.
    Note that https://github.com/Objelisks/lsystembot uses placebo letters that don't correspond to
    drawing primitives, but still allow for grammatical transformations.
    """

    commandsets = frozenset(["default"])

    def __init__(self, commandset):
        if commandset not in self.commandsets:
            raise ValueError(f"{commandset=} not in {self.commandsets}.")
        self.commandset = commandset

    def tokenize(self, input: io.TextIOWrapper) -> Tokens:
        """Tokenize the given input using the configured commandset."""
        return []

    def interpret(self, tokens: Tokens):
        """Interpret the given tokens as 3D Turtle commands."""
        return []

    def serialize(self, lines: Lines, output: io.TextIOWrapper, format: str):
        """Serialize the turtle's path in the given format.

        :param output: The output buffer to write the turtle's path to.
        :param format: One of 'wkt', 'wkb'.

        TODO: Use hex, base64, or raw bytes for wkb?
        TODO: Use something other than io.TextIOWrapper for WKB.
        """
        if format != "wkt":
            raise ValueError(f"{format=} is unsupported. Use 'wkt'")
