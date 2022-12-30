#!/usr/bin/env python3
"""Project 3D WKT/WKB to 2D."""
import argparse
import logging
import pathlib
import sys

root = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(root))
from generative.flatten import flatten, unflatten
from generative.io import (
    deserialize_flat,
    deserialize_geometries,
    serialize_flat,
    serialize_geometries,
)
from generative.projection import project

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
        "--input-format",
        "-I",
        default="wkt",
        choices=["wkt", "wkb", "flat"],
        help="The input geometry format. Defaults to WKT. Use 'flat' for better performance.",
    )
    parser.add_argument(
        "--output-format",
        "-O",
        default="wkt",
        choices=["wkt", "wkb", "flat"],
        help="The output geometry format. Defaults to WKT. Use 'flat' for better performance.",
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
    if args.input_format != "flat":
        geometries = deserialize_geometries(args.input, args.input_format)
        tagged_points = flatten(geometries)
    else:
        tagged_points = deserialize_flat(args.input)
    transformed_points = project(tagged_points, args.kind, args.dimensions, args.scale)

    if args.output_format != "flat":
        transformed_geoms = unflatten(transformed_points)
        serialize_geometries(transformed_geoms, args.output, args.output_format)
    else:
        serialize_flat(transformed_points, args.output)


if __name__ == "__main__":
    args = parse_args()
    logging.basicConfig(
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
        level=LOG_LEVELS.get(args.log_level),
        stream=sys.stderr,
    )
    logger = logging.getLogger(name=__file__)

    main(args)
