#!/bin/bash
set -o errexit
set -o pipefail
set -o nounset
set -o noclobber

# BEGIN ASEMIC_SETUP_SNIPPET
glyphs() {
    local glyph_kind="$1"
    local number="$2"
    local size="$3"

    {
        for _ in $(seq "$number"); do
            $glyph_kind "$size"
        done
    } | pack --width 1000 --height 1000 --padding 20
}
# END ASEMIC_SETUP_SNIPPET
extract_snippet ASEMIC_SETUP_SNIPPET

ASEMIC_RANDOM_ROUNDED=./examples/asemic/random-rounded.svg
debug "Generating $ASEMIC_RANDOM_ROUNDED ..."
# BEGIN ASEMIC_RANDOM_ROUNDED_SNIPPET
random_rounded() {
    local size="$1"
    point-cloud --log-level WARN --domain unit-square --points 15 --scale 6 |
        urquhart --output-format tgf |
        traverse --log-level WARN --traversals 5 --length 5 --untraversed |
        transform --scale="$size" |
        smooth --iterations 4 |
        bundle
}
glyphs random_rounded 90 10 | wkt2svg --output $ASEMIC_RANDOM_ROUNDED
# END ASEMIC_RANDOM_ROUNDED_SNIPPET
extract_snippet ASEMIC_RANDOM_ROUNDED_SNIPPET

ASEMIC_RANDOM_TRIANGULATED=./examples/asemic/random-triangulated.svg
debug "Generating $ASEMIC_RANDOM_TRIANGULATED ..."
# BEGIN ASEMIC_RANDOM_TRIANGULATED_SNIPPET
random_triangulated() {
    local size="$1"
    point-cloud --log-level WARN --domain unit-square --points 10 --scale 6 |
        triangulate --output-format tgf |
        traverse --log-level WARN --traversals 3 --length 3 --remove-after-traverse |
        transform --scale="$size" |
        smooth --iterations 4 |
        bundle
}
glyphs random_triangulated 100 10 | wkt2svg --output $ASEMIC_RANDOM_TRIANGULATED
# END ASEMIC_RANDOM_TRIANGULATED_SNIPPET
extract_snippet ASEMIC_RANDOM_TRIANGULATED_SNIPPET

ASEMIC_GRID_ROUNDED=./examples/asemic/grid-rounded.svg
debug "Generating $ASEMIC_GRID_ROUNDED ..."
# BEGIN ASEMIC_GRID_ROUNDED_SNIPPET
grid_rounded() {
    local size="$1"
    grid --output-format graph --width=2 --height=3 |
        traverse --log-level WARN --traversals 5 --length 5 --remove-after-traverse |
        transform --scale="$size" |
        smooth --iterations 4 |
        bundle
}
glyphs grid_rounded 120 20 | wkt2svg --output $ASEMIC_GRID_ROUNDED
# END ASEMIC_GRID_ROUNDED_SNIPPET
extract_snippet ASEMIC_GRID_ROUNDED_SNIPPET

ASEMIC_GRID_BEVELED=./examples/asemic/grid-beveled.svg
debug "Generating $ASEMIC_GRID_BEVELED ..."
# BEGIN ASEMIC_GRID_BEVELED_SNIPPET
grid_beveled() {
    local size="$1"
    grid --output-format graph --width=2 --height=3 |
        traverse --log-level WARN --traversals 5 --length 5 --remove-after-traverse |
        transform --scale="$size" |
        smooth --iterations 1 |
        bundle
}
glyphs grid_beveled 120 20 | wkt2svg --output $ASEMIC_GRID_BEVELED
# END ASEMIC_GRID_BEVELED_SNIPPET
extract_snippet ASEMIC_GRID_BEVELED_SNIPPET

ASEMIC_GRID_TRIANGULATED=./examples/asemic/grid-triangulated.svg
debug "Generating $ASEMIC_GRID_TRIANGULATED ..."
# BEGIN ASEMIC_GRID_TRIANGULATED_SNIPPET
grid_triangulated() {
    local size="$1"
    grid --grid-type triangle --output-format graph --width=2 --height=3 |
        traverse --log-level WARN --traversals 4 --length 5 --remove-after-traverse |
        transform --scale="$size" |
        smooth --iterations 4 |
        bundle
}
glyphs grid_triangulated 100 20 | wkt2svg --output $ASEMIC_GRID_TRIANGULATED
# END ASEMIC_GRID_TRIANGULATED_SNIPPET
extract_snippet ASEMIC_GRID_TRIANGULATED_SNIPPET

ASEMIC_GRID_JAGGED=./examples/asemic/grid-jagged.svg
debug "Generating $ASEMIC_GRID_JAGGED ..."
# BEGIN ASEMIC_GRID_JAGGED_SNIPPET
grid_jagged() {
    local size="$1"
    grid --grid-type ragged --output-format graph --width=2 --height=3 |
        traverse --log-level WARN --traversals 4 --length 5 --remove-after-traverse |
        transform --scale="$size" |
        smooth --iterations 4 |
        bundle
}
glyphs grid_jagged 100 20 | wkt2svg --output $ASEMIC_GRID_JAGGED
# END ASEMIC_GRID_JAGGED_SNIPPET
extract_snippet ASEMIC_GRID_JAGGED_SNIPPET

ASEMIC_GRID_RADIAL=./examples/asemic/grid-radial.svg
debug "Generating $ASEMIC_GRID_RADIAL ..."
# BEGIN ASEMIC_GRID_RADIAL_SNIPPET
grid_radial() {
    local size="$1"
    grid --grid-type radial --output-format graph --width=5 --height=3 |
        traverse --log-level WARN --traversals 5 --length 5 --remove-after-traverse |
        transform --scale="$size" |
        smooth --iterations 4 |
        bundle
}
glyphs grid_radial 75 10 | wkt2svg --output $ASEMIC_GRID_RADIAL
# END ASEMIC_GRID_RADIAL_SNIPPET
extract_snippet ASEMIC_GRID_RADIAL_SNIPPET

ASEMIC_GRID_RADIAL_DENSE=./examples/asemic/grid-radial-dense.svg
debug "Generating $ASEMIC_GRID_RADIAL_DENSE ..."
# BEGIN ASEMIC_GRID_RADIAL_DENSE_SNIPPET
grid_radial_dense() {
    local size="$1"
    grid --grid-type radial --output-format graph --width=5 --height=4 --ring-fill-ratio=0.7 |
        traverse --log-level WARN --traversals 10 --length 30 |
        transform --scale="$size" |
        bundle
}
glyphs grid_radial_dense 64 10 | wkt2svg --output $ASEMIC_GRID_RADIAL_DENSE
# END ASEMIC_GRID_RADIAL_DENSE_SNIPPET
extract_snippet ASEMIC_GRID_RADIAL_DENSE_SNIPPET
