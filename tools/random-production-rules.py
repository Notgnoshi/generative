#!/usr/bin/env python
"""Generate random L-System production rules."""
import argparse
import itertools
import json
import logging
import random
import sys

import numpy as np
from multidict import MultiDict

LOG_LEVELS = {
    "CRITICAL": logging.CRITICAL,
    "ERROR": logging.ERROR,
    "WARNING": logging.WARNING,
    "INFO": logging.INFO,
    "DEBUG": logging.DEBUG,
}
DEFAULT_LEVEL = "INFO"


def generate_random_seed():
    return random.randint(0, 2 ** 32 - 1)


def parse_args():
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )

    parser.add_argument(
        "--output",
        "-o",
        type=argparse.FileType("w"),
        default=sys.stdout,
        help="A file to output the expanded axiom to. Defaults to stdout.",
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
        "--seed",
        "-s",
        type=int,
        default=generate_random_seed(),
        help="The value to seed the random generator with",
    )
    parser.add_argument(
        "--placeholders",
        "-p",
        action="store_true",
        default=False,
        help="Whether to allow tokens other than f,g,F,G,v,d,D",
    )
    parser.add_argument(
        "--stochastic",
        action="store_true",
        default=False,
        help="Whether to allow stochastic production rules",
    )
    parser.add_argument(
        "--context-sensitive",
        "-c",
        action="store_true",
        default=False,
        help="Whether to allow context-sensitive production rules",
    )
    parser.add_argument(
        "--drawing-bias",
        type=float,
        default=1,
        help="The bias to apply for generating 'f' and 'g' tokens in the RHS.",
    )
    parser.add_argument(
        "--skipping-bias",
        type=float,
        default=1,
        help="The bias to apply for generating 'F' and 'G' tokens in the RHS.",
    )
    parser.add_argument(
        "--placeholder-bias",
        type=float,
        default=1,
        help="The bias to apply for generating placeholder tokens in the RHS.",
    )
    parser.add_argument(
        "--orientation-bias",
        type=float,
        default=1,
        help="The bias to apply for generating orientation tokens in the RHS.",
    )
    parser.add_argument(
        "--branching-bias",
        type=float,
        default=1,
        help="The bias to apply for generating '[' and ']' tokens in the RHS.",
    )
    parser.add_argument(
        "--temperature",
        "-t",
        type=float,
        default=1,
        help="The temperature to use in generating the random rules.",
    )
    parser.add_argument(
        "--dynamic",
        "-d",
        action="store_true",
        default=False,
        help="Whether to allow the probability distribution for tokens to change over time",
    )

    return parser.parse_args()


def generate_lhs_distribution(allow_placeholders=False):
    """Generate the probability distribution for the LHS tokens."""
    distribution = {
        "F": 0.2,
        "G": 0.1,
        "f": 0.05,
        "g": 0.05,
    }
    remaining_pm = 1.0 - sum(distribution.values())

    # Split the remaining probability mass amongst the placeholder tokens.
    if allow_placeholders:
        tokens = "abcehijklmnopqrstuwxyz"
        distribution.update({t: remaining_pm / len(tokens) for t in tokens})
    # Split the remaining probability mass between the LHS tokens.
    else:
        for token in distribution:
            distribution[token] += remaining_pm / len(distribution)
    return distribution


def generate_lhs_tokens(distribution, rng):
    """Generate a set of tokens to build production rules for."""
    # Iteration order over a set is apparently non-deterministic, so use a list instead...
    tokens = []
    token_pool = list(distribution.keys())
    token_probabilities = list(distribution.values())
    while "f" not in tokens:
        tokens.append(rng.choice(token_pool, p=token_probabilities))

    return tokens


def generate_rhs_distribution(available_tokens: list, biases: dict, temperature):
    """Generate the probability distribution for the RHS tokens.

    :param available_tokens: The tokens appearing on the LHS
    :param biases: The set of biases used to tune the distribution.

    Each tunable parameter can go from 0 to infinity. Before biasing, every token, with the
    exception of the drawing toggling tokens, has a uniform probability. Increasing a certain bias,
    makes that token more likely than the others.

    The biases are multiplicative before normalization.

    :param temperature: The softmax temperature for normalizing the probability distribution after
    applying the biases.
    """
    other_tokens = list("-+v^<>|[]")

    tokens = available_tokens + other_tokens
    # toggle_probability_mass = 10 ** -max(3, 2 * len(available_tokens))
    toggle_probability_mass = 1e-7
    pm = 1.0 - toggle_probability_mass

    distribution = {t: pm / len(tokens) for t in tokens}
    distribution["d"] = toggle_probability_mass / 2
    distribution["D"] = toggle_probability_mass / 2

    return bias(distribution, biases, temperature)


def bias(distribution, biases, temperature):
    for token in distribution:
        distribution[token] *= biases[token] * temperature
    # distribution = softmax(distribution, temperature)
    distribution = normalize(distribution)
    return distribution


def softmax(distribution, temperature):
    denom = sum(np.exp(np.array(list(distribution.values())) / temperature))
    for token in distribution:
        distribution[token] = np.exp(distribution[token] / temperature) / denom
    return distribution


def normalize(distribution):
    denom = sum(distribution.values())
    for token in distribution:
        distribution[token] = distribution[token] / denom
    return distribution


def generate_rhs_token(
    distribution: dict, biases: dict, rng, temperature: float, dynamic: bool, alpha: float
):
    """Add a new token to the RHS of a certain rule."""
    token_pool = list(distribution.keys())
    token_probabilities = list(distribution.values())
    token = rng.choice(token_pool, p=token_probabilities)

    # Allow the generation of certain tokens to bias the generation of future ones.
    if dynamic:
        if token == "[":
            biases["["] /= alpha
            biases["]"] *= alpha
        elif token == "]":
            biases["]"] /= alpha
            biases["["] *= alpha
        elif token == "d":
            biases["d"] /= alpha
            biases["D"] *= alpha
        elif token == "D":
            # Make it even less likely to turn drawing off after we turned it back on
            biases["d"] /= alpha
            biases["D"] /= alpha
        elif token == "-":
            biases["-"] *= alpha
            biases["+"] *= alpha
        elif token == "+":
            biases["+"] *= alpha
            biases["-"] *= alpha
        elif token == "v":
            biases["v"] *= alpha
            biases["^"] *= alpha
        elif token == "^":
            biases["^"] *= alpha
            biases["v"] *= alpha
        elif token == "<":
            biases["<"] *= alpha
            biases[">"] *= alpha
        elif token == ">":
            biases[">"] *= alpha
            biases["<"] *= alpha
        else:
            biases[token] /= alpha

        distribution = bias(distribution, biases, temperature)

    return token


def generate_rule(token, distribution, base_biases, temperature, rng, dynamic):
    """Generate a rule for the given token."""
    biases = base_biases.copy()
    length = rng.integers(3, 20)
    production = ""
    for _ in range(length):
        alpha = 1.5
        biases[token] *= alpha
        new_token = generate_rhs_token(distribution, biases, rng, temperature, dynamic, alpha)
        if new_token == token:
            biases[token] = 1
        production += new_token
    # TODO: Close any unmatched pops, pushes or drawing toggles.
    # Prepend ['s and d's. Append ]'s and D's. Only add up to one 'd' or 'D'.
    return production


def pairwise(iterable):
    """Pairwise: s -> (s0,s1), (s1,s2), (s2, s3), ..."""
    a, b = itertools.tee(iterable)
    next(b, None)
    return zip(a, b)


def main(args):
    logger.info("Using random seed %d", args.seed)
    rng = np.random.default_rng(args.seed)

    # F,G - Step forward while drawing
    # f,g - Step forward without drawing
    # -,+ - Yaw around the normal axis
    # v,^ - Pitch around the transverse axis
    # <,> - Roll around the longitudinal axis
    # |   - Flip orientation 180 degrees
    # d,D - Turn drawing off, on
    # [,] - Push, pop position and orientation onto a stack

    # Pick the random distribution used for picking the LHS tokens.
    lhs_distribution = generate_lhs_distribution(args.placeholders)
    logger.debug("LHS Distribution: %s", lhs_distribution)

    # Generate the LHS tokens
    lhs_tokens = generate_lhs_tokens(lhs_distribution, rng)
    logger.debug("LHS Tokens: %s", lhs_tokens)

    # Pick a base random distribution used for picking the RHS tokens.
    biases = {
        "-": args.orientation_bias,
        "+": args.orientation_bias,
        "v": args.orientation_bias,
        "^": args.orientation_bias,
        "<": args.orientation_bias,
        ">": args.orientation_bias,
        "|": args.orientation_bias,
        "[": args.branching_bias * 2,
        "]": args.branching_bias,
        "d": 1e-5,
        "D": 1e-5,
        "F": args.drawing_bias,
        "G": args.drawing_bias,
        "f": args.skipping_bias,
        "g": args.skipping_bias,
    }
    biases.update({t: args.placeholder_bias for t in "abcehijklmnopqrstuwxyz"})
    logger.debug("RHS biases: %s", biases)
    rhs_distribution = generate_rhs_distribution(lhs_tokens, biases, args.temperature)
    logger.debug("RHS Distribution: %s sum(%f)", rhs_distribution, sum(rhs_distribution.values()))

    # Consider the two rules
    #   a -> aa
    #   b -> ab
    # These rules guarantee that the resulting string expands exponentially when the rules are
    # iteratively applied. But then consider the rules
    #   a -> a
    #   b -> a
    # which could very conceivably be randomly generated. These particular rules will not result
    # in a string expansion (which ultimately results in an expanding image when the rules are
    # applied).
    #
    # I _think_ that as long as we guarantee each production rule contains at least two lhs_tokens
    # in the production, we can guarantee that the resulting rule will not result in a steady state.
    #
    # There are probably better ways that require evaluating the graph formed by following the
    # rules, but that doesn't sound particularly interesting to me. So simple is what you get.

    # TODO: If stochastic, generate a distribution for each token
    # Generates rules of the form 'lhs : probability -> rhs'
    rules = MultiDict()
    for token in lhs_tokens:
        num_competing_rules = rng.integers(1, 5) if args.stochastic else 1
        subdivisions = []
        if num_competing_rules > 1:
            # Subdivide the interval [0, 1] into N pieces, and use the length of each as the probability
            subdivisions = rng.choice(list(range(0, 100)), size=num_competing_rules - 1)
            subdivisions.sort()

        subdivisions = [0] + list(subdivisions) + [100]
        probabilities = []
        for begin, end in pairwise(subdivisions):
            probabilities.append((end - begin) / 100)

        for p in probabilities:
            rules.add(
                token,
                {
                    "lhs": token,
                    "probability": p if args.stochastic else None,
                    "left_context": None,
                    "right_context": None,
                    "rhs": None,
                },
            )

    # Generate the RHS tokens
    for token, rule in rules.items():
        rule["rhs"] = generate_rule(
            token, rhs_distribution, biases, args.temperature, rng, args.dynamic
        )

    # If context-sensitive, generate left and/or right contexts from the RHS
    # Generates rules of the form 'lhs ctx < lhs > rhs ctx -> rhs'
    # TODO: Since context is a filter, maybe it makes sense to add multiple rules with the same lhs?
    if args.context_sensitive:
        for token, rule in rules.items():
            rhs = rule["rhs"]
            # TODO: Maybe don't always pick the first matching token if there are multiple?
            t_idx = rhs.find(token)
            if t_idx != -1:
                choice = rng.integers(0, 3, endpoint=True)
                # Don't add context
                if choice == 0:
                    continue
                # Add left context
                if t_idx != 0 and choice in (1, 3):
                    rule["left_context"] = rhs[t_idx - 1]
                # Add right context
                if t_idx != len(rhs) - 1 and choice in (2, 3):
                    rule["right_context"] = rhs[t_idx + 1]

    # Format the rules according to the syntax described by the tools/parse.py script.
    formatted_rules = []
    for token, production in rules.items():
        rule = f"{token}"
        if production["left_context"] is not None:
            rule = f"{production['left_context']} < " + rule
        if production["right_context"] is not None:
            rule = rule + f"{production['right_context']} > " + rule
        if production["probability"] is not None:
            rule = rule + f": {production['probability']}"
        rule += f" -> {production['rhs']}"
        formatted_rules.append(rule)

    axiom = rng.choice(lhs_tokens)
    json.dump({"seed": args.seed, "rules": formatted_rules, "axiom": axiom}, args.output)
    args.output.write("\n")


if __name__ == "__main__":
    args = parse_args()

    logging.basicConfig(
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
        level=LOG_LEVELS.get(args.log_level),
        stream=sys.stderr,
    )
    logger = logging.getLogger(name=__file__)

    main(args)
