#!/bin/bash
set -o errexit
set -o pipefail
set -o nounset
set -o noclobber

# BEGIN DLA_ORGANIC_SNIPPET
dla \
    --seed 461266331856721221 \
    --seeds 2 \
    --attraction-distance 10 \
    --min-move-distance 1 \
    --stubbornness 10 \
    --particle-spacing 0.1 |
    geom2graph --graph2geom --tolerance=0.001 |
    wkt2svg --scale 30 --output ./examples/dla/organic.svg
# END DLA_ORGANIC_SNIPPET
extract_snippet DLA_ORGANIC_SNIPPET
