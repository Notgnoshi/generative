#!/usr/bin/env python3
"""Change geometry formats between WKT, WKB, and tagged points.

The geometries will still be loaded if the input and output formats are the same.
This makes this script useful for filtering out invalid input.
"""
import argparse
import logging
import pathlib
import sys

root = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(root))
from generative.flatten import flatten
from generative.io import (
    deserialize_flat,
    deserialize_geometries,
    serialize_flat,
    serialize_geometries,
)

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
        "--input",
        "-i",
        type=argparse.FileType("r"),
        default=sys.stdin,
        help="",
    )
    parser.add_argument(
        "--output",
        "-o",
        type=argparse.FileType("w"),
        default=sys.stdout,
        help="",
    )
    parser.add_argument(
        "--log-level",
        "-l",
        type=str,
        default=DEFAULT_LEVEL,
        choices=LOG_LEVELS.keys(),
        help=f"Set the logging output level. Defaults to {DEFAULT_LEVEL}.",
    )
    parser.add_argument(
        "--input-format",
        "-I",
        type=str,
        default="wkt",
        choices=["wkt", "wkb", "flat"],
        help="The input geometry format.",
    )
    parser.add_argument(
        "--output-format",
        "-O",
        type=str,
        default="wkt",
        choices=["wkt", "wkb", "flat"],
        help="The output geometry format.",
    )

    return parser.parse_args()


def main(args):
    # Can skip deserialization into geometries.
    if args.input_format == "flat" and args.output_format == "flat":
        tagged_points = deserialize_flat(args.input)
        serialize_flat(tagged_points, args.output)
    else:
        geometries = deserialize_geometries(args.input, args.input_format)
        serialize_geometries(geometries, args.output, args.output_format)


if __name__ == "__main__":
    args = parse_args()

    logging.basicConfig(
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
        level=LOG_LEVELS.get(args.log_level),
        stream=sys.stderr,
    )
    logger = logging.getLogger(name=__file__)

    main(args)
