#!/bin/bash
set -o errexit
set -o pipefail
set -o nounset
# We _want_ to clobber any existing stash files in /tmp
set +o noclobber

# BEGIN TRIANGULATE_SNIPPET
point-cloud --seed 11878883030565683752 --points 20 --scale 200 |
    triangulate | sort | tee /tmp/triangulation.wkt |
    wkt2svg --output ./examples/urquhart/triangulation.svg
# END TRIANGULATE_SNIPPET
extract_snippet TRIANGULATE_SNIPPET

# BEGIN URQUHART_SNIPPET
point-cloud --seed 11878883030565683752 --points 20 --scale 200 |
    urquhart | sort >/tmp/urquhart.wkt
{
    echo "STROKE(gray)"
    echo "STROKEDASHARRAY(6)"
    # the triangulation minus the urquhart graph
    comm -23 /tmp/triangulation.wkt /tmp/urquhart.wkt
    echo "STROKE(black)"
    echo "STROKEDASHARRAY(none)"
    cat /tmp/urquhart.wkt
} | wkt2svg --output ./examples/urquhart/urquhart.svg
# END URQUHART_SNIPPET
extract_snippet URQUHART_SNIPPET

set -o noclobber
