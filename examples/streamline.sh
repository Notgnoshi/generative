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
        --time-steps=5 \
        --draw-vector-field \
        --vector-field-style="STROKE(gray)" \
        --streamline-kind=per-vertex \
        --streamline-style="STROKE(black)" \
        --draw-geometries \
        --geometry-style="STROKE(red)" \
        "$@" \
    | cargo run --bin wkt2svg -- \
        --padding \
        --scale 100 \
    | display -
