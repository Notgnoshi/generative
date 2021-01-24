"""Fogleworms.

How many distinct tilings (rotation and reflection invariant) of N worms of length N are there
for an NxN grid? Problem from https://twitter.com/FogleBird/status/1349563985268531200

Treat each grid as an array

    0 1 2   0 1 2 3
    3 4 5   4 5 6 7
    6 7 8   8 9 A B
            C D E F

Each "worm" is a tuple of indices. For example the worm

    # # 2
    3 # 5
    6 7 8

would be the tuple (0, 1, 4).
Note that there are two solutions for a 3x3 grid:

    1 2 3   1 2 2
    1 2 3   1 2 3
    1 2 3   1 3 3

But for a 4x4 grid, there are _many_ solutions.

For some starting configuration,

    # # # 3
    4 5 # 7
    8 9 A B
    C D E F

Start with the smallest unmarked index, in this case 3, and "flood fill" with priority given to the
smallest reachable index.

    # # # X
    4 5 # X
    8 9 X X
    C D E F

Then repeat.

    # # # #
    X X # #
    X X # #
    C D E F

    # # # #
    # # # #
    # # # #
    X X X X

Now we've filled the whole grid, so we backtrack.

    # # # #
    X X # #
    X X # #
    C D E F

Now, we'll pop off each element, and try to place a new worm at the same time.

    # # # #
    X X # #
    8 X # #
    C D E F

    # # # #
    X X # #
    8 X # #
    C X E F

Success! But now when we try to place a new worm, it'll fail

    # # # #
    # # # #
    X # # #
    X # E F

So we backtrack and try something else.

    # # # #
    X X # #
    8 X # #
    C X E F

Again, we'll pop worm segments off one-by-one and try to place a new worm at the same time.

    # # # #
    X X # #
    8 X # #
    C D E F

    # # # #
    X X # #
    8 9 # #
    C D E F

    # # # #
    X 5 # #
    8 9 # #
    C D E F

    # # # #
    X 5 # #
    X X # #
    C X E F

    ...
"""
from typing import List, Tuple, Union

from sortedcontainers import SortedSet

Worm = Tuple[int]


def place_worm(grid, unmarked: SortedSet) -> Union[Worm, None]:
    """Place a valid worm on the given grid.

    Starts with the lowest unfilled index, and flood fills in the four cardinal directions with
    priority given to the lowest remaining unfilled index.
    """
