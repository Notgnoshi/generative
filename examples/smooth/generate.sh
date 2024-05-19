#!/bin/bash
set -o errexit
set -o pipefail
set -o nounset
set -o noclobber

# BEGIN SMOOTH_SNIPPET1
smooth <examples/unit-square.wkt --iterations 1 |
    wkt2svg --scale=200 --output=examples/smooth/beveled.svg
# END SMOOTH_SNIPPET1
extract_snippet SMOOTH_SNIPPET1

# BEGIN SMOOTH_SNIPPET2
smooth <examples/unit-square.wkt --iterations 5 |
    wkt2svg --scale=200 --output=examples/smooth/rounded.svg
# END SMOOTH_SNIPPET2
extract_snippet SMOOTH_SNIPPET2
