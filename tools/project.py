#!/usr/bin/env python3
"""Project 3D WKT/WKB to 2D."""
import argparse
import logging
import pathlib
import sys

root = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(root))
from lsystem import deserialize_geometries, serialize_geometries  # isort:skip
from lsystem import project  # isort:skip

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
        "--kind",
        "-k",
        default="pca",
        choices=["xy", "xz", "yz", "pca", "I"],
        help="What kind of projection to use. Defaults to using PCA to pick a 2D basis.",
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
    logger.debug(args)
    geometries = deserialize_geometries(args.input, args.format)
    transformed = project(geometries, args.kind)
    serialize_geometries(transformed, args.output, args.format)


if __name__ == "__main__":
    args = parse_args()
    logging.basicConfig(
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
        level=LOG_LEVELS.get(args.log_level),
        stream=sys.stderr,
    )
    logger = logging.getLogger(name="project.py")

    main(args)
