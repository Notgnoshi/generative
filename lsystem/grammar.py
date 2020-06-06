import itertools
import logging
import random
from dataclasses import dataclass
from typing import Callable, Dict, Generator, Iterable, List, NewType, Set, Tuple, Union

import numpy as np
from more_itertools import peekable
from multidict import MultiDict

logger = logging.getLogger(__name__)

Number = Union[int, float]
TokenName = NewType("TokenName", str)
VariableName = NewType("VariableName", str)
Constants = Dict[VariableName, Number]


# Python 3.7+
@dataclass
class Token:
    """A token in the language defined by the L-System grammar."""

    name: str
    parameters: Union[None, Tuple[Number]] = None


@dataclass
class RuleMapping:
    """Each rule is the mapping of a token to a tuple of values; this is the tuple of values.

    Care should be made in creating the production function, and the condition.
    """

    # Call this function to get the replacement tokens.
    # Constants, token being rewritten, left context, right context
    production: Callable[[Constants, Token, Union[None, Token], Union[None, Token]], Tuple[Token]]
    # If not None, and between 0 and 1, apply this rule with the given probability.
    probability: Union[None, Number] = None
    # Call this function to determine if the condition is met. Same arguments as the production.
    condition: Union[None, Callable[[Constants, Token, Token, Token], bool]] = None
    # Note that the tokens include their parameter values, but for the purpose of context, the
    # parameters are ignored.
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
    """A context-sensitive, stochastic, and parametric Lindenmayer System grammar parser.

    Parses strings of tokens using a set of production rules.
    Unlike traditional parsing, the rules are applied on the _entire_ string of tokens from left to
    right _before_ returning to the first symbol in the string.

    Example
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

    The rules may be parametric, and applied only if a certain condition is met, and the
    replacements may modify per-token parameters for consideration the next iteration.
    There may also be global constants (not parameters) that may be used in the condition evaluation
    or the replacement.

    The context matching only matches the token, and not the parameters.

    Thus, production rules are the following mapping:
        token -> tuple(left_context, right_context, probability, condition, production)

    The left and right contexts are single tokens, possibly parametric.
    The probability may be None, or a float in (0, 1].
    The condition is a boolean expression using Python syntax. Available variables are:
        * Any global constants
        * Any context parameters
        * Any parameters for the token being replaced
    The production is a series of tokens, possible parametric with mathematical operations performed
    using Python syntax on the parameters. The productions have the same access as the conditions.

    There may be multiple rules for the same token.
    If, after considering context and the condition expression, there are multiple matching rules,
    one will be picked randomly with the probability distribution specified in the rule definitions.
    """

    def __init__(
        self,
        rules: MultiDict[TokenName, RuleMapping],
        constants: Constants = None,
        ignore: Set[TokenName] = None,
        seed: int = None,
    ):
        """Initialize a Lindenmayer-System grammar parser with the given rules.

        NOTE: We assume all parameters given are valid. This includes things like the callables
        in the RuleMappings, expressions referring to things that exist, there being a rule for
        every token, etc.

        TODO: I do not expect direct user interaction with this class, but the details of wrapping
        it still need to be figured out.

        :param rules: A set of production rules. A mapping of token -> replacements, along with
        conditions on the replacements.
        """
        self.tokens: Set[TokenName] = set(rules.keys())
        self.constants: Constants = constants if constants is not None else dict()
        self.ignore: Set[TokenName] = ignore if ignore is not None else set()
        self.rules: MultiDict[TokenName, RuleMapping] = rules

        self.seed = seed if seed is not None else random.randint(0, 2 ** 32 - 1)
        np.random.seed(self.seed)
        logger.info(f"Using random seed: {self.seed}")

    def pick_rule(self, rules: List[RuleMapping]) -> RuleMapping:
        """If there are multiple matching rules for a given token, pick one randomly.

        Only pick randomly if all rules have a probability value assigned that sum to 1.
        Otherwise just pick the first.
        """
        if len(rules) == 1:
            return rules[0]

        for rule in rules:
            if rule.probability is None:
                return rule
        p = [r.probability for r in rules]
        if sum(p) > 1.0:
            raise ValueError("Probabilities cannot sum over 1.0")

        choice = np.random.choice(rules, p=p)
        return choice[0]

    def apply_rules(
        self, token: Token, left_ctx: Token = None, right_ctx: Token = None
    ) -> Iterable[Token]:
        """Apply the production rules to the given token with the specified context.

        Note that the left and right context are optional to facilitate the edge cases for the
        first and last tokens in the string.
        """
        # Get all rules that match the given token
        rules = self.rules.getall(token.name)

        # Filter rules by context. Either there's no context in the rule, or the context matches
        rules = [r for r in rules if r.left_context is None or r.left_context == left_ctx]
        rules = [r for r in rules if r.right_context is None or r.right_context == right_ctx]

        # Of the remaining rules, pick one randomly.
        rule = self.pick_rule(rules)

        replacement = rule.production(self.constants, token, left_ctx, right_ctx)
        logger.debug(f"Rewriting {token} -> {replacement}")
        return replacement

    def rewrite(self, tokens: Iterable[Token]) -> Iterable[Token]:
        """Apply the production rules to the given string to rewrite it."""
        # more-itertools is _awesome_.
        tokens = peekable(tokens)
        first: Token = next(tokens)
        second: Token = next(tokens, None)

        # Handle the edge case at the beginning (the first token doesn't have left context).
        for token in self.apply_rules(first, left_ctx=None, right_ctx=second):
            yield token

        # Add the first two tokens back
        if second is not None:
            tokens.prepend(second)
        tokens.prepend(first)
        # Handle the rest like a sane person.
        for left_ctx, token, right_ctx in triplewise(tokens):
            for token in self.apply_rules(token, left_ctx, right_ctx):
                yield token

        # I don't know how else to handle the additional edge case of the tokens iterable
        # only containing two tokens.
        try:
            left_ctx = token
            token = right_ctx
        except UnboundLocalError:  # The loop never executed, so the loop vars don't exist.
            left_ctx = first
            token = second

        if token is not None:
            # Handle the edge case at the end (the last token doesn't have right context).
            for token in self.apply_rules(token, left_ctx, right_ctx=None):
                yield token

    def loop(self, axiom: Iterable[Token]) -> Generator[Iterable[Token], None, None]:
        """Infinitely apply the production rules to the given starting axiom."""
        i = 0
        logger.debug(f"Iteration 0: {axiom}")
        while True:
            i += 1
            # Depending on the implementation of _apply_once, it may compute the entire iteration
            # and cache the results, or compute them on the fly.
            #
            # We must also avoid yielding an exhausted generator. The itertools.tee docs say that
            # if one iterable uses most or all the data before another iterator starts (precisely
            # this case) it is faster to use a list.
            axiom = list(self.rewrite(axiom))
            logger.debug(f"Iteration {i}: {axiom}")
            yield axiom

    def loopn(self, axiom: Iterable[Token], n: int = 1) -> Iterable[Token]:
        """Apply the productions rules n times to the given axiom, and return the result."""
        return next(itertools.islice(self.loop(axiom), n - 1, None))
