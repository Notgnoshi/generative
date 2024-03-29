#!/bin/bash
set -o errexit
set -o pipefail
set -o nounset

glyph() {
    local size="$1"
    local width=2
    local height=3

    grid --output-format graph --width="$width" --height="$height" |
        traverse --traversals 4 --length 5 --remove-after-traverse |
        transform --scale="$size" |
        smooth --iterations 4 |
        bundle
}

glyphs() {
    local number="$1"
    local size="$2"

    for _ in $(seq "$number"); do
        glyph "$size"
    done
}

glyphs 100 20 |
    pack --width 1000 --height 1000 --padding 20
