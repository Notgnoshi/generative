#!/usr/bin/env python3
"""Resize/Rescale geometries.

Resize
    Set the size of one or more dimensions.
    If only one dimension is set, the rest will be scaled in order to preserve the aspect ratio.
    Units may be specified in 'px' (default), 'in', or 'cm'.

Rescale
    Scale up or down one or more dimensions.
    If only one dimension is specified, the others will be scaled by the same amount to preserve
    the aspect ratio. To get a scale in only one dimensions, specify a scale factor of 1.0 for the
    others.

Sizes and scales may be mixed and matched. E.g., you can set the scale in one dimension, and a size
in another.
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

    # TODO: Need to work with wkt2svg.py to make this _acutally_ PPI.
    parser.add_argument("--ppi", type=int, default=72, help="Pixels per inch. Defaults to 72")

    group = parser.add_mutually_exclusive_group()
    group.add_argument(
        "--sizex",
        "-dx",
        type=str,
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
        "--sizey",
        "-dy",
        type=str,
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
        "--sizez",
        "-dz",
        type=str,
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


def to_pixels(size: str, ppi: int):
    """Parse the size strings from the commandline into a common unit."""


def parse_sizes(args):
    return (
        to_pixels(args.sizex, args.ppi),
        to_pixels(args.sizey, args.ppi),
        to_pixels(args.sizez, args.ppi),
    )


def main(args):
    geometries = deserialize_geometries(args.input, args.format)
    tagged_points = flatten(geometries)
    points, tags = unzip(tagged_points)

    # TODO: Load all of the points into memory, assuming 2D until a 3D point is encountered.
    # If a 3D point is encountered, previous 2D points should be zero-padded.

    # TODO: Find the axis-aligned bounding box for the geometries.

    # TODO: Find the scale factors needed if any resizes are requested.

    # TODO: Rescale by elementwise multiplication by a 3-tuple of scale factors.

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
