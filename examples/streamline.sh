#!/bin/bash
set -o errexit
set -o pipefail
set -o nounset
set -o noclobber

# echo "POLYGON((3.5 3.5, 4.5 3.5, 4.5 4.5, 3.5 4.5, 3.5 3.5))" |
#     cargo run --bin streamline -- \
#         --min-x=0 \
#         --max-x=10 \
#         --min-y=0 \
#         --max-y=10 \
#         --delta-h=0.1 \
#         --delta-t=0.05 \
#         --time-steps=10 \
#         --draw-vector-field \
#         --vector-field-style="STROKE(gray)" \
#         --vector-field-style="STROKEDASHARRAY(1)" \
#         --streamline-kind=per-vertex \
#         --draw-geometries \
#         --streamline-style="STROKE(black)" \
#         --streamline-style="STROKEDASHARRAY(0)" \
#         --geometry-style="STROKE(red)" \
#         --geometry-style="STROKEDASHARRAY(0)" \
#         "$@" \
#     | cargo run --bin wkt2svg -- \
#         --padding \
#         --scale 100 \
#     | display -

cargo run --bin point-cloud -- \
    --points 40 \
    --scale 1 \
    --domain=unit-square |
    cargo run --bin streamline -- \
        --min-x=0 \
        --max-x=1 \
        --min-y=0 \
        --max-y=1 \
        --delta-h=0.5 \
        --delta-t=0.05 \
        --time-steps=50 \
        --draw-geometries \
        --geometry-style="FILL(black)" \
        --geometry-style="POINTRADIUS(2)" |
    cargo run --bin wkt2svg -- \
        --padding \
        --scale 1000 |
    display -
