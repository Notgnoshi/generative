#!/bin/bash
set -o errexit
set -o pipefail
set -o nounset
set -o noclobber

# BEGIN TRANSFORM_SNIPPET
transform <examples/unit-square.wkt \
    --rotation=45 |
    transform --scale=200 --scale-x=0.8 |
    wkt2svg --output ./examples/transform/square.svg
# END TRANSFORM_SNIPPET
extract_snippet TRANSFORM_SNIPPET
