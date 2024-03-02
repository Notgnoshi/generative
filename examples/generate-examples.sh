#!/bin/bash
set -o errexit
set -o pipefail
set -o nounset

# This script runs all the things in the README that modify the example files.

SOURCE="${BASH_SOURCE[0]}"
REPO_ROOT="$(cd -P "$(dirname "$SOURCE")" >/dev/null 2>&1 && pwd)/.."
export REPO_ROOT="$(readlink --canonicalize --no-newline "${REPO_ROOT}")"

cd "$REPO_ROOT"
cargo build --release

PATH="$REPO_ROOT/target/release/:$PATH"

echo "Snap..."
"$REPO_ROOT/examples/snap/generate.sh"

echo "Bitwise..."
bitwise --x-max 96 --y-max 96 "(x & y) & (x ^ y) % 11" |
    transform --scale 10 |
    wkt2svg --output "$REPO_ROOT/examples/bitwise.svg"

echo "DLA..."
dla \
    --seed 461266331856721221 \
    --seeds 2 \
    --attraction-distance 10 \
    --min-move-distance 1 \
    --stubbornness 10 \
    --particle-spacing 0.1 |
    geom2graph --graph2geom |
    transform --scale 20 |
    wkt2svg --output "$REPO_ROOT/examples/diffusion-limited-aggregation/organic.svg"

echo "Lindenmayer..."
"$REPO_ROOT/tools/parse-production-rules.py" --config "$REPO_ROOT/examples/sierpinski-tree.json" |
    "$REPO_ROOT/tools/interpret-lstring.py" |
    "$REPO_ROOT/tools/project.py" --kind=yz |
    wkt2svg --output "$REPO_ROOT/examples/sierpinski-tree.svg"

"$REPO_ROOT/tools/parse-production-rules.py" --config "$REPO_ROOT/examples/maya-tree-2.json" |
    "$REPO_ROOT/tools/interpret-lstring.py" \
        --stepsize=1 \
        --angle=30 \
        --output="$REPO_ROOT/examples/maya-tree-2.wkt"

echo "Projections..."
"$REPO_ROOT/tools/parse-production-rules.py" --config "$REPO_ROOT/examples/sierpinski-tree.json" |
    "$REPO_ROOT/tools/interpret-lstring.py" |
    "$REPO_ROOT/tools/project.py" --kind=pca |
    wkt2svg --output "$REPO_ROOT/examples/sierpinski-tree-pca.svg"

"$REPO_ROOT/tools/parse-production-rules.py" --config "$REPO_ROOT/examples/fractal-plant-1.json" |
    "$REPO_ROOT/tools/interpret-lstring.py" --stepsize=3 --angle=22.5 |
    "$REPO_ROOT/tools/project.py" --kind=yz --output "$REPO_ROOT/examples/fractal-plant-1.wkt"
"$REPO_ROOT/tools/project.py" --kind=pca --input "$REPO_ROOT/examples/fractal-plant-1.wkt" --output "$REPO_ROOT/examples/fractal-plant-1-pca.wkt"
wkt2svg --input "$REPO_ROOT/examples/fractal-plant-1.wkt" --output "$REPO_ROOT/examples/fractal-plant-1.svg"
wkt2svg --input "$REPO_ROOT/examples/fractal-plant-1-pca.wkt" --output "$REPO_ROOT/examples/fractal-plant-1-pca.svg"

echo "Triangulation..."
point-cloud --seed 11878883030565683752 --points 20 --scale 200 >/tmp/points.wkt
triangulate </tmp/points.wkt | sort >/tmp/delaunay.wkt
wkt2svg </tmp/delaunay.wkt >examples/delaunay.svg

echo "Urquhart..."
urquhart </tmp/points.wkt | sort >/tmp/urquhart.wkt
comm -23 /tmp/delaunay.wkt /tmp/urquhart.wkt >/tmp/difference.wkt
{
    echo "STROKE(gray)"
    echo "STROKEDASHARRAY(6)"
    cat /tmp/difference.wkt
    echo "STROKE(black)"
    echo "STROKEDASHARRAY(none)"
    cat /tmp/urquhart.wkt
} >/tmp/combined.wkt
wkt2svg </tmp/combined.wkt >examples/urquhart.svg
