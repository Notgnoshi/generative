import itertools
import unittest

from lsystem.grammar import ContextFreeGrammar


class ContextFreeGrammarTests(unittest.TestCase):
    def test_apply_once(self):
        rules = {
            "a": "ab",
            "b": "a",
        }
        g = ContextFreeGrammar(rules)

        axiom = "a"
        axiom = "".join(g._apply_once(axiom))
        self.assertEqual(axiom, "ab")
        axiom = "".join(g._apply_once(axiom))
        self.assertEqual(axiom, "aba")
        axiom = "".join(g._apply_once(axiom))
        self.assertEqual(axiom, "abaab")
        axiom = "".join(g._apply_once(axiom))
        self.assertEqual(axiom, "abaababa")

    def test_iapply(self):
        rules = {
            "a": "ab",
            "b": "a",
        }
        g = ContextFreeGrammar(rules)
        axiom = "a"

        generator = itertools.islice(g.iapply(axiom), 4)
        for tokens, expected in zip(generator, ["ab", "aba", "abaab", "abaababa"]):
            self.assertEqual("".join(tokens), expected)

    def test_apply(self):
        rules = {
            "a": "ab",
            "b": "a",
        }
        g = ContextFreeGrammar(rules)

        stream = g.apply("a", times=4)
        self.assertSequenceEqual("".join(stream), "abaababa")
