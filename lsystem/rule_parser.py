from typing import Set, Tuple

from multidict import MultiDict
from pyparsing import (
    Literal,
    OneOrMore,
    Optional,
    ParserElement,
    Word,
    delimitedList,
    oneOf,
    pyparsing_common,
)

from .grammar import RuleMapping, Token, TokenName

ParserElement.enablePackrat()

COLON = Literal(":")
LESS_THAN = Literal("<")
GREATER_THAN = Literal(">")
ARROW = Literal("->")
LEFT_PAREN = Literal("(")
RIGHT_PAREN = Literal(")")

PITCH_UP = Literal("^")
PITCH_DOWN = Literal("v")
# TODO: Does the dual uses of '<' and '>' pose any parsing problems?
ROLL_CCW = Literal("<")
ROLL_CW = Literal(">")
YAW_LEFT = Literal("-")
YAW_RIGHT = Literal("+")
PUSH_STACK = Literal("[")
POP_STACK = Literal("]")
FLIP_DIRECTION = Literal("|")

_l_token = (
    pyparsing_common.identifier
    | PITCH_UP
    | PITCH_DOWN
    | ROLL_CCW
    | ROLL_CW
    | YAW_LEFT
    | YAW_RIGHT
    | PUSH_STACK
    | POP_STACK
    | FLIP_DIRECTION
)

_l_probability = pyparsing_common.real
_l_rule_lhs = (
    Optional(_l_token.setResultsName("left_context") + LESS_THAN)
    + _l_token.setResultsName("lhs")
    + Optional(GREATER_THAN + _l_token.setResultsName("right_context"))
)
_l_rule_rhs = delimitedList(_l_token)
_l_rule = (
    _l_rule_lhs
    + Optional(COLON + _l_probability.setResultsName("probability"))
    + ARROW
    + _l_rule_rhs.setResultsName("rhs")
)

_l_tokens = delimitedList(_l_token)
_l_ignore = Literal("#ignore") + Optional(COLON) + _l_tokens.setResultsName("ignore")


class RuleParser:
    """Parse plaintext rules from e.g. unit tests, commandline args, JSON, to a Rule object.

    Rule Format
        "Rules" have one of two formats.

        1. A list of tokens to ignore when considering context.
        2. A production rule.

        The ignore tokens have the format

            #ignore: tok1,tok2,tok3

        The production rules have the format

            [left_context <] lhs [> right_context] [: probability] -> rhs[,rhs[...]]

    TODO: Provide an alternate parser that works on single-character tokens to avoid the infernal
    death by comma.
    """

    def __init__(self):
        """Create a rule parser.

        Parsing rule after rule will construct the RuleParser.rule and RuleParser.ignore members.
        """
        self.rules: MultiDict[TokenName, RuleMapping] = MultiDict()
        self.ignore: Set[TokenName] = set()

    def _parse(self, rule: str):
        """Parse the given rule into textual tokens."""
        rule = rule.strip()
        if rule.startswith("#"):
            return _l_ignore.parseString(rule)
        # NOTE: Expanding this to parametric grammars is nontrivial.
        return _l_rule.parseString(rule)

    def parse(self, rule: str) -> Tuple[Token, RuleMapping]:
        """Parse the given rule into rhs -> production mappings.

        As a bit of a terrible design, ignore token lists will be parsed and added to
        RuleParser.ignore. But the rhs -> production mappings will be created and returned.

        TODO: Find a better design.
        """
        results = self._parse(rule)

        if "ignore" in results:
            for tok in results["ignore"]:
                self.ignore.add(tok)
            return None

        if "lhs" not in results or "rhs" not in results:
            raise ValueError("Something went horribly wrong")

        probability = results.get("probability", None)
        left_context = results.get("left_context", None)
        right_context = results.get("right_context", None)

        if left_context is not None:
            left_context = Token(left_context)

        if right_context is not None:
            right_context = Token(right_context)

        lhs = Token(results["lhs"])
        rhs = tuple(Token(r) for r in results["rhs"])

        return lhs, RuleMapping(rhs, probability, left_context, right_context)

    def add_rule(self, rule: str):
        """Add the given rule to the parser.

        This is the intended public interface to this object.
        """
        results = self.parse(rule)
        if results is not None:
            lhs, mapping = results
            self.rules.add(lhs.name, mapping)
