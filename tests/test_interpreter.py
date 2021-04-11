import io
import unittest

from shapely.geometry import LineString

from generative.lsystem.interpreter import LSystemInterpeter


# Debugging failing tests is impossible when all you get is
# '<shapely.geometry.linestring.LineString object at 0x7f10d52ea880>'
def better_repr(self):
    return self.wkt


LineString.__repr__ = better_repr


class InterpreterTests(unittest.TestCase):
    def setUp(self):
        self.i = LSystemInterpeter("default", 1.0, 90)

    def test_forward(self):
        commands = io.StringIO("FG")
        expected = [LineString([(0, 0, 0), (0, 0, 2)])]
        tokens = self.i.tokenize(commands)
        lines = list(self.i.interpret(tokens))
        self.assertListEqual(lines, expected)

    def test_forward_no_draw(self):
        commands = io.StringIO("fGF")
        expected = [LineString([(0, 0, 1), (0, 0, 3)])]
        tokens = self.i.tokenize(commands)
        lines = list(self.i.interpret(tokens))
        self.assertListEqual(lines, expected)

    def test_forward_no_draw_cuts_line(self):
        commands = io.StringIO("FfF")
        expected = [
            LineString([(0, 0, 0), (0, 0, 1)]),
            LineString([(0, 0, 2), (0, 0, 3)]),
        ]
        tokens = self.i.tokenize(commands)
        lines = list(self.i.interpret(tokens))
        self.assertListEqual(lines, expected)

    def test_yaw(self):
        commands = io.StringIO("F+F-F")
        expected = [LineString([(0, 0, 0), (0, 0, 1), (0, -1, 1), (0, -1, 2)])]
        tokens = self.i.tokenize(commands)
        lines = list(self.i.interpret(tokens))
        self.assertEqual(len(lines), len(expected))
        for actual, desired in zip(lines, expected):
            self.assertTrue(actual.almost_equals(desired))

    def test_draw_no_draw(self):
        commands = io.StringIO("dF+FDFF")
        expected = [LineString([(0, -1, 1), (0, -3, 1)])]
        tokens = self.i.tokenize(commands)
        lines = list(self.i.interpret(tokens))
        self.assertEqual(len(lines), len(expected))
        for actual, desired in zip(lines, expected):
            self.assertTrue(actual.almost_equals(desired))

    def test_draw_no_draw_cuts_line(self):
        commands = io.StringIO("dF+DFdFDF")
        expected = [
            LineString([(0, 0, 1), (0, -1, 1)]),
            LineString([(0, -2, 1), (0, -3, 1)]),
        ]
        tokens = self.i.tokenize(commands)
        lines = list(self.i.interpret(tokens))
        self.assertEqual(len(lines), len(expected))
        for actual, desired in zip(lines, expected):
            self.assertTrue(
                actual.almost_equals(desired), f"actual: {actual.wkt} expected: {desired.wkt}"
            )

    def test_pitch(self):
        commands = io.StringIO("^FFvF")
        expected = [LineString([(0, 0, 0), (2, 0, 0), (2, 0, 1)])]
        tokens = self.i.tokenize(commands)
        lines = list(self.i.interpret(tokens))
        self.assertEqual(len(lines), len(expected))
        for actual, desired in zip(lines, expected):
            self.assertTrue(actual.almost_equals(desired))

    def test_roll(self):
        # Roll is about the longitudinal axis, so roll + forward won't change direction.
        commands = io.StringIO(">+F")
        expected = [LineString([(0, 0, 0), (1, 0, 0)])]
        tokens = self.i.tokenize(commands)
        lines = list(self.i.interpret(tokens))
        self.assertEqual(len(lines), len(expected))
        for actual, desired in zip(lines, expected):
            self.assertTrue(actual.almost_equals(desired))

    def test_flip(self):
        commands = io.StringIO("F|F")
        expected = [LineString([(0, 0, 0), (0, 0, 1), (0, 0, 0)])]
        tokens = self.i.tokenize(commands)
        lines = list(self.i.interpret(tokens))
        self.assertEqual(len(lines), len(expected))
        for actual, desired in zip(lines, expected):
            self.assertTrue(actual.almost_equals(desired))

    def test_stack(self):
        commands = io.StringIO("FF[+FF]-FF")
        expected = [
            LineString([(0, 0, 0), (0, 0, 2), (0, -2, 2)]),
            LineString([(0, 0, 2), (0, 2, 2)]),
        ]
        tokens = self.i.tokenize(commands)
        lines = list(self.i.interpret(tokens))
        self.assertListEqual(lines, expected)

    def test_orientation_change_doesnt_drop_points(self):
        # little 'f' means step forward without drawing, so this shouldn't generate a line.
        commands = io.StringIO(">>>f>>f")
        tokens = self.i.tokenize(commands)
        lines = list(self.i.interpret(tokens))
        self.assertEqual(len(lines), 0)
