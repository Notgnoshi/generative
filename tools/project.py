#!/usr/bin/env python3
"""Project 3D WKT/WKB to 2D."""
import argparse
import logging
import pathlib
import sys

root = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(root))
from generative.wkio import deserialize_geometries, serialize_geometries  # isort:skip
from generative.projection import flatten, project, unflatten  # isort:skip

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
    # TODO: Should I specify both the input and output formats separately?
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
    parser.add_argument(
        "--kind",
        "-k",
        default="pca",
        choices=["xy", "xz", "yz", "pca", "svd", "I", "isometric", "auto"],
        help="What kind of projection to use. Defaults to using PCA to pick a 2D basis.",
    )
    parser.add_argument(
        "--dimensions",
        "-n",
        type=int,
        default=2,
        choices=[2, 3],
        help="The target dimensionality for the PCA, SVD, or Isometric projections.",
    )
    parser.add_argument(
        "--scale", "-s", type=float, default=1, help="A multiplicative scale factor"
    )

    return parser.parse_args()


def main(args):
    logger.debug(args)
    geometries = deserialize_geometries(args.input, args.format)
    tagged_points = flatten(geometries)
    transformed_points = project(tagged_points, args.kind, args.dimensions, args.scale)
    transformed_geoms = unflatten(transformed_points)
    serialize_geometries(transformed_geoms, args.output, args.format)


if __name__ == "__main__":
    args = parse_args()
    logging.basicConfig(
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
        level=LOG_LEVELS.get(args.log_level),
        stream=sys.stderr,
    )
    logger = logging.getLogger(name="project.py")

    main(args)
