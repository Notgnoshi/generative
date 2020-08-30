#!/usr/bin/env python3
"""Interpret an L-String as a set of 3D Turtle commands and record the turtle's path.

Multiple lines of input will be treated as a continuation of a single L-String.

TODO: Describe the L-String commands.
"""
import argparse
import logging
import pathlib
import sys

root = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(root))
from lsystem.interpreter import LSystemInterpeter  # isort:skip


LOG_LEVELS = {
    "CRITICAL": logging.CRITICAL,
    "ERROR": logging.ERROR,
    "WARNING": logging.WARNING,
    "INFO": logging.INFO,
    "DEBUG": logging.DEBUG,
}
DEFAULT_LEVEL = "WARNING"


def parse_args():
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )

    parser.add_argument(
        "input",
        nargs="?",
        type=argparse.FileType("r"),
        default=sys.stdin,
        help="A file containing the L-String to interpret. Defaults to stdin.",
    )
    parser.add_argument(
        "output",
        nargs="?",
        # TODO: I seem to not be able to open stdout in binary mode.
        # See: https://github.com/python/cpython/pull/13165
        # Potential workaround: open in 'wb' mode, and default to sys.stdout.buffer.
        type=argparse.FileType("w"),
        default=sys.stdout,
        help="A file to output the expanded axiom to. Defaults to stdout.",
    )
    parser.add_argument(
        "--commandset",
        "-c",
        type=str,
        default="default",
        choices=LSystemInterpeter.commandsets,
        help="The commandset to use to interpret the given L-String. Defaults to 'default'.",
    )
    parser.add_argument(
        "--format",
        "-f",
        type=str,
        default="wkt",
        # TODO: Decide if WKB will actually output raw binary, or hex/base64 encoded.
        # TODO: Does WKB require binary stdout?
        choices=["wkt", "wkb"],
        help="The output format for the turtle path. Defaults to WKT.",
    )
    parser.add_argument(
        "-l",
        "--log-level",
        type=str,
        default=DEFAULT_LEVEL,
        choices=LOG_LEVELS.keys(),
        help=f"Set the logging output level. Defaults to {DEFAULT_LEVEL}.",
    )

    return parser.parse_args()


def main(args):
    logger.debug(f"args: {args}")

    interpreter = LSystemInterpeter(args.commandset)
    tokens = interpreter.tokenize(args.input)
    lines = interpreter.interpret(tokens)
    interpreter.serialize(lines, output=args.output, format=args.format)


if __name__ == "__main__":
    args = parse_args()

    logging.basicConfig(
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
        level=LOG_LEVELS.get(args.log_level),
        stream=sys.stderr,
    )
    logger = logging.getLogger(name="interpreter.py")

    main(args)
