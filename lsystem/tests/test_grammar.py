import unittest

from lsystem.grammar import ContextFreeGrammar


class ContextFreeGrammarTests(unittest.TestCase):
    def test_simple(self):
        rules = {
            "a": "ab",
            "b": "a",
        }
        g = ContextFreeGrammar(rules)

        axiom = "a"
        axiom = "".join(g._apply(axiom))
        self.assertEqual(axiom, "ab")
        axiom = "".join(g._apply(axiom))
        self.assertEqual(axiom, "aba")
        axiom = "".join(g._apply(axiom))
        self.assertEqual(axiom, "abaab")
        axiom = "".join(g._apply(axiom))
        self.assertEqual(axiom, "abaababa")

    def test_stream(self):
        rules = {
            "a": "ab",
            "b": "a",
        }
        g = ContextFreeGrammar(rules)

        stream = g.apply("a", times=4)
        self.assertSequenceEqual("".join(stream), "abaababa")
