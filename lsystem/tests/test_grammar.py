import itertools
import unittest

from multidict import MultiDict

from lsystem.grammar import LSystemGrammar, RuleMapping, Token, triplewise


def tokenize(s: str):
    return [Token(c) for c in s]


class ContextFreeParsing(unittest.TestCase):
    """Test LSystemGrammar with context free parsing."""

    def setUp(self):
        self.rules = MultiDict(
            {
                "a": RuleMapping(lambda c, t, l, r: (Token("a"), Token("b"))),
                "b": RuleMapping(lambda c, t, l, r: (Token("a"),)),
            }
        )
        self.system = LSystemGrammar(self.rules)

    def test_apply_rules_a(self):
        i = self.system.apply_rules(Token("a"))
        self.assertSequenceEqual(i, tokenize("ab"))

    def test_apply_rules_b(self):
        i = self.system.apply_rules(Token("b"))
        self.assertSequenceEqual(i, tokenize("a"))

    def test_rewrite_a(self):
        axiom = tokenize("a")
        rewrite = list(self.system.rewrite(axiom))
        self.assertSequenceEqual(rewrite, tokenize("ab"))

    def test_rewrite_b(self):
        axiom = tokenize("b")
        rewrite = list(self.system.rewrite(axiom))
        self.assertSequenceEqual(rewrite, tokenize("a"))

    def test_rewrite_abaa(self):
        axiom = tokenize("abaa")
        rewrite = list(self.system.rewrite(axiom))
        self.assertSequenceEqual(rewrite, tokenize("abaabab"))

    def test_rewrite_ab(self):
        # Two-token strings are an edge case in the context sensitive parsing
        axiom = tokenize("ab")
        rewrite = list(self.system.rewrite(axiom))
        self.assertSequenceEqual(rewrite, tokenize("aba"))

    def test_loop(self):
        axiom = tokenize("a")
        g = self.system.loop(axiom)
        expected = ["ab", "aba", "abaab", "abaababa"]
        expected = [tokenize(a) for a in expected]
        for actual, expectation in zip(g, expected):
            self.assertSequenceEqual(actual, expectation)

    def test_loopn(self):
        axiom = tokenize("a")
        expected = tokenize("abaababa")
        actual = self.system.loopn(axiom, 4)
        self.assertSequenceEqual(actual, expected)


class StochasticParsing(unittest.TestCase):
    """Test LSystemGrammar with stochastic parsing."""

    def setUp(self):
        self.rules = MultiDict(
            {
                "a": RuleMapping(lambda c, t, l, r: tokenize("a"), probability=0.33),
                "a": RuleMapping(lambda c, t, l, r: tokenize("aa"), probability=0.33),
                "a": RuleMapping(lambda c, y, l, r: tokenize("aaa"), probability=0.34),
            }
        )

    # TODO: There's a bug here somewhere (I think, hard to tell with randomness)
    # def test_pick_rule_a(self):
    #     system = LSystemGrammar(self.rules, seed=None)
    #     rule = system.pick_rule(self.rules.getall("a"))

    # def test_pick_rule_b(self):
    #     system = LSystemGrammar(self.rules, seed=0x626C61)
    #     rule = system.pick_rule(self.rules.getall("a"))

    # def test_pick_rule_c(self):
    #     system = LSystemGrammar(self.rules, seed=0xA656974)
    #     rule = system.pick_rule(self.rules.getall("a"))


class ContextSensitiveParsing(unittest.TestCase):
    """Test LSystemGrammar with context sensitive parsing."""


class ParametricParsing(unittest.TestCase):
    """Test LSystemGrammar with parametric parsing."""


class CombinedParsing(unittest.TestCase):
    """Test LSystemGrammar with combined context sensitive, stochastic, and parametric parsing."""
