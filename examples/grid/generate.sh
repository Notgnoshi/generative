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

# BEGIN GRID_RADIAL_SNIPPET
grid --output-format lines --grid-type radial --size 20 |
    wkt2svg --output ./examples/grid/radial.svg
# END GRID_RADIAL_SNIPPET
extract_snippet GRID_RADIAL_SNIPPET

# BEGIN GRID_RADIAL_FILLED_SNIPPET
grid --output-format lines --grid-type radial --ring-fill-ratio 0.3 --size 20 |
    wkt2svg --output ./examples/grid/radial-filled.svg
# END GRID_RADIAL_FILLED_SNIPPET
extract_snippet GRID_RADIAL_FILLED_SNIPPET
