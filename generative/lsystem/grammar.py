import itertools
import logging
import random
from dataclasses import dataclass
from typing import Iterable, List, NewType, Set, Tuple, Union

import numpy as np
from more_itertools import peekable
from multidict import MultiDict

logger = logging.getLogger(__name__)

Number = Union[int, float]
TokenName = NewType("TokenName", str)


@dataclass
class Token:
    """A token in the language defined by the L-System grammar."""

    name: str


@dataclass
class RuleMapping:
    """Each rule is the mapping of a token to a tuple of values; this is the tuple of values.

    Care should be made in creating the production function, and the condition.
    """

    production: Tuple[Token]
    # If not None, and between 0 and 1, apply this rule with the given probability.
    probability: Union[None, Number] = None
    left_context: Union[None, Token] = None
    right_context: Union[None, Token] = None


def triplewise(iterable):
    """Iterate over the given iterable in triples."""
    a, b, c = itertools.tee(iterable, 3)
    next(b, None)
    next(c, None)
    next(c, None)
    return zip(a, b, c)


class LSystemGrammar:
    """A context-sensitive and stochastic Lindenmayer System grammar parser.

    Parses strings of tokens using a set of production rules.
    Unlike traditional parsing, the rules are applied on the _entire_ string of tokens from left to
    right _before_ returning to the first symbol in the string.

    Example:
        Given the rules 'a -> ab', 'b -> a', and the starting axiom 'a',

        1st iteration: a -> ab
        2nd iteration: ab -> aba (apply 'a -> ab' on the first token, then 'b -> a' on the second).
        3rd iteration: aba -> abaab

    The rules may be context-sensitive, and may have at most one token of context to the left,
    right, or both directions.
    Some tokens may optionally be ignored when considering context.

    The rules may be stochastic, with more than one possible replacement for a given token.
    The probabilities for each token can be specified.
    If multiple replacements for the same token are parsed, but probabilities are not given, we
    assume uniform probability.

    The left and right contexts are single tokens.
    The probability may be None, or a float in (0, 1].

    There may be multiple rules for the same token.
    If, after considering context, there are multiple matching rules,
    one will be picked randomly with the probability distribution specified in the rule definitions.
    """

    def __init__(
        self,
        rules: MultiDict[TokenName, RuleMapping],
        ignore: Set[TokenName] = None,
        seed: int = None,
    ):
        """Initialize a Lindenmayer-System grammar parser with the given rules.

        :param rules: A set of production rules. A mapping of token -> replacements.
        """
        self.ignore: Set[TokenName] = ignore if ignore is not None else set()
        self.rules: MultiDict[TokenName, RuleMapping] = rules

        self.seed = seed if seed is not None else random.randint(0, 2**32 - 1)
        np.random.seed(self.seed)
        logger.info(f"Using random seed: {self.seed}")

    def pick_rule(self, rules: List[RuleMapping], token, left_ctx, right_ctx) -> RuleMapping:
        """Pick the right rule based off the probability values or the parametric condition."""
        # If there's not choice, no need to make it a random choice.
        if len(rules) == 1:
            return rules[0]

        for rule in rules:
            if rule.probability is None:
                return rule

        return np.random.choice(rules, p=[r.probability for r in rules])

    def apply_rules(
        self, token: Token, left_ctx: Token = None, right_ctx: Token = None
    ) -> Iterable[Token]:
        """Apply the production rules to the given token with the specified context.

        Note that the left and right context are optional to facilitate the edge cases for the
        first and last tokens in the string.
        """
        # Get all rules that match the given token
        if token.name in self.rules:
            rules = self.rules.getall(token.name)
        else:
            logger.debug(f"No rules found for {token}")
            return (token,)

        # Filter rules by context. Either there's no context in the rule, or the context matches
        # Note the edge cases at the ends of the string where there is only context to one side.

        # Filter by left context
        rules = [
            r
            for r in rules
            # Either there is no left context in the rule
            if r.left_context is None
            # Or there is, and the left context is available and matches.
            or (left_ctx is not None and r.left_context.name == left_ctx.name)
        ]
        # Filter by right context
        rules = [
            r
            for r in rules
            # Either there is no right context in the rule
            if r.right_context is None
            # Or there is, and the right context is available and matches.
            or (right_ctx is not None and r.right_context.name == right_ctx.name)
        ]

        # If we don't have a matching rule, just passthrough the token.
        if not rules:
            logger.debug(f"No rule matching context found for {token}. Passing through.")
            return (token,)

        # Of the remaining rules, pick one randomly.
        rule = self.pick_rule(rules, token, left_ctx, right_ctx)
        logger.debug(f"Applying rule {token} -> {rule.production}")
        return rule.production

    def rewrite(self, tokens: Iterable[Token]) -> Iterable[Token]:
        """Apply the production rules to the given string to rewrite it."""
        tokens = peekable(tokens)
        left = None
        right = None

        for token in tokens:
            # Peek until you find a right token that isn't ignored.
            # The peekable indices are centered on the current item.
            for i in itertools.count(0):
                try:
                    if tokens[i].name not in self.ignore:
                        right = tokens[i]
                        break
                except IndexError:
                    right = None
                    break

            for replacement in self.apply_rules(token, left_ctx=left, right_ctx=right):
                yield replacement

            # Update the left context for the next iteration.
            if token.name not in self.ignore:
                left = token

    def loop(self, axiom: Iterable[Token], n: int = 1) -> Iterable[Token]:
        """Apply the productions rules n times to the given axiom, and return the result."""
        for _ in range(n):
            axiom = self.rewrite(axiom)
        return axiom
