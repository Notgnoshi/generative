from typing import Set, Tuple

from multidict import MultiDict
from pyparsing import (
    Literal,
    OneOrMore,
    Optional,
    ParserElement,
    White,
    Word,
    alphanums,
    delimitedList,
    pyparsing_common,
)

from .grammar import RuleMapping, Token, TokenName


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
    """

    def __init__(self, long_tokens=False):
        """Create a rule parser.

        Parsing rule after rule will construct the RuleParser.rule and RuleParser.ignore members.

        :param long_tokens: Whether to support multiple character tokens. Requires the tokens be
            comma or whitespace (or both) separated. Defaults to False.
        """
        self.rules: MultiDict[TokenName, RuleMapping] = MultiDict()
        self.ignore: Set[TokenName] = set()
        self.long_tokens = long_tokens

    @staticmethod
    def __get_grammars(long_tokens: bool):
        """Get the grammars for parsing rules and ignore lists.

        :param long_tokens: Whether the tokens must be delimited to support long tokens.
        """
        ParserElement.enablePackrat()

        COLON = Literal(":")
        LESS_THAN = Literal("<")
        GREATER_THAN = Literal(">")
        ARROW = Literal("->")

        PITCH_UP = Literal("^")
        PITCH_DOWN = Literal("v")
        ROLL_CCW = Literal("<")
        ROLL_CW = Literal(">")
        YAW_LEFT = Literal("-")
        YAW_RIGHT = Literal("+")
        PUSH_STACK = Literal("[")
        POP_STACK = Literal("]")
        FLIP_DIRECTION = Literal("|")

        token = (
            (Word(alphanums) if long_tokens else Word(alphanums, min=1, exact=1))
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

        probability = pyparsing_common.real
        rule_lhs = (
            Optional(token.setResultsName("left_context") + LESS_THAN)
            + token.setResultsName("lhs")
            + Optional(GREATER_THAN + token.setResultsName("right_context"))
        )
        rule_rhs = delimitedList(token, delim=White()) if long_tokens else OneOrMore(token)
        rule = (
            rule_lhs
            + Optional(COLON + probability.setResultsName("probability"))
            + ARROW
            + rule_rhs.setResultsName("rhs")
        )

        tokens = delimitedList(token, delim=White()) if long_tokens else OneOrMore(token)
        ignore = Literal("#ignore") + Optional(COLON) + tokens.setResultsName("ignore")

        return rule, ignore

    def _parse(self, rule: str):
        """Parse the given rule into textual tokens."""
        rule_grammar, ignore_grammar = self.__get_grammars(self.long_tokens)
        rule = rule.replace(",", " ")
        rule = rule.strip()
        if rule.startswith("#"):
            return ignore_grammar.parseString(rule)
        # NOTE: Expanding this to parametric grammars is nontrivial.
        return rule_grammar.parseString(rule)

    def parse(self, rule: str) -> Tuple[Token, RuleMapping]:
        """Parse the given rule into rhs -> production mappings.

        As a bit of a terrible design, ignore token lists will be parsed and added to
        RuleParser.ignore. But the rhs -> production mappings will still be created and returned.
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
