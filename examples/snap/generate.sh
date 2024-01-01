#!/bin/bash
set -o errexit
set -o pipefail
set -o nounset

make_grid() {
    echo "LINESTRING(0   0, 0   0.5, 0   1, 0   1.5)"
    echo "LINESTRING(0.5 0, 0.5 0.5, 0.5 1, 0.5 1.5)"
    echo "LINESTRING(1   0, 1   0.5, 1   1, 1   1.5)"
    echo "LINESTRING(1.5 0, 1.5 0.5, 1.5 1, 1.5 1.5)"

    echo "LINESTRING(0 0,   0.5 0,   1 0,   1.5 0)"
    echo "LINESTRING(0 0.5, 0.5 0.5, 1 0.5, 1.5 0.5)"
    echo "LINESTRING(0 1.0, 0.5 1.0, 1 1.0, 1.5 1.0)"
    echo "LINESTRING(0 1.5, 0.5 1.5, 1 1.5, 1.5 1.5)"
}

points_before() {
    echo "POINT(0.125 0.125)"
    echo "POINT(0.125 0.75)"
    echo "POINT(0.125 1.375)"

    echo "POINT(0.75 0.125)"
    echo "POINT(0.75 0.75)"
    echo "POINT(0.75 1.375)"

    echo "POINT(1.375 0.125)"
    echo "POINT(1.375 0.75)"
    echo "POINT(1.375 1.375)"
}

points_after() {
    points_before |
        snap \
            --strategy regular-grid \
            --tolerance 0.5
}

snap_grid_example() {
    echo "STROKE(gray)"
    echo "STROKEDASHARRAY(6)"
    make_grid
    echo "STROKEDASHARRAY(none)"
    echo "STROKE(black)"
    points_before
    echo "STROKE(red)"
    # NOTE: The SVG y-axis is flipped, so rounding away from zero visually LOOKS like rounding down
    points_after
}

snap_grid_example |
    wkt2svg --scale 200 --output "$REPO_ROOT/examples/snap/grid.svg"

# Four squares with 0.1 of padding between each of them
graph_before() {
    echo "0     POINT(0 0)"
    echo "1     POINT(1 0)"
    echo "2     POINT(1.1 0)"
    echo "3     POINT(2.1 0)"

    echo "4     POINT(0 1)"
    echo "5     POINT(1 1)"
    echo "6     POINT(1.1 1)"
    echo "7     POINT(2.1 1)"

    echo "8     POINT(0 1.1)"
    echo "9     POINT(1 1.1)"
    echo "10    POINT(1.1 1.1)"
    echo "11    POINT(2.1 1.1)"

    echo "12    POINT(0 2.1)"
    echo "13    POINT(1 2.1)"
    echo "14    POINT(1.1 2.1)"
    echo "15    POINT(2.1 2.1)"

    echo "#"

    echo "0 1"
    echo "1 5"
    echo "5 4"
    echo "4 0"

    echo "2 3"
    echo "3 7"
    echo "7 6"
    echo "6 2"

    echo "8 9"
    echo "9 13"
    echo "13 12"
    echo "12 8"

    echo "10 11"
    echo "11 15"
    echo "15 14"
    echo "14 10"
}

graph_after() {
    local strategy="$1"
    local tolerance="$2"

    graph_before |
        snap \
            --strategy "$strategy" \
            --tolerance "$tolerance" \
            --input-format tgf
}

snap_graph_example() {
    local strategy="$1"
    local tolerance="$2"

    echo "STROKEDASHARRAY(none)"
    echo "STROKE(black)"
    graph_after "$strategy" "$tolerance" | geom2graph --graph2geom
    echo "STROKE(gray)"
    echo "STROKEDASHARRAY(6)"
    graph_before | geom2graph --graph2geom
}

graph_before | geom2graph --graph2geom |
    wkt2svg --scale 200 --output "$REPO_ROOT/examples/snap/graph-before.svg"

snap_graph_example "closest-point" "0.2" |
    wkt2svg  --scale 200 --output "$REPO_ROOT/examples/snap/graph-closest.svg"

snap_graph_example "regular-grid" "1.0" |
    wkt2svg  --scale 200 --output "$REPO_ROOT/examples/snap/graph-grid.svg"
