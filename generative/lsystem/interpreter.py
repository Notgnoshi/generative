import io
import logging
from typing import Iterable, NewType

import numpy as np
from shapely.geometry import LineString

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
        d,D - Turn drawing off, on
        [,] - Push, pop position and orientation onto a stack
    """

    commandsets = frozenset(["default"])

    def __init__(self, commandset, stepsize, angle):
        """Initialize an L-System interpreter with the given commandset and turtle config."""
        if commandset not in self.commandsets:
            raise ValueError(f"{commandset=} not in {self.commandsets}.")
        self.commandset = commandset
        self.turtle = Turtle()
        self.stepsize = stepsize
        self.angle = angle
        self.drawing = True
        self.orientation_changed = False
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
        for line in self._interpret(tokens):
            if line is not None:
                yield line

    def _interpret(self, tokens: Tokens) -> Lines:
        if self.commandset == "default":
            yield from self._interpret_default(tokens)
        else:
            raise ValueError(f"commandset '{self.commandset}' unsupported")

    def _flush_active_line(self) -> LineString:
        if not self.active_line or not self.drawing:
            return None

        self._append_position()
        if len(self.active_line) < 2:
            logger.error(f"Tried to flush incomplete line {self.active_line}")
            return None

        line = LineString(self.active_line)
        self.active_line = []

        logger.debug(f"Flushing active line {line.wkt}")
        return line

    def _append_position(self):
        # If any coordinate isn't equal to the last recorded position.
        if self.drawing and (
            not self.active_line or np.any(self.active_line[-1] != self.turtle.position)
        ):
            self.active_line.append(self.turtle.position)

    def _interpret_default(self, tokens: Tokens):
        for token in tokens:
            # Step forward and draw
            if token in {"F", "G"}:
                if self.drawing and (len(self.active_line) == 0 or self.orientation_changed):
                    logger.debug(
                        "Making first step forwards since last flush or orientation change. pos: %s",
                        self.turtle.position,
                    )
                    self.orientation_changed = False
                    self.active_line.append(self.turtle.position)
                self.turtle.forward(self.stepsize)
            # Step forward without drawing
            elif token in {"f", "g"}:
                yield self._flush_active_line()
                self.turtle.forward(self.stepsize)
            elif token == "-":
                self.orientation_changed = True
                self.turtle.yaw(-self.angle)
            elif token == "+":
                self.orientation_changed = True
                self.turtle.yaw(+self.angle)
            elif token == "v":
                self.orientation_changed = True
                self.turtle.pitch(-self.angle)
            elif token == "^":
                self.orientation_changed = True
                self.turtle.pitch(+self.angle)
            elif token == "<":
                self.orientation_changed = True
                self.turtle.roll(-self.angle)
            elif token == ">":
                self.orientation_changed = True
                self.turtle.roll(+self.angle)
            elif token == "|":
                self.orientation_changed = True
                # TODO: Determine if we should also roll 180deg.
                self.turtle.yaw(180)
            # Turn drawing off
            elif token == "d":
                yield self._flush_active_line()
                self.drawing = False
            # Turn drawing back on
            elif token == "D":
                self.drawing = True
            elif token == "[":
                self.stack.append((self.turtle.position, self.turtle.rotation))
                logger.debug("pushing turtle position, orientation.")
            elif token == "]":
                yield self._flush_active_line()
                logger.debug("popping turtle position, orientation.")
                if not self.stack:
                    logger.warning("Stack empty. Can't pop.")
                else:
                    self.turtle.position, self.turtle.rotation = self.stack.pop()
        yield self._flush_active_line()
