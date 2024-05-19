#!/bin/bash
set -o errexit
set -o pipefail
set -o nounset
set -o noclobber

GRID_GRAPH_OUTPUT=$(
    # BEGIN GRID_GRAPH_SNIPPET
    grid --output-format graph --grid-type quad --width 1 --height 1
    # END GRID_GRAPH_SNIPPET
)
extract_snippet GRID_GRAPH_SNIPPET 4
# BEGIN GRID_HEX_SNIPPET
grid --output-format lines --grid-type hexagon --size 20 |
    wkt2svg --output ./examples/grid/hex.svg
# END GRID_HEX_SNIPPET
extract_snippet GRID_HEX_SNIPPET
