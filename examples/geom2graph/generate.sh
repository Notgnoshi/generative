#!/bin/bash
set -o errexit
set -o pipefail
set -o nounset
set +o noclobber # Enable clobbering files in /tmp/

# BEGIN GEOM2GRAPH_SNIPPET
# Offset the unit square to result in two overlapping geometries
transform <examples/unit-square.wkt --offset-x=0.5 --offset-y=0.5 >/tmp/offset-square.wkt
cat examples/unit-square.wkt /tmp/offset-square.wkt | geom2graph >/tmp/graph.tgf
# END GEOM2GRAPH_SNIPPET
extract_snippet GEOM2GRAPH_SNIPPET

GEOM2GRAPH_OUTPUT=$(cat /tmp/graph.tgf)

# BEGIN GEOM2GRAPH_SNIPPET2
# Extract the vertices, so we can overlay them
grep --only-matching 'POINT.*$' /tmp/graph.tgf >/tmp/vertices.wkt
# Convert graph back into a set of geometries
geom2graph --graph2geom </tmp/graph.tgf >/tmp/offset-squares.wkt

{
    cat /tmp/offset-squares.wkt
    echo "STROKE(red)"
    cat /tmp/vertices.wkt
} | wkt2svg --scale=200 --output ./examples/geom2graph/offset-squares.svg

# END GEOM2GRAPH_SNIPPET2
extract_snippet GEOM2GRAPH_SNIPPET2
