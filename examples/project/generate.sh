#!/bin/bash
set -o errexit
set -o pipefail
set -o nounset
set -o noclobber

# BEGIN PROJECT_SNIPPET
./tools/project.py --kind=isometric --input=examples/unit-cube.wkt |
    wkt2svg --scale=200 --output ./examples/project/isometric.svg
# END PROJECT_SNIPPET
extract_snippet PROJECT_SNIPPET
