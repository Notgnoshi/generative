#!/bin/bash
set -o errexit
set -o pipefail
set -o nounset
set -o noclobber

# BEGIN SNAP_CREATE_GRID
generate_grid() {
    transform <examples/unit-square.wkt --offset-y=1.1
    transform <examples/unit-square.wkt --offset-x=1.1 --offset-y=1.1
    cat examples/unit-square.wkt
    transform <examples/unit-square.wkt --offset-x=1.1
}

generate_grid | wkt2svg --scale=200 --output=examples/snap/grid.svg
# END SNAP_CREATE_GRID
extract_snippet SNAP_CREATE_GRID

# BEGIN SNAP_CLOSEST
generate_grid |
    geom2graph |
    snap --input-format=tgf --strategy=closest-point --tolerance=0.2 |
    geom2graph --graph2geom |
    wkt2svg --scale=200 --output=examples/snap/snap-closest.svg
# END SNAP_CLOSEST
extract_snippet SNAP_CLOSEST

# BEGIN SNAP_GRID
generate_grid |
    geom2graph |
    snap --input-format=tgf --strategy=regular-grid --tolerance=1.0 |
    geom2graph --graph2geom |
    wkt2svg --scale=200 --output=examples/snap/snap-grid.svg
# END SNAP_GRID
extract_snippet SNAP_GRID
