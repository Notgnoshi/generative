#!/bin/bash
set -o errexit
set -o pipefail
set -o nounset

# This script runs all the things in the README that modify the example files.

SOURCE="${BASH_SOURCE[0]}"
REPO_ROOT="$(cd -P "$(dirname "$SOURCE")" >/dev/null 2>&1 && pwd)/.."
REPO_ROOT="$(readlink --canonicalize --no-newline "${REPO_ROOT}")"

# Geometry Formats
$REPO_ROOT/tools/format.py --input $REPO_ROOT/examples/maya-tree-2.wkt --output-format flat --output $REPO_ROOT/examples/maya-tree-2.flat
$REPO_ROOT/tools/format.py --input-format flat --input $REPO_ROOT/examples/maya-tree-2.flat --output-format wkb --output $REPO_ROOT/examples/maya-tree-2.wkb
diff <($REPO_ROOT/tools/format.py --input-format wkb --input $REPO_ROOT/examples/maya-tree-2.wkb --output-format wkt) $REPO_ROOT/examples/maya-tree-2.wkt

# SVG Generation
$REPO_ROOT/tools/parse.py --config $REPO_ROOT/examples/sierpinski-tree.json |
    $REPO_ROOT/tools/interpret.py |
    $REPO_ROOT/tools/project.py --kind=yz |
    $REPO_ROOT/tools/wkt2svg.py -o $REPO_ROOT/examples/sierpinski-tree.svg

$REPO_ROOT/tools/parse.py --config $REPO_ROOT/examples/sierpinski-tree.json |
    $REPO_ROOT/tools/interpret.py |
    $REPO_ROOT/tools/project.py --kind=pca |
    $REPO_ROOT/tools/wkt2svg.py -o $REPO_ROOT/examples/sierpinski-tree-pca.svg

$REPO_ROOT/tools/parse.py --config $REPO_ROOT/examples/fractal-plant-1.json |
    $REPO_ROOT/tools/interpret.py --stepsize=3 --angle=22.5 |
    $REPO_ROOT/tools/project.py --kind=yz |
    $REPO_ROOT/tools/wkt2svg.py -o $REPO_ROOT/examples/fractal-plant-1.svg

$REPO_ROOT/tools/parse.py --config $REPO_ROOT/examples/fractal-plant-3d.json | $REPO_ROOT/tools/interpret.py --stepsize=3 --angle=22.5 >/tmp/plant.wkt
for projection in pca svd; do
    $REPO_ROOT/tools/project.py --kind=$projection --scale 10 --input /tmp/plant.wkt | $REPO_ROOT/tools/wkt2svg.py -o $REPO_ROOT/examples/plant-$projection.svg
done

# Converting the WKT to a graph
$REPO_ROOT/tools/parse.py --config $REPO_ROOT/examples/fractal-plant-1.json |
    $REPO_ROOT/tools/interpret.py --stepsize=3 --angle=22.5 |
    $REPO_ROOT/tools/project.py --kind=pca --output $REPO_ROOT/examples/fractal-plant-1.wkt

$REPO_ROOT/tools/geom2graph/build/src/geom2graph \
    --tolerance=0.001 \
    --input $REPO_ROOT/examples/fractal-plant-1.wkt \
    --output $REPO_ROOT/examples/fractal-plant-1.tgf

$REPO_ROOT/tools/parse.py --config $REPO_ROOT/examples/maya-tree-2.json |
    $REPO_ROOT/tools/interpret.py \
        --stepsize=1 \
        --angle=30 \
        --output=$REPO_ROOT/examples/maya-tree-2.wkt

$REPO_ROOT/tools/geom2graph/build/src/geom2graph \
    --tolerance=0.001 \
    --input $REPO_ROOT/examples/maya-tree-2.wkt \
    --output $REPO_ROOT/examples/maya-tree-2.tgf

# Generating Random L-Systems
mkdir -p $REPO_ROOT/examples/random-lsystems
for i in $(seq 0 13); do
    # $REPO_ROOT/tools/geom2graph/build/src/geom2graph --tolerance 1e-3 |  # Use geom2graph round trip to simplify geometries
    # $REPO_ROOT/tools/geom2graph/build/src/geom2graph --tolerance 1e-3 --graph2geom |
    jq ".[$i]" $REPO_ROOT/examples/random-lsystems/saved.json |
    $REPO_ROOT/tools/parse.py -c - -n $(jq ".[$i].iterations" $REPO_ROOT/examples/random-lsystems/saved.json) |
    $REPO_ROOT/tools/interpret.py -l ERROR -a $(jq ".[$i].angle" $REPO_ROOT/examples/random-lsystems/saved.json) |
    $REPO_ROOT/tools/project.py --scale $(jq ".[$i].scale" $REPO_ROOT/examples/random-lsystems/saved.json) --kind pca |
    $REPO_ROOT/tools/wkt2svg.py --output $REPO_ROOT/examples/random-lsystems/random-$i.svg
done
