#!/bin/bash
set -o errexit
set -o pipefail
set -o nounset
set -o noclobber

echo "POLYGON((3.5 3.5, 4.5 3.5, 4.5 4.5, 3.5 4.5, 3.5 3.5))" |
    cargo run --bin streamline -- \
        --min-x=0 \
        --max-x=10 \
        --min-y=0 \
        --max-y=10 \
        --delta-h=0.5 \
        --delta-t=0.5 \
        --time-steps=10 \
        --draw-vector-field \
        --vector-field-style="STROKE(gray)" \
        --vector-field-style="STROKEDASHARRAY(1)" \
        --streamline-kind=per-vertex \
        --draw-geometries \
        --streamline-style="STROKE(black)" \
        --streamline-style="STROKEDASHARRAY(0)" \
        --geometry-style="STROKE(red)" \
        --geometry-style="STROKEDASHARRAY(0)" \
        "$@" \
    | cargo run --bin wkt2svg -- \
        --padding \
        --scale 100 \
    | display -
