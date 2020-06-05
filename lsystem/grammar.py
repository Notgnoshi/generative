import itertools
from dataclasses import dataclass
from typing import Callable, Dict, Generator, Iterable, Set, Tuple, Union

from multidict import MultiDict

Number = Union[int, float]


@dataclass
class Token:
    name: str
    parameters: Dict[str, Number]


@dataclass
class RuleMapping:
    production: Tuple[Token]
    probability: Union[None, Number]
    condition: Callable[[Dict[str, Number], Tuple, Tuple, Tuple], bool]
    left_context: Token
    right_context: Token


Rule = Tuple[Token, RuleMapping]


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
        rules: Dict[Token, RuleMapping],
        constants: Dict[str, Number] = None,
        ignore: Set[Token] = None,
    ):
        """Initialize a Lindenmayer-System grammar parser with the given rules.

        :param rules: A set of production rules. A mapping of token -> replacements, along with
        conditions on the replacements.
        """
        self.tokens: Set[Token] = set(rules.keys())
        self.constants: Dict[str, Number] = constants if constants is not None else dict()
        self.ignore: Set[Token] = ignore if ignore is not None else set()

    def rewrite(self, tokens: Iterable[Token]) -> Iterable[Token]:
        """Apply the production rules to the given string to rewrite it."""
        # Need to keep track of the cursor location so that we can look up context on either side.
        # Either that, or we iterate over the tokens three at a time, with an edge case for the first

    def loop(self, axiom: Iterable[Token]) -> Generator[Iterable[Token], None, None]:
        """Infinitely apply the production rules to the given starting axiom."""
        while True:
            # Depending on the implementation of _apply_once, it may compute the entire iteration
            # and cache the results, or compute them on the fly.
            #
            # We must also avoid yielding an exhausted generator. The itertools.tee docs say that
            # if one iterable uses most or all the data before another iterator starts (precisely
            # this case) it is faster to use a list.
            axiom = list(self.rewrite(axiom))
            yield axiom

    def loopn(self, axiom: Iterable[Token], n: int = 1) -> Iterable[Token]:
        return next(itertools.islice(self.loop(axiom), n - 1, None))
