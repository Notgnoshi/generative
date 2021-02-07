#!/usr/bin/env python3
"""Given a set of Lindenmayer System production rules, iteratively apply them to a starting axiom.

This does _not_ interpret the L-strings with a turtle, only expand the initial axiom.

Config File Format:
    "iterations": <int>,
    "seed": <int>, optional
    "axiom": <str>, comma-separated list of tokens
    "rules": [
        "[left_context <] lhs [> right_context] [: probability] -> rhs[,[...]]",
        "#ignore: tok1,tok2,..."
    ]
"""
import argparse
import logging
import pathlib
import sys
from typing import List, Set, Tuple

import commentjson
from multidict import MultiDict

root = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(root))
from generative.lsystem.grammar import LSystemGrammar, Token, TokenName, RuleMapping  # isort:skip
from generative.lsystem.production import RuleParser  # isort:skip

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
        "--long-tokens",
        "-L",
        action="store_true",
        default=False,
        help="Whether to support comma/whitespace separate long tokens. Otherwise assume single character tokens. Parser output will be space-separated in this mode.",
    )
    parser.add_argument(
        "-r",
        "--rule",
        type=str,
        default=[],
        action="append",
        help="Add the given production rule. Multiple '--rule's can be given. Uses the same syntax as the config file.",
    )
    parser.add_argument(
        "-a",
        "--axiom",
        type=str,
        default=None,
        help="The starting axiom to apply the production rules to.",
    )
    parser.add_argument(
        "-n",
        "--iterations",
        type=int,
        default=None,
        help="The number of iterations to run.",
    )
    parser.add_argument(
        "--seed",
        type=int,
        default=None,
        help="The random seed to use for stochastic grammars. If not given, one will be chosen and printed to stderr.",
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


def parse_rules(
    rules: List[str], long_tokens
) -> Tuple[MultiDict[TokenName, RuleMapping], Set[TokenName]]:
    parser = RuleParser(long_tokens)
    for rule in rules:
        parser.add_rule(rule)

    return parser.rules, parser.ignore


def main(args):
    rules, ignore = parse_rules(args.rule, args.long_tokens)
    logger.debug(f"Parsed rules: {rules}")
    grammar = LSystemGrammar(rules, ignore, args.seed)

    n = args.iterations or 4
    if args.long_tokens:
        axiom = args.axiom.replace(",", " ")
        axiom = axiom.split()
        axiom = [Token(t) for t in axiom]
    else:
        axiom = args.axiom.replace(",", " ")
        axiom = "".join(axiom.split())
        axiom = [Token(c) for c in axiom]

    result = grammar.loop(axiom, n)

    for token in result:
        # TODO: Build an internal buffer and write the output in chunks
        args.output.write(token.name + " " * args.long_tokens)
    args.output.write("\n")


if __name__ == "__main__":
    args = parse_args()
    logging.basicConfig(
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
        level=LOG_LEVELS.get(args.log_level),
        stream=sys.stderr,
    )
    logger = logging.getLogger(name="parser.py")

    if args.config is not None:
        config = commentjson.load(args.config)

        # The rules are implicitly ordered in the grammar parser.
        rules = config.get("rules", [])
        if args.rule:
            rules += args.rule
        args.rule = rules

        # Commandline arguments take priority over the config file.
        if args.axiom is None:
            args.axiom = config["axiom"]
        if args.seed is None:
            args.seed = config.get("seed", None)
        if args.iterations is None:
            args.iterations = config.get("iterations", None)

        # If long tokens are specified (not the default) anywhere, either by the config file or
        # by commandline arguments, use long tokens.
        if config.get("long_tokens", None) is not None:
            args.long_tokens = args.long_tokens or config["long_tokens"]
        if config.get("long-tokens", None) is not None:
            args.long_tokens = args.long_tokens or config["long-tokens"]

    if args.axiom is None:
        args.axiom = ""

    main(args)
