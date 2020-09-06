import io
import logging
from typing import Iterable, NewType

from shapely.geometry import LineString
from shapely import wkb, wkt

from .turtle import Turtle

logger = logging.getLogger(__name__)

Token = NewType("Token", str)
Tokens = Iterable[Token]
Lines = Iterable[LineString]


class LSystemInterpeter:
    """Interpret L-System strings as turtle commands.

    Default commandset:

        F,G - Step forward while drawing
        f,g - Step forward without drawing
        -,+ - Yaw around the normal axis
        v,^ - Pitch around the transverse axis
        <,> - Roll around the longitudinal axis
        |   - Flip orientation 180 degrees
        d,D - Turn drawing on, off
        [,] - Push, pop position and orientation onto a stack
    """

    commandsets = frozenset(["default"])

    def __init__(self, commandset, stepsize, angle):
        if commandset not in self.commandsets:
            raise ValueError(f"{commandset=} not in {self.commandsets}.")
        self.commandset = commandset
        self.turtle = Turtle()
        self.stepsize = stepsize
        self.angle = angle
        self.drawing = True
        self.active_line = []
        self.stack = []

    def tokenize(self, commands: io.TextIOWrapper) -> Tokens:
        """Tokenize the given input using the configured commandset."""
        if self.commandset == "default":
            return self._tokenize_default(commands)
        raise ValueError(f"tokenize not supported (yet) for '{self.commandset}'")

    @staticmethod
    def _tokenize_default(commands: io.TextIOWrapper) -> Tokens:
        # The default set of commands are each a single character.
        # So tokenizing is really easy. Yay.
        while True:
            chunk = commands.read(256)
            if not chunk:
                break
            for char in chunk:
                yield char

    def interpret(self, tokens: Tokens) -> Lines:
        """Interpret the given tokens as 3D Turtle commands."""
        for token in tokens:
            self._interpret_t(token)
            # Record the turtle path
            if self.drawing:
                self.active_line.append(self.turtle.position)
            # We stopped drawing, and we have a line fragment to pass on.
            elif len(self.active_line) > 1:
                line = LineString(self.active_line)
                logger.debug("Finished drawing line.")
                self.active_line = []
                yield line
        if len(self.active_line) > 1:
            line = LineString(self.active_line)
            logger.debug("Ran out of tokens before line finished.")
            return line

    def _interpret_t(self, token):
        logger.debug(f"Interpreting {token=}")
        if self.commandset == "default":
            self._interpret_t_default(token)
        else:
            raise ValueError(f"commandset '{self.commandset}' unsupported")

    def _interpret_t_default(self, token: Token):
        if token in {"F", "G"}:
            self.turtle.forward(self.stepsize)
            self.drawing = True
        elif token in {"f", "g"}:
            self.turtle.forward(self.stepsize)
            self.drawing = False
        elif token == "-":
            self.turtle.yaw(-self.angle)
        elif token == "+":
            self.turtle.yaw(+self.angle)
        elif token == "v":
            self.turtle.pitch(-self.angle)
        elif token == "^":
            self.turtle.pitch(+self.angle)
        elif token == "<":
            self.turtle.roll(-self.angle)
        elif token == ">":
            self.turtle.roll(+self.angle)
        elif token == "|":
            # TODO: Determine if we should also roll 180deg.
            self.turtle.yaw(180)
        elif token == "d":
            self.drawing = True
        elif token == "D":
            self.drawing = False
        elif token == "[":
            self.stack.append((self.turtle.position, self.turtle.rotation))
            logger.debug("pushing turtle position, orientation.")
        elif token == "]":
            logger.debug("popping turtle position, orientation.")
            if not self.stack:
                logger.warning("Stack empty. Can't pop.")
            else:
                self.turtle.position, self.turtle.rotation = self.stack.pop()
            self.drawing = False

    def serialize(self, lines: Lines, output: io.TextIOWrapper, format: str):
        """Serialize the turtle's path in the given format.

        :param output: The output buffer to write the turtle's path to.
        :param format: One of 'wkt', 'wkb'.
        """
        if format == "wkt":
            for line in lines:
                wkt.dump(line, output, trim=True)
                output.write("\n")
            return

        if format == "wkb":
            for line in lines:
                wkb.dump(line, output, hex=True)
                output.write("\n")
            return

        raise ValueError(f"{format=} is unsupported. Use 'wkt' or 'wkb'")
