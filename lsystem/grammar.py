import abc
import itertools
from typing import Dict, Generator, Iterable, NewType

Token = NewType("Token", str)
Tokens = Iterable[Token]


class LSystemGrammar(abc.ABC):
    """A representation of a Lindenmayer System grammar.

    An L-System grammar is composed of a set of rules of the form:

        x -> y

    These rules mean: replace occurences of the string 'x' with the string 'y'. Such strings may
    be composed of multiple tokens. The production rules are then iteratively applied to a starting
    axiom, comprised of at least one token.

    Example:
        a -> ab
        b -> a

        0th iteration:  a     (axiom)
        1st iteration:  ab    (apply a -> ab)
        2nd iteration:  aba   (apply a -> ab, then b -> a, left-to-right)
        3rd iteration:  abaab (apply rules left-to-right)
    """

    def __init__(self, rules: Dict[Tokens, Tokens]):
        """Initialize the grammar with a set of production rules."""
        self.rules = rules

    @abc.abstractmethod
    def _apply_once(self, text: Tokens) -> Tokens:
        """Apply one iteration of the production rules to the given text left-to-right."""

    def iapply(self, axiom: Tokens) -> Generator[Tokens, None, None]:
        """Infinitely apply the production rules to the given axiom.

        :param axiom: The axiom to apply the production rule to.
        :returns: A generator of token iterables.
        """
        while True:
            # Depending on the implementation of _apply_once, it may compute the entire iteration
            # and cache the results, or compute them on the fly.
            #
            # We must also avoid yielding an exhausted generator. The itertools.tee docs say that
            # if one iterable uses most or all the data before another iterator starts (precisely
            # this case) it is faster to use a list.
            axiom = list(self._apply_once(axiom))
            yield axiom

    def apply(self, axiom: Tokens, times: int = 1) -> Iterable[Token]:
        """Apply the production rules to the given axiom a fixed number of times.

        The production rules are applied left-to-right for every token once per iteration.

        :param axiom: The axiom to apply the production rules to.
        :param times: The number of times to sequentially apply the rules.
        :returns: An iterable of tokens.
        """
        return next(itertools.islice(self.iapply(axiom), times - 1, None))


class ContextFreeGrammar(LSystemGrammar):
    """A context free L-System grammar.

    Each production rule in a context-free grammar has a single nonterminal token on the LHS of
    the rule. That is, it is not necessary to consider the surrounding context when applying the
    production rules.
    """

    def _apply_once(self, text: Tokens) -> Tokens:
        # Each token (hopefully) results in more than one result token.
        # So chain the rewrites together so that the result in a single iterator of tokens.
        rewrites = (self.rules.get(token, token) for token in text)
        return itertools.chain.from_iterable(rewrites)


class ContextSensitiveGrammar(LSystemGrammar):
    """A context sensitive L-System grammar.

    In context sensitive grammars, each production rule might require context matching on either
    side of the LHS nonterminal symbol. This is annotated by

        a < x > b -> y

    where by, in order to apply the rule 'x -> y', 'x' must be surrounded by the left context 'a',
    and the right context 'b'. Both left, and right contexts are optional.

    This can be rewritten into the rule

        axb -> ayb

    TODO: I don't know how to write a context-sensitive grammar parser.
    """


class StochasticGrammar(LSystemGrammar):
    """A stochastic L-System grammar.

    A stochastic grammar is one where there might be multiple RHS productions for a given LHS
    nonterminal symbol. When parsing/applying the grammar, the RHS productions are chosen with
    some probability specified in the rule.

    Example:
        In the following grammar, 'x' could be replaced by either 'y' or 'z' with equal probability.

        x: 0.5 -> y
        x: 0.5 -> z

        It's likely that this will require the representation

        x -> [(y, 0.5), (z, 0.5)]

    TODO: This will require changing how the rules are specified.
    """


class ParametricGrammar(LSystemGrammar):
    """A parametric L-System grammar.

    In a parametric grammar, nonterminal symbols may be followed by a condition and a parameter
    list. For example, you might have the rule

        a(x, y): x == 0 -> a(1, y + 1)b(x + 1, y)

    These parameters maybe considered by both the application of the production rules, and by the
    drawing system responsible for rendering a given L-System.

    TODO: This will require changing how the rules are specified.
    """
