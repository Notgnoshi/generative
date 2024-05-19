#!/bin/bash
set -o errexit
set -o pipefail
set -o nounset
set -o noclobber

# BEGIN WKT2SVG_SNIPPET
wkt2svg --output=examples/wkt2svg/styles.svg <<EOF
POINT(0 0)
POINT(100 100)
STROKEWIDTH(4)
STROKEDASHARRAY(6 1)
POINTRADIUS(20)
FILL(red)
POINT(50 50)
EOF
# END WKT2SVG_SNIPPET
extract_snippet WKT2SVG_SNIPPET
