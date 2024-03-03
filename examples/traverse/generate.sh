#!/bin/bash
set -o errexit
set -o pipefail
set -o nounset
set -o noclobber

# BEGIN TRAVERSE_SNIPPET
grid --grid-type hexagon --output-format graph |
    traverse \
        --seed=10268415722561053759 \
        --traversals 2 \
        --length 20 \
        --untraversed |
    wkt2svg --scale 100 --output ./examples/traverse/hex-walk.svg
# END TRAVERSE_SNIPPET
extract_snippet TRAVERSE_SNIPPET
