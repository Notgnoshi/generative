import unittest

from generative.lsystem.grammar import RuleMapping, Token
from generative.lsystem.production import RuleParser


class RuleParsingParser(unittest.TestCase):
    def test_simple(self):
        parser = RuleParser()
        rule = "a -> a,b"
        result = parser._parse(rule)

        self.assertEqual(result["lhs"], "a")
        self.assertSequenceEqual(result["rhs"], ["a", "b"])

    def test_probability(self):
        parser = RuleParser()
        rule = "a: 0.5 -> b"
        result = parser._parse(rule)

        self.assertEqual(result["lhs"], "a")
        self.assertEqual(result["probability"], 0.5)
        self.assertSequenceEqual(result["rhs"], ["b"])

    def test_left_context(self):
        parser = RuleParser()
        rule = "a<b -> cd,e"
        result = parser._parse(rule)

        self.assertEqual(result["lhs"], "b")
        self.assertSequenceEqual(result["rhs"], ["cd", "e"])
        self.assertEqual(result["left_context"], "a")

    def test_right_context(self):
        parser = RuleParser()
        rule = "a>b -> c"
        result = parser._parse(rule)

        self.assertEqual(result["lhs"], "a")
        self.assertEqual(result["right_context"], "b")
        self.assertSequenceEqual(result["rhs"], ["c"])

    def test_both_context(self):
        parser = RuleParser()
        rule = "l<a>r -> b"
        result = parser._parse(rule)

        self.assertEqual(result["left_context"], "l")
        self.assertEqual(result["right_context"], "r")

    def test_context_roll(self):
        parser = RuleParser()
        rule = "<<a -> b"
        result = parser._parse(rule)

        self.assertEqual(result["left_context"], "<")
        self.assertEqual(result["lhs"], "a")
        self.assertSequenceEqual(result["rhs"], ["b"])

        rule = "><a -> b"
        result = parser._parse(rule)

        self.assertEqual(result["left_context"], ">")
        self.assertEqual(result["lhs"], "a")
        self.assertSequenceEqual(result["rhs"], ["b"])

    def test_ignore(self):
        parser = RuleParser()
        rule = "#ignore a,b"
        result = parser._parse(rule)
        self.assertSequenceEqual(result["ignore"], ["a", "b"])

        rule = "#ignore:a,b"
        result = parser._parse(rule)
        self.assertSequenceEqual(result["ignore"], ["a", "b"])


def tokenize(s: str):
    return tuple(Token(c) for c in s)


class RuleParsingMappings(unittest.TestCase):
    def test_simple(self):
        parser = RuleParser()
        rule = "a -> a,b"
        lhs, mapping = parser.parse(rule)

        self.assertEqual(lhs, Token("a"))
        self.assertEqual(mapping, RuleMapping(tokenize("ab")))

    def test_probability(self):
        parser = RuleParser()
        rule = "a: 0.33 -> b"
        lhs, mapping = parser.parse(rule)

        self.assertEqual(lhs, Token("a"))
        self.assertEqual(mapping, RuleMapping(tokenize("b"), probability=0.33))

    def test_context(self):
        parser = RuleParser()
        rule = "left < tok>right:0.2->prod,uct"
        lhs, mapping = parser.parse(rule)

        self.assertEqual(lhs, Token("tok"))
        self.assertEqual(
            mapping,
            RuleMapping(
                (Token("prod"), Token("uct")),
                probability=0.2,
                left_context=Token("left"),
                right_context=Token("right"),
            ),
        )

    def test_ignore(self):
        parser = RuleParser()
        rule = "#ignore a,b"
        result = parser.parse(rule)
        self.assertIsNone(result)

        self.assertIn("a", parser.ignore)
        self.assertIn("b", parser.ignore)
