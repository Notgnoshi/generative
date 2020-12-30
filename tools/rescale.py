#!/usr/bin/env python3
"""Resize/Rescale geometries.

If only one dimension is set, the rest will be scaled in order to respect the aspect ratio.
"""
import argparse
import logging
import pathlib
import sys

root = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(root))
from generative.wkio import deserialize_geometries, serialize_geometries  # isort:skip
from generative.projection import flatten, unflatten, unzip  # isort:skip

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
        help="The WKT/WKB input to parse. Defaults to stdin.",
    )
    parser.add_argument(
        "--output",
        "-o",
        type=argparse.FileType("w"),
        default=sys.stdout,
        help="Where to output the WKT/WKB. Defaults to stdout.",
    )
    parser.add_argument(
        "--format",
        "-f",
        default="wkt",
        choices=["wkt", "wkb"],
        help="The input and output format. Defaults to WKT.",
    )
    parser.add_argument(
        "-l",
        "--log-level",
        type=str,
        default=DEFAULT_LEVEL,
        choices=LOG_LEVELS.keys(),
        help=f"Set the logging output level. Defaults to {DEFAULT_LEVEL}.",
    )

    # TODO: Units?
    group = parser.add_mutually_exclusive_group()
    group.add_argument(
        "--dimx",
        "-dx",
        type=float,
        default=None,
        help="The size in the x dimension",
    )
    group.add_argument(
        "--scalex",
        "-sx",
        type=float,
        default=None,
        help="The scale in the x dimension",
    )

    group = parser.add_mutually_exclusive_group()
    group.add_argument(
        "--dimy",
        "-dy",
        type=float,
        default=None,
        help="The size in the y dimension",
    )
    group.add_argument(
        "--scaley",
        "-sy",
        type=float,
        default=None,
        help="The scale in the y dimension",
    )

    group = parser.add_mutually_exclusive_group()
    group.add_argument(
        "--dimz",
        "-dz",
        type=float,
        default=None,
        help="The size in the z dimension",
    )
    group.add_argument(
        "--scalez",
        "-sz",
        type=float,
        default=None,
        help="The scale in the z dimension",
    )

    return parser.parse_args()


def main(args):
    geometries = deserialize_geometries(args.input, args.format)
    tagged_points = flatten(geometries)
    points, tags = unzip(tagged_points)
    # TODO: Rescale/resize
    tagged_points = zip(points, tags)
    transformed_geoms = unflatten(tagged_points)
    serialize_geometries(transformed_geoms, args.output, args.format)


if __name__ == "__main__":
    args = parse_args()
    logging.basicConfig(
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
        level=LOG_LEVELS.get(args.log_level),
        stream=sys.stderr,
    )
    logger = logging.getLogger(name=__file__)

    main(args)
