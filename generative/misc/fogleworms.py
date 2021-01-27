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
from typing import Tuple

import numpy as np
from sortedcontainers import SortedSet

Worm = Tuple[int]
Grid = np.ndarray


def neighbors(index, size) -> Tuple[Tuple[int]]:
    """Get the neighbors of the given 1D index of a 2D square matrix.

    The neighbors do _not_ wrap around, so there could be 2, 3, or 4 neighbors yielded.
    """
    north = index - size
    east = index + 1
    south = index + size
    west = index - 1

    # Return an ordered tuple as a micro optimization that won't matter in the scheme of things.
    if north >= 0:
        yield north
    edge = index % size
    if edge != 0:
        yield west
    if edge != (size - 1):
        yield east
    if south < size ** 2:
        yield south


def place_worm(size, unfilled: SortedSet) -> Worm:
    """Place a valid worm on the given grid.

    Starts with the lowest unfilled index, and flood fills in the four cardinal directions with
    priority given to the lowest remaining unfilled index.

    The unfilled object will be updated with the new worm, and a tuple of indices
    will be returned for the newly added worm.

    Note that the worm may container fewer than 'size' segments.
    """
    current = unfilled.pop(0)
    worm = [current]

    for _ in range(size - 1):
        # Look for the next current index
        for neighbor in neighbors(current, size):
            if neighbor in unfilled:
                worm.append(neighbor)
                unfilled.remove(neighbor)
                current = neighbor
                break
    return tuple(worm)
