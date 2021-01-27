import unittest

from generative.misc.fogleworms import neighbors, place_worm
from sortedcontainers import SortedSet


class TestPlaceWorm(unittest.TestCase):
    def test_neighbors_1x1(self):
        adj = tuple(neighbors(0, 1))
        self.assertTupleEqual(adj, ())

    def test_neighbors_2x2(self):
        adj = tuple(neighbors(0, 2))
        self.assertTupleEqual(adj, (1, 2))
        adj = tuple(neighbors(3, 2))
        self.assertTupleEqual(adj, (1, 2))

    def test_neighbors_3x3(self):
        adj = tuple(neighbors(4, 3))
        self.assertTupleEqual(adj, (1, 3, 5, 7))
        adj = tuple(neighbors(5, 3))
        self.assertTupleEqual(adj, (2, 4, 8))

    def test_fill_1x1(self):
        size = 1
        unfilled = SortedSet([0])
        worm = place_worm(size, unfilled)

        self.assertTupleEqual(worm, (0,))
        self.assertSetEqual(unfilled, set())

    def test_fill_2x2(self):
        size = 2
        unfilled = SortedSet([0, 1, 2, 3])
        worm = place_worm(size, unfilled)

        self.assertTupleEqual(worm, (0, 1))
        self.assertSetEqual(unfilled, {2, 3})

        unfilled = SortedSet([0, 2])
        worm = place_worm(size, unfilled)

        self.assertTupleEqual(worm, (0, 2))
        self.assertSetEqual(unfilled, set())

        unfilled = SortedSet([1, 2, 3])
        worm = place_worm(size, unfilled)

        self.assertTupleEqual(worm, (1, 3))
        self.assertSetEqual(unfilled, {2,})

        # If there's not enough room for a full worm, you'll get the longest possible.
        unfilled = SortedSet([2])
        worm = place_worm(size, unfilled)

        self.assertTupleEqual(worm, (2,))
        self.assertSetEqual(unfilled, set())
