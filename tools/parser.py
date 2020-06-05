#!/usr/bin/env python3
"""Given a set of Lindenmayer System production rules, iteratively apply them to a starting axiom.

TODO: Define the grammar syntax, file format. The file format should be able to include things
for future stages, like the turn angle, colors?, thicknesses?, randomness?
"""
import argparse
import json
import pathlib
import sys

root = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(root))
from lsystem.grammar import LSystemGrammar  # isort:skip


def parse_args():
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )

    parser.add_argument(
        "-c",
        "--config",
        type=argparse.FileType("r"),
        default=None,
        help="The JSON config file as described above. Uses stdin if '-' is given. Commandline arguments, if given, will override the values in the config file.",
    )
    parser.add_argument(
        "output",
        nargs="?",
        type=argparse.FileType("w"),
        default=sys.stdout,
        help="A file to output the expanded axiom to. Defaults to stdout.",
    )
    parser.add_argument(
        "-r",
        "--rule",
        type=str,
        action="append",
        help="Add the whitespace sensitive production rule. Multiple '--rule's can be given. Uses the same syntax as the config file.",
    )
    parser.add_argument(
        "-a",
        "--axiom",
        type=str,
        nargs=1,
        help="The starting axiom to apply the production rules to.",
    )
    parser.add_argument(
        "-n", "--iterations", type=int, nargs=1, help="The number of iterations to run."
    )
    parser.add_argument(
        "--seed",
        type=int,
        default=None,
        help="The random seed to use for stochastic grammars. If not given, one will be chosen and printed to stderr.",
    )

    return parser.parse_args()


def main(args):
    print(args)


if __name__ == "__main__":
    args = parse_args()
    # TODO: Read the config file, and override options from args.
    main(args)
