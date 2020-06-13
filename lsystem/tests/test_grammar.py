import itertools
import logging
import unittest

from multidict import MultiDict

from lsystem.grammar import LSystemGrammar, RuleMapping, Token, triplewise

logger = logging.getLogger(__name__)


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
            [
                ("a", RuleMapping(lambda c, t, l, r: tokenize("a"), probability=0.33)),
                ("a", RuleMapping(lambda c, t, l, r: tokenize("aa"), probability=0.33)),
                ("a", RuleMapping(lambda c, y, l, r: tokenize("aaa"), probability=0.34)),
            ]
        )

    def test_pick_rule_a(self):
        system = LSystemGrammar(self.rules, seed=0x420)
        rule = system.pick_rule(self.rules.getall("a"))
        rewrite = rule.production(None, None, None, None)
        self.assertSequenceEqual(rewrite, tokenize("aa"))

    def test_pick_rule_b(self):
        system = LSystemGrammar(self.rules, seed=0x626C61)
        rule = system.pick_rule(self.rules.getall("a"))
        rewrite = rule.production(None, None, None, None)
        self.assertSequenceEqual(rewrite, tokenize("aaa"))

    def test_pick_rule_c(self):
        system = LSystemGrammar(self.rules, seed=0xA656974)
        rule = system.pick_rule(self.rules.getall("a"))
        rewrite = rule.production(None, None, None, None)
        self.assertSequenceEqual(rewrite, tokenize("a"))


class ContextSensitiveParsing(unittest.TestCase):
    """Test LSystemGrammar with context sensitive parsing."""

    def test_left_context(self):
        rules = MultiDict(
            [
                ("a", RuleMapping(lambda c, t, l, r: tokenize("ab"))),
                ("b", RuleMapping(lambda c, t, l, r: tokenize("b"), left_context=Token("a"))),
                ("b", RuleMapping(lambda c, t, l, r: tokenize("a"), left_context=Token("b"))),
            ]
        )
        system = LSystemGrammar(rules)
        axiom = tokenize("a")
        rewrite1 = list(system.rewrite(axiom))
        self.assertSequenceEqual(rewrite1, tokenize("ab"))
        rewrite2 = list(system.rewrite(rewrite1))
        self.assertSequenceEqual(rewrite2, tokenize("abb"))
        rewrite3 = list(system.rewrite(rewrite2))
        self.assertSequenceEqual(rewrite3, tokenize("abba"))

    def test_left_context_edge(self):
        rules = MultiDict(
            [
                ("a", RuleMapping(lambda c, t, l, r: tokenize("ab"))),
                ("b", RuleMapping(lambda c, t, l, r: tokenize("b"), left_context=Token("a"))),
                ("b", RuleMapping(lambda c, t, l, r: tokenize("a"), left_context=Token("b"))),
            ]
        )
        system = LSystemGrammar(rules)
        axiom = tokenize("b")
        rewrite = list(system.rewrite(axiom))
        # If there's no context to match, pick the first rule listed.
        self.assertSequenceEqual(rewrite, tokenize("b"))

    def test_right_context(self):
        rules = MultiDict(
            [
                ("a", RuleMapping(lambda c, t, l, r: tokenize("ba"))),
                ("b", RuleMapping(lambda c, t, l, r: tokenize("b"), right_context=Token("a"))),
                ("b", RuleMapping(lambda c, t, l, r: tokenize("a"), right_context=Token("b"))),
            ]
        )
        system = LSystemGrammar(rules)
        axiom = tokenize("a")
        rewrite1 = list(system.rewrite(axiom))
        self.assertSequenceEqual(rewrite1, tokenize("ba"))
        rewrite2 = list(system.rewrite(rewrite1))
        self.assertSequenceEqual(rewrite2, tokenize("bba"))
        rewrite3 = list(system.rewrite(rewrite2))
        self.assertSequenceEqual(rewrite3, tokenize("abba"))

    def test_right_context_edge(self):
        rules = MultiDict(
            [
                ("a", RuleMapping(lambda c, t, l, r: tokenize("ba"))),
                ("b", RuleMapping(lambda c, t, l, r: tokenize("b"), right_context=Token("a"))),
                ("b", RuleMapping(lambda c, t, l, r: tokenize("a"), right_context=Token("b"))),
            ]
        )
        system = LSystemGrammar(rules)
        axiom = tokenize("b")
        rewrite = list(system.rewrite(axiom))
        self.assertSequenceEqual(rewrite, tokenize("b"))

    def test_context_both_sides(self):
        rules = MultiDict(
            [
                ("a", RuleMapping(lambda c, t, l, r: tokenize("aba"))),
                (
                    "b",
                    RuleMapping(
                        lambda c, t, l, r: tokenize("bb"),
                        left_context=Token("a"),
                        right_context=Token("a"),
                    ),
                ),
                (
                    "b",
                    RuleMapping(
                        lambda c, t, l, r: tokenize("ab"),
                        left_context=Token("b"),
                        right_context=Token("a"),
                    ),
                ),
            ]
        )
        system = LSystemGrammar(rules)
        axiom = tokenize("a")
        rewrite = list(system.rewrite(axiom))
        self.assertSequenceEqual(rewrite, tokenize("aba"))
        rewrite2 = list(system.rewrite(rewrite))
        self.assertSequenceEqual(rewrite2, tokenize("ababbaba"))
        # This set of rules generates a replacement for which there isn't a rule that matches.
        # So the test asserts that the unmatched token just gets left alone ( 'b' -> 'b' ).
        rewrite3 = list(system.rewrite(rewrite2))
        self.assertSequenceEqual(rewrite3, tokenize("ababbabababababbaba"))

    def test_locomotion(self):
        rules = MultiDict(
            {
                "a": RuleMapping(lambda c, t, l, r: tokenize("b"), left_context=Token("b")),
                "b": RuleMapping(lambda c, t, l, r: tokenize("a")),
            }
        )
        system = LSystemGrammar(rules)
        axiom = tokenize("baaaaaa")
        rewrite = list(system.rewrite(axiom))
        self.assertSequenceEqual(rewrite, tokenize("abaaaaa"))
        rewrite = list(system.rewrite(rewrite))
        self.assertSequenceEqual(rewrite, tokenize("aabaaaa"))
        rewrite = list(system.rewrite(rewrite))
        self.assertSequenceEqual(rewrite, tokenize("aaabaaa"))
        rewrite = list(system.rewrite(rewrite))
        self.assertSequenceEqual(rewrite, tokenize("aaaabaa"))
        rewrite = list(system.rewrite(rewrite))
        self.assertSequenceEqual(rewrite, tokenize("aaaaaba"))
        rewrite = list(system.rewrite(rewrite))
        self.assertSequenceEqual(rewrite, tokenize("aaaaaab"))

    def test_context_ignore(self):
        rules = MultiDict(
            {
                "0": RuleMapping(
                    lambda c, t, l, r: tokenize("1f1"),
                    left_context=Token("1"),
                    right_context=Token("1"),
                ),
                "1": RuleMapping(
                    lambda c, t, l, r: tokenize("0"),
                    left_context=Token("1"),
                    right_context=Token("1"),
                ),
            }
        )
        system = LSystemGrammar(rules, ignore={"f"})
        axiom = tokenize("f1f1f1")
        rewrite = list(system.rewrite(axiom))
        self.assertSequenceEqual(rewrite, tokenize("f1f0f1"))
        rewrite = list(system.rewrite(rewrite))
        self.assertSequenceEqual(rewrite, tokenize("f1f1f1f1"))
        rewrite = list(system.rewrite(rewrite))
        self.assertSequenceEqual(rewrite, tokenize("f1f0f0f1"))


class ParametricParsing(unittest.TestCase):
    """Test LSystemGrammar with parametric parsing."""


class CombinedParsing(unittest.TestCase):
    """Test LSystemGrammar with combined context sensitive, stochastic, and parametric parsing."""
